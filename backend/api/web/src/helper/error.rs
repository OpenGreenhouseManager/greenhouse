use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use greenhouse_core::{
    http_error::{HttpErrorMapping, HttpErrorResponse},
    service_client::ClientError,
};

pub(crate) type HttpResult<T> = core::result::Result<T, Error>;

/// Error returned by web_api handlers: a failed call to a backing service.
#[derive(Debug)]
pub(crate) struct Error(ClientError);

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        match &e {
            ClientError::Api { status, .. } if !status.is_server_error() => {}
            _ => {
                sentry::capture_error(&e);
            }
        }
        Self(e)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        HttpErrorResponse::new(self).into_response()
    }
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl HttpErrorMapping for Error {
    fn to_status_code(&self) -> StatusCode {
        self.0.to_status_code()
    }

    fn to_error_message(&self) -> String {
        self.0.to_error_message()
    }
}
// endregion: --- Error Boilerplate
