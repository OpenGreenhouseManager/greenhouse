//! axum front end for [`crate::handler`].

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use greenhouse_core::smart_device_dto::endpoints::{ACTIVATE, CONFIG, READ, STATUS, WRITE};
use serde::{Serialize, de::DeserializeOwned};

use super::{Arc, Server};
use crate::device_builder::{Device, DeviceState};
use crate::handler::{self, Reply};

impl IntoResponse for Reply {
    fn into_response(self) -> Response {
        match self.body {
            Some(body) => (self.status, [(CONTENT_TYPE, "application/json")], body).into_response(),
            None => self.status.into_response(),
        }
    }
}

impl<T> Device<T>
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// The device API as an axum [`Router`], for serving it yourself (e.g. next
    /// to other routes). [`Device::run`] does this on `0.0.0.0:{port}`.
    pub fn into_router(self) -> Router {
        router(self.state)
    }
}

fn router<T>(state: Arc<DeviceState<T>>) -> Router
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    type S<T> = State<Arc<DeviceState<T>>>;
    Router::new()
        .route(
            READ,
            get(|State(s): S<T>| async move { handler::read(&s).await }),
        )
        .route(
            WRITE,
            post(|State(s): S<T>, body: Bytes| async move { handler::write(&s, &body).await }),
        )
        .route(
            CONFIG,
            get(|State(s): S<T>| async move { handler::get_config(&s).await }).post(
                |State(s): S<T>, body: Bytes| async move { handler::post_config(&s, &body).await },
            ),
        )
        .route(
            STATUS,
            get(|State(s): S<T>| async move { handler::status(&s).await }),
        )
        .route(
            ACTIVATE,
            post(|State(s): S<T>, body: Bytes| async move { handler::activate(&s, &body).await }),
        )
        .with_state(state)
}

pub(crate) async fn serve<T>(state: Arc<DeviceState<T>>, _: Server) -> !
where
    T: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    let port = state.config().port;
    let listener = match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
        Ok(listener) => listener,
        Err(e) => panic!("cannot bind the device API to port {port}: {e}"),
    };
    log::info!("serving device API on port {port}");
    let result = axum::serve(listener, router(state)).await;
    panic!("device API server stopped: {result:?}")
}
