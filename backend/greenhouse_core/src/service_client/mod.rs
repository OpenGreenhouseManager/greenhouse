//! Typed HTTP client for calls between greenhouse services.
//!
//! [`ServiceClient`] wraps one pooled [`reqwest::Client`] and a base URL and
//! handles status checks and error bodies the same way for every call. The
//! typed wrappers ([`AuthServiceClient`], [`DeviceServiceClient`], ...) build
//! on it so callers never deal with URLs or raw responses.

use std::{fmt, sync::Arc, time::Duration};

use axum::http::StatusCode;
use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};

use crate::http_error::{ErrorResponseBody, HttpErrorMapping};

#[cfg(feature = "auth_service_dto")]
mod auth;
#[cfg(feature = "data_storage_service_dto")]
mod data_storage;
#[cfg(feature = "device_service_dto")]
mod device;
#[cfg(feature = "scripting_service_dto")]
mod scripting;
#[cfg(feature = "smart_device_dto")]
mod smart_device;

#[cfg(feature = "auth_service_dto")]
pub use auth::AuthServiceClient;
#[cfg(feature = "data_storage_service_dto")]
pub use data_storage::DataStorageServiceClient;
#[cfg(feature = "device_service_dto")]
pub use device::DeviceServiceClient;
#[cfg(feature = "scripting_service_dto")]
pub use scripting::ScriptingServiceClient;
#[cfg(feature = "smart_device_dto")]
pub use smart_device::SmartDeviceClient;

/// Builds the `reqwest::Client` that should be shared by all service clients
/// of one process. Cloning a `reqwest::Client` shares its connection pool.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failed to build reqwest client")
}

/// Error returned by every service client call.
#[derive(Debug)]
pub enum ClientError {
    /// The request could not be sent or the connection failed.
    Request(reqwest::Error),
    /// The service answered with success, but the body could not be read.
    Decode(reqwest::Error),
    /// The service answered with a non-success status.
    Api { status: StatusCode, message: String },
}

impl ClientError {
    /// Status code of an [`ClientError::Api`] error, if any.
    pub fn api_status(&self) -> Option<StatusCode> {
        match self {
            ClientError::Api { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Request(e) => write!(f, "request to service failed: {e}"),
            ClientError::Decode(e) => write!(f, "could not decode service response: {e}"),
            ClientError::Api { status, message } => {
                write!(f, "service returned {status}: {message}")
            }
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClientError::Request(e) | ClientError::Decode(e) => Some(e),
            ClientError::Api { .. } => None,
        }
    }
}

impl HttpErrorMapping for ClientError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            ClientError::Request(e) => e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            ClientError::Decode(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ClientError::Api { status, .. } => *status,
        }
    }

    fn to_error_message(&self) -> String {
        match self {
            ClientError::Request(e) | ClientError::Decode(e) => e.to_string(),
            ClientError::Api { message, .. } => message.clone(),
        }
    }
}

pub type ClientResult<T> = Result<T, ClientError>;

/// HTTP client bound to one service's base URL.
#[derive(Clone, Debug)]
pub struct ServiceClient {
    http: reqwest::Client,
    base_url: Arc<str>,
}

impl ServiceClient {
    pub fn new(http: reqwest::Client, base_url: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        Self {
            http,
            base_url: base_url.trim_end_matches('/').into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Starts a request to `base_url + path`. `path` should start with `/`
    /// or be empty.
    pub fn request(&self, method: Method, path: &str) -> ServiceRequest {
        let url = format!("{}{}", self.base_url, path);
        ServiceRequest {
            builder: self.http.request(method.clone(), &url),
            method,
            url,
        }
    }

    pub fn get(&self, path: &str) -> ServiceRequest {
        self.request(Method::GET, path)
    }

    pub fn post(&self, path: &str) -> ServiceRequest {
        self.request(Method::POST, path)
    }

    pub fn put(&self, path: &str) -> ServiceRequest {
        self.request(Method::PUT, path)
    }

    pub fn delete(&self, path: &str) -> ServiceRequest {
        self.request(Method::DELETE, path)
    }
}

/// A request being built by [`ServiceClient`].
#[must_use = "a ServiceRequest does nothing until it is sent"]
pub struct ServiceRequest {
    builder: reqwest::RequestBuilder,
    method: Method,
    url: String,
}

impl ServiceRequest {
    pub fn json<T: Serialize + ?Sized>(mut self, body: &T) -> Self {
        self.builder = self.builder.json(body);
        self
    }

    pub fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.builder = self.builder.query(query);
        self
    }

    /// Overrides the client's default timeout for this request.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.timeout(timeout);
        self
    }

    /// Sends the request and decodes a JSON response body.
    pub async fn send_json<R: DeserializeOwned>(self) -> ClientResult<R> {
        let (method, url) = (self.method.clone(), self.url.clone());
        self.send()
            .await?
            .json()
            .await
            .map_err(|e| decode_error(&method, &url, e))
    }

    /// Sends the request and returns the response body as text.
    pub async fn send_text(self) -> ClientResult<String> {
        let (method, url) = (self.method.clone(), self.url.clone());
        self.send()
            .await?
            .text()
            .await
            .map_err(|e| decode_error(&method, &url, e))
    }

    /// Sends the request and ignores the response body.
    pub async fn send_empty(self) -> ClientResult<()> {
        self.send().await.map(drop)
    }

    async fn send(self) -> ClientResult<reqwest::Response> {
        let Self {
            builder,
            method,
            url,
        } = self;

        let resp = builder.send().await.map_err(|e| {
            tracing::error!(%method, %url, error = ?e, "request to service failed");
            ClientError::Request(e)
        })?;

        let status = resp.status();
        if status.is_success() {
            return Ok(resp);
        }

        // Services answer with `ErrorResponseBody`; fall back to the raw body.
        let body = resp.text().await.unwrap_or_default();
        let message = serde_json::from_str::<ErrorResponseBody>(&body)
            .map(|b| b.error)
            .unwrap_or(body);

        tracing::debug!(%method, %url, %status, %message, "service returned an error");
        Err(ClientError::Api { status, message })
    }
}

fn decode_error(method: &Method, url: &str, e: reqwest::Error) -> ClientError {
    tracing::error!(%method, %url, error = ?e, "could not decode service response");
    ClientError::Decode(e)
}
