//! The device API, independent of the HTTP server: each function takes the
//! request body and returns the status plus an optional JSON body. The
//! platform's server only routes requests here.
//!
//! All routes exist on every device. One the device mode has no handler for
//! (e.g. `/write` on an output device) answers 404. Malformed bodies get 400.

use alloc::vec::Vec;

use greenhouse_core::smart_device_dto::activation::ActivateRequestDto;
use greenhouse_core::smart_device_dto::config::ConfigRequestDto;
use greenhouse_core::smart_device_dto::read::ReadResponseDto;
use greenhouse_core::smart_device_dto::write::WriteRequestDto;
use serde::{Serialize, de::DeserializeOwned};

use crate::config::Config;
use crate::device_builder::DeviceState;
use crate::platform::{Arc, ConfigStore, StatusCode};

pub(crate) struct Reply {
    pub status: StatusCode,
    /// Serialized JSON.
    pub body: Option<Vec<u8>>,
}

impl Reply {
    fn status(status: StatusCode) -> Self {
        Self { status, body: None }
    }

    fn json<V: Serialize>(value: &V) -> Self {
        match serde_json::to_vec(value) {
            Ok(body) => Self {
                status: StatusCode::OK,
                body: Some(body),
            },
            Err(e) => {
                log::error!("response serialization failed: {e}");
                Self::status(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

fn parse<V: DeserializeOwned>(body: &[u8]) -> Result<V, Reply> {
    serde_json::from_slice(body).map_err(|e| {
        log::warn!("invalid request body: {e}");
        Reply::status(StatusCode::BAD_REQUEST)
    })
}

pub(crate) async fn read<T: Clone + Default>(state: &DeviceState<T>) -> Reply {
    let Some(handler) = state.read_handler.as_ref() else {
        return Reply::status(StatusCode::NOT_FOUND);
    };
    let data = handler(state.config()).await;
    Reply::json(&ReadResponseDto { data })
}

pub(crate) async fn write<T: Clone + Default>(state: &DeviceState<T>, body: &[u8]) -> Reply {
    let Some(handler) = state.write_handler.as_ref() else {
        return Reply::status(StatusCode::NOT_FOUND);
    };
    match parse::<WriteRequestDto>(body) {
        Ok(payload) => Reply::status(handler(payload.data, state.config()).await),
        Err(reply) => reply,
    }
}

pub(crate) async fn status<T: Clone + Default>(state: &DeviceState<T>) -> Reply {
    Reply::json(&(state.status_handler)(state.config()).await)
}

pub(crate) async fn get_config<T>(state: &DeviceState<T>) -> Reply
where
    T: Clone + Default + Serialize,
{
    // `Option` for compatibility: the response used to be `null` when the
    // config file was unreadable. The config is always in memory now.
    Reply::json(&Some(state.config().to_response_dto(&state.mode)))
}

pub(crate) async fn post_config<T>(state: &DeviceState<T>, body: &[u8]) -> Reply
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    let request = match parse::<ConfigRequestDto<T>>(body) {
        Ok(request) => request,
        Err(reply) => return reply,
    };
    let mut storage = state.storage.lock().await;
    let config = (state.config_interceptor)(request, state.config()).await;
    persist(state, &mut storage, config)
}

pub(crate) async fn activate<T>(state: &DeviceState<T>, body: &[u8]) -> Reply
where
    T: Clone + Default + Serialize,
{
    let request = match parse::<ActivateRequestDto>(body) {
        Ok(request) => request,
        Err(reply) => return reply,
    };
    let mut storage = state.storage.lock().await;
    let config = state.config().activated(request);
    persist(state, &mut storage, config)
}

/// Stores the config, then makes it current, so memory never runs ahead of
/// storage. Callers hold the storage lock for the whole read-modify-write.
fn persist<T>(state: &DeviceState<T>, storage: &mut ConfigStore, config: Config<T>) -> Reply
where
    T: Clone + Default + Serialize,
{
    let Ok(bytes) = serde_json::to_vec(&config) else {
        return Reply::status(StatusCode::INTERNAL_SERVER_ERROR);
    };
    if let Err(e) = storage.store(&bytes) {
        log::error!("failed to persist config: {e}");
        return Reply::status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    state.config.set(Arc::new(config));
    Reply::status(StatusCode::OK)
}
