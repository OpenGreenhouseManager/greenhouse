//! HTTP routes; same contract as `greenhouse_core::smart_device_interface::handler`.
//!
//! All routes are always registered. A route whose handler the device mode does
//! not have (e.g. `/write` on an output device) answers 404, matching the Linux
//! routers, which do not register it at all.

use alloc::boxed::Box;
use alloc::vec::Vec;

use embassy_net::Stack;
use greenhouse_core::smart_device_dto::activation::ActivateRequestDto;
use greenhouse_core::smart_device_dto::config::{self as dto, ConfigRequestDto, ConfigResponseDto};
use greenhouse_core::smart_device_dto::endpoints::{ACTIVATE, CONFIG, READ, STATUS, WRITE};
use greenhouse_core::smart_device_dto::read::ReadResponseDto;
use greenhouse_core::smart_device_dto::write::WriteRequestDto;
use picoserve::io::Write;
use picoserve::response::{Content, StatusCode};
use picoserve::routing::{PathRouter, get, post};
use picoserve::{Router, Server};
use serde::{Serialize, de::DeserializeOwned};

use crate::Arc;
use crate::config::{Config, Mode, ScriptingApi};
use crate::device_builder::{DeviceBuilder, DeviceState};

/// Concurrent connections served.
const WORKERS: usize = 3;
/// Holds the request head and body; the config POST is the largest request.
const HTTP_BUF_LEN: usize = 4096;
const TCP_BUF_LEN: usize = 1024;

/// A serialized JSON response body.
struct Json(Vec<u8>);

impl Content for Json {
    fn content_type(&self) -> &'static str {
        "application/json"
    }

    fn content_length(&self) -> usize {
        self.0.len()
    }

    async fn write_content<W: Write>(self, mut writer: W) -> Result<(), W::Error> {
        writer.write_all(&self.0).await
    }
}

type JsonResponse = Result<(StatusCode, Json), StatusCode>;

fn json<V: Serialize>(value: &V) -> JsonResponse {
    match serde_json::to_vec(value) {
        Ok(body) => Ok((StatusCode::OK, Json(body))),
        Err(e) => {
            log::error!("response serialization failed: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn parse<V: DeserializeOwned>(body: &[u8]) -> Result<V, StatusCode> {
    serde_json::from_slice(body).map_err(|e| {
        log::warn!("invalid request body: {e}");
        StatusCode::BAD_REQUEST
    })
}

pub(crate) async fn serve<T>(builder: DeviceBuilder<T>) -> !
where
    T: Clone + Default + Serialize + DeserializeOwned + 'static,
{
    let DeviceBuilder { state, stack } = builder;
    let port = state.config().port;
    // Lives for the rest of the program; handlers capture `&'static`.
    let state: &'static DeviceState<T> = Box::leak(Box::new(state));
    let app = router(state);
    let config = picoserve::Config::const_default().keep_connection_alive();

    log::info!("serving device API on port {port}");
    embassy_futures::join::join_array(core::array::from_fn::<_, WORKERS, _>(|id| {
        worker(id, &app, &config, stack, port)
    }))
    .await;
    unreachable!("HTTP workers never return")
}

async fn worker<P: PathRouter>(
    id: usize,
    app: &Router<P>,
    config: &picoserve::Config,
    stack: Stack<'static>,
    port: u16,
) {
    let mut http_buf = [0u8; HTTP_BUF_LEN];
    let mut tcp_rx = [0u8; TCP_BUF_LEN];
    let mut tcp_tx = [0u8; TCP_BUF_LEN];
    loop {
        Server::new(app, config, &mut http_buf)
            .listen_and_serve(id, stack, port, &mut tcp_rx, &mut tcp_tx)
            .await;
    }
}

fn router<T>(state: &'static DeviceState<T>) -> Router<impl PathRouter>
where
    T: Clone + Default + Serialize + DeserializeOwned + 'static,
{
    Router::new()
        .route(READ, get(move || read(state)))
        .route(WRITE, post(move |body: Vec<u8>| write(state, body)))
        .route(
            CONFIG,
            get(move || get_config(state)).post(move |body: Vec<u8>| post_config(state, body)),
        )
        .route(STATUS, get(move || status(state)))
        .route(ACTIVATE, post(move |body: Vec<u8>| activate(state, body)))
}

async fn read<T: Clone + Default>(state: &DeviceState<T>) -> JsonResponse {
    let handler = state.read_handler.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let data = handler(state.config()).await;
    json(&ReadResponseDto { data })
}

async fn write<T: Clone + Default>(state: &DeviceState<T>, body: Vec<u8>) -> StatusCode {
    let Some(handler) = state.write_handler.as_ref() else {
        return StatusCode::NOT_FOUND;
    };
    match parse::<WriteRequestDto>(&body) {
        Ok(payload) => StatusCode::new(handler(payload.data, state.config()).await.as_u16()),
        Err(status) => status,
    }
}

async fn status<T: Clone + Default>(state: &DeviceState<T>) -> JsonResponse {
    json(&(state.status_handler)(state.config()).await)
}

async fn get_config<T>(state: &DeviceState<T>) -> JsonResponse
where
    T: Clone + Default + Serialize,
{
    let config = state.config();
    let (mode, input_type, output_type) = match &state.mode {
        Mode::Input(t) => (dto::Mode::Input, Some(t.clone()), None),
        Mode::Output(t) => (dto::Mode::Output, None, Some(t.clone())),
        Mode::InputOutput(i, o) => (dto::Mode::InputOutput, Some(i.clone()), Some(o.clone())),
    };
    // `Option` like the Linux handler, which answers `null` when the config
    // file is unreadable; here the config is always in memory.
    json(&Some(ConfigResponseDto {
        mode,
        input_type,
        output_type,
        scripting_api: config.scripting_api.as_ref().map(|s| dto::ScriptingApi {
            url: s.url.clone(),
            token: s.token.clone(),
        }),
        additional_config: config.additional_config.clone(),
    }))
}

async fn post_config<T>(state: &DeviceState<T>, body: Vec<u8>) -> StatusCode
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    let request = match parse::<ConfigRequestDto<T>>(&body) {
        Ok(request) => request,
        Err(status) => return status,
    };
    let config = (state.config_interceptor)(request, state.config()).await;
    persist(state, config).await
}

async fn activate<T>(state: &DeviceState<T>, body: Vec<u8>) -> StatusCode
where
    T: Clone + Default + Serialize,
{
    let request = match parse::<ActivateRequestDto>(&body) {
        Ok(request) => request,
        Err(status) => return status,
    };
    let mut config = (*state.config()).clone();
    config.scripting_api = Some(ScriptingApi {
        url: request.url,
        token: request.token,
    });
    persist(state, config).await
}

/// Stores the config in flash, then makes it current (like the Linux version,
/// which only swaps the in-memory config after the file write succeeded).
async fn persist<T>(state: &DeviceState<T>, config: Config<T>) -> StatusCode
where
    T: Clone + Default + Serialize,
{
    let Ok(bytes) = serde_json::to_vec(&config) else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    if let Err(e) = state.storage.lock().await.store(&bytes) {
        log::error!("failed to persist config: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    *state.config.borrow_mut() = Arc::new(config);
    StatusCode::OK
}
