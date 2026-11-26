use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use greenhouse_core::auth::{AuthUser, Guest, Role};

/// Requires a verified token for every route behind it.
///
/// Guests are read-only: their writes are acknowledged with an empty 200
/// and never reach a service, so the frontend's guest mode keeps working.
pub(crate) async fn require_auth(user: AuthUser, req: Request<Body>, next: Next) -> Response {
    let is_write = matches!(*req.method(), Method::POST | Method::PUT | Method::DELETE);
    if user.has_role(Guest::NAME) && is_write {
        return StatusCode::OK.into_response();
    }
    next.run(req).await
}
