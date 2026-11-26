//! Request authentication for the public APIs.
//!
//! [`AuthUser`] is an axum extractor that reads the `auth-token` cookie and
//! verifies it with the service named by the state's [`AuthVerifier`]. The
//! result is cached in the request extensions, so a router-wide
//! `from_extractor` layer and a handler argument cost one verification.
//!
//! ```ignore
//! Router::new()
//!     .route("/admin_only", post(handler))
//!     .route_layer(axum::middleware::from_extractor_with_state::<AuthUser, _>(state))
//!
//! async fn handler(_: RequireRole<Admin>, user: AuthUser) { /* ... */ }
//! ```

use std::marker::PhantomData;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use tower_cookies::{Cookie, Cookies};

use crate::service_client::{AuthServiceClient, ClientError, ScriptingServiceClient};

/// Name of the cookie that carries the token.
pub const AUTH_COOKIE: &str = "auth-token";

/// Role carried by tokens verified by [`AuthVerifier::ScriptingService`].
pub const SCRIPT_ROLE: &str = "script";

/// Who verifies tokens for an API.
#[derive(Clone, Debug)]
pub enum AuthVerifier {
    /// User JWTs, checked by `auth_service`.
    AuthService(AuthServiceClient),
    /// Scripting tokens, checked by `scripting_service`.
    ScriptingService(ScriptingServiceClient),
}

impl AuthVerifier {
    /// Verifies `token` and returns its role.
    pub async fn verify(&self, token: &str) -> Result<String, ClientError> {
        match self {
            AuthVerifier::AuthService(client) => Ok(client.check_token(token).await?.role),
            AuthVerifier::ScriptingService(client) => {
                client.check_token(token).await?;
                Ok(SCRIPT_ROLE.to_string())
            }
        }
    }
}

/// A request whose token was verified.
#[derive(Clone, Debug)]
pub struct AuthUser {
    /// The verified token, for forwarding to services that need it.
    pub token: String,
    pub role: String,
}

impl AuthUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.role == role
    }
}

/// Why a request was not authenticated.
#[derive(Debug)]
pub enum AuthRejection {
    /// No `auth-token` cookie.
    MissingToken,
    /// The verifier rejected the token.
    InvalidToken,
    /// The token is valid but lacks the required role.
    MissingRole(&'static str),
    /// The verifying service could not be reached or answered garbage.
    VerifierUnavailable,
    /// `CookieManagerLayer` is not installed on the router.
    MissingCookieLayer,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        // 403 makes the frontend drop the cookie and go to the login page,
        // so only use it when the token itself is the problem.
        match self {
            AuthRejection::MissingToken | AuthRejection::InvalidToken => {
                StatusCode::FORBIDDEN.into_response()
            }
            AuthRejection::MissingRole(role) => {
                (StatusCode::UNAUTHORIZED, format!("requires role {role}")).into_response()
            }
            AuthRejection::VerifierUnavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
            AuthRejection::MissingCookieLayer => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    AuthVerifier: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<AuthUser>() {
            return Ok(user.clone());
        }

        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthRejection::MissingCookieLayer)?;

        let Some(token) = cookies.get(AUTH_COOKIE).map(|c| c.value().to_string()) else {
            tracing::trace!("no auth cookie");
            return Err(AuthRejection::MissingToken);
        };

        let role = match AuthVerifier::from_ref(state).verify(&token).await {
            Ok(role) => role,
            Err(ClientError::Api { status, .. }) if !status.is_server_error() => {
                tracing::trace!(%status, "token rejected");
                cookies.remove(Cookie::from(AUTH_COOKIE));
                return Err(AuthRejection::InvalidToken);
            }
            Err(e) => {
                tracing::error!(error = %e, "token verification failed");
                return Err(AuthRejection::VerifierUnavailable);
            }
        };

        let user = AuthUser { token, role };
        parts.extensions.insert(user.clone());
        Ok(user)
    }
}

/// A role that [`RequireRole`] can check for.
pub trait Role {
    const NAME: &'static str;
}

pub struct Admin;
impl Role for Admin {
    const NAME: &'static str = "admin";
}

pub struct Guest;
impl Role for Guest {
    const NAME: &'static str = "guest";
}

/// Extractor that rejects requests whose [`AuthUser`] lacks role `R`.
pub struct RequireRole<R: Role>(pub AuthUser, PhantomData<R>);

impl<R: Role> RequireRole<R> {
    pub fn into_inner(self) -> AuthUser {
        self.0
    }
}

impl<S, R> FromRequestParts<S> for RequireRole<R>
where
    AuthVerifier: FromRef<S>,
    S: Send + Sync,
    R: Role,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if !user.has_role(R::NAME) {
            return Err(AuthRejection::MissingRole(R::NAME));
        }
        Ok(RequireRole(user, PhantomData))
    }
}
