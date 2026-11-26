use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tower_cookies::{Cookie, Cookies};

use super::{AUTH_TOKEN, service};
use crate::{AppState, auth::Error};

pub(crate) async fn check_token(
    State(AppState { config }): State<AppState>,
    cookies: Cookies,
    req: Request<Body>,
    next: Next,
) -> Response {
    let Some(token) = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string()) else {
        tracing::trace!("Missing Cookie");
        return status_response(StatusCode::FORBIDDEN);
    };

    match service::check_token(&config.service_addresses.scripting_service, &token).await {
        Ok(()) => next.run(req).await,
        // Only drop the cookie when the scripting service rejected the token itself.
        Err(Error::Api(e)) if e.status == StatusCode::UNAUTHORIZED => {
            cookies.remove(Cookie::from(AUTH_TOKEN));
            tracing::trace!("Invalid Cookie");
            status_response(StatusCode::FORBIDDEN)
        }
        // The token could not be checked (e.g. database or network failure): reject the
        // request but keep the cookie so a transient error does not log the device out.
        Err(e) => {
            tracing::error!("Could not check token: {e:?}");
            status_response(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

fn status_response(status: StatusCode) -> Response {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap()
}
