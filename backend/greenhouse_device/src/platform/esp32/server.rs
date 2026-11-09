//! picoserve front end for [`crate::handler`].

use alloc::vec::Vec;

use embassy_net::Stack;
use greenhouse_core::smart_device_dto::endpoints::{ACTIVATE, CONFIG, READ, STATUS, WRITE};
use picoserve::io::Write;
use picoserve::response::{Content, IntoResponse, ResponseWriter, StatusCode};
use picoserve::routing::{PathRouter, get, post};
use picoserve::{ResponseSent, Router, Server};
use serde::{Serialize, de::DeserializeOwned};

use crate::device_builder::{DeviceBuilder, DeviceState};
use crate::handler::{self, Reply};

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

impl IntoResponse for Reply {
    async fn write_to<R: picoserve::io::Read, W: ResponseWriter<Error = R::Error>>(
        self,
        connection: picoserve::response::Connection<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        let status = StatusCode::new(self.status.as_u16());
        match self.body {
            Some(body) => {
                (status, Json(body))
                    .write_to(connection, response_writer)
                    .await
            }
            None => status.write_to(connection, response_writer).await,
        }
    }
}

pub(crate) async fn serve<T>(builder: DeviceBuilder<T>) -> !
where
    T: Clone + Default + Serialize + DeserializeOwned + 'static,
{
    let DeviceBuilder { state, server } = builder;
    let port = state.config().port;
    let app = router(state);
    let config = picoserve::Config::const_default().keep_connection_alive();

    log::info!("serving device API on port {port}");
    embassy_futures::join::join_array(core::array::from_fn::<_, WORKERS, _>(|id| {
        worker(id, &app, &config, server.stack, port)
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
        .route(READ, get(move || handler::read(state)))
        .route(
            WRITE,
            post(move |body: Vec<u8>| async move { handler::write(state, &body).await }),
        )
        .route(
            CONFIG,
            get(move || handler::get_config(state))
                .post(move |body: Vec<u8>| async move { handler::post_config(state, &body).await }),
        )
        .route(STATUS, get(move || handler::status(state)))
        .route(
            ACTIVATE,
            post(move |body: Vec<u8>| async move { handler::activate(state, &body).await }),
        )
}
