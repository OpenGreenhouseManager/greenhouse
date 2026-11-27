//! Request authentication for the public APIs.
//!
//! [`AuthUser`] is an axum extractor that reads the `auth-token` cookie and
//! verifies it with the state's [`AuthVerifier`]. User access tokens are
//! Ed25519 JWTs checked locally by [`JwtVerifier`], so a request costs no call
//! to `auth_service`. Device keys are still checked by `device_service`. The
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

use std::{fmt, marker::PhantomData, path::Path, sync::Arc};

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, errors::ErrorKind};
use tower_cookies::{Cookie, Cookies};

use crate::{
    auth_service_dto::user_token::{AUDIENCE, ISSUER, UserToken},
    service_client::{ClientError, DeviceServiceClient},
};

/// Name of the cookie that carries the access token.
pub const AUTH_COOKIE: &str = "auth-token";

/// Name of the HTTP-only cookie that carries the refresh token.
pub const REFRESH_COOKIE: &str = "refresh-token";

/// Role carried by tokens verified by [`AuthVerifier::DeviceKey`].
pub const SCRIPT_ROLE: &str = "script";

/// Verifies user access tokens with `auth_service`'s Ed25519 public key.
#[derive(Clone)]
pub struct JwtVerifier {
    key: Arc<DecodingKey>,
    validation: Arc<Validation>,
    /// Same checks without `exp`, for logout.
    validation_any_age: Arc<Validation>,
}

impl fmt::Debug for JwtVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtVerifier").finish_non_exhaustive()
    }
}

impl JwtVerifier {
    /// Builds a verifier from a PEM encoded Ed25519 public key (SPKI).
    pub fn from_public_key_pem(pem: &[u8]) -> Result<Self, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[ISSUER]);
        validation.set_audience(&[AUDIENCE]);
        validation.set_required_spec_claims(&["exp", "iss", "aud"]);
        validation.leeway = 30;
        let mut validation_any_age = validation.clone();
        validation_any_age.validate_exp = false;
        validation_any_age.set_required_spec_claims(&["iss", "aud"]);
        Ok(Self {
            key: Arc::new(DecodingKey::from_ed_pem(pem)?),
            validation: Arc::new(validation),
            validation_any_age: Arc::new(validation_any_age),
        })
    }

    /// Reads the public key from `path`.
    pub fn from_public_key_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let pem = std::fs::read(path)
            .map_err(|e| format!("cannot read JWT public key {}: {e}", path.display()))?;
        Self::from_public_key_pem(&pem)
            .map_err(|e| format!("invalid JWT public key {}: {e}", path.display()))
    }

    /// Checks signature, `exp`, `iss` and `aud` and returns the claims.
    pub fn verify(&self, token: &str) -> Result<UserToken, VerifyError> {
        jsonwebtoken::decode::<UserToken>(token, &self.key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                ErrorKind::ExpiredSignature => VerifyError::Expired,
                _ => VerifyError::Invalid,
            })
    }

    /// Like [`Self::verify`] but accepts an expired token. Only for ending
    /// the session the token names, never for granting access.
    pub fn verify_ignoring_expiry(&self, token: &str) -> Result<UserToken, VerifyError> {
        jsonwebtoken::decode::<UserToken>(token, &self.key, &self.validation_any_age)
            .map(|data| data.claims)
            .map_err(|_| VerifyError::Invalid)
    }
}

/// Why a token could not be verified.
#[derive(Debug, PartialEq, Eq)]
pub enum VerifyError {
    /// The token was valid but its `exp` has passed.
    Expired,
    /// The token is forged, malformed or for someone else.
    Invalid,
    /// The verifying service could not be reached or answered garbage.
    Unavailable,
}

/// Identity taken from a verified token.
#[derive(Clone, Debug)]
pub struct Verified {
    pub role: String,
    /// Set for user tokens, `None` for device keys.
    pub user_name: Option<String>,
}

/// Who verifies tokens for an API.
#[derive(Clone, Debug)]
pub enum AuthVerifier {
    /// User access JWTs, verified locally.
    Jwt(JwtVerifier),
    /// Device scripting keys, checked by `device_service`.
    DeviceKey(DeviceServiceClient),
}

impl AuthVerifier {
    /// Verifies `token` and returns who it belongs to.
    pub async fn verify(&self, token: &str) -> Result<Verified, VerifyError> {
        match self {
            AuthVerifier::Jwt(verifier) => {
                let claims = verifier.verify(token)?;
                Ok(Verified {
                    role: claims.role,
                    user_name: Some(claims.user_name),
                })
            }
            AuthVerifier::DeviceKey(client) => match client.check_key(token).await {
                Ok(_) => Ok(Verified {
                    role: SCRIPT_ROLE.to_string(),
                    user_name: None,
                }),
                Err(ClientError::Api { status, .. }) if !status.is_server_error() => {
                    tracing::trace!(%status, "device key rejected");
                    Err(VerifyError::Invalid)
                }
                Err(e) => {
                    tracing::error!(error = %e, "device key verification failed");
                    Err(VerifyError::Unavailable)
                }
            },
        }
    }
}

/// A request whose token was verified.
#[derive(Clone, Debug)]
pub struct AuthUser {
    /// The verified token, for forwarding to services that need it.
    pub token: String,
    pub role: String,
    /// The user the token was issued to. `None` for device keys.
    pub user_name: Option<String>,
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
    /// The access token expired; the client should refresh and retry.
    ExpiredToken,
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
            // 401 without a role message makes the frontend refresh the
            // session and retry once.
            AuthRejection::ExpiredToken => StatusCode::UNAUTHORIZED.into_response(),
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

        let verified = match AuthVerifier::from_ref(state).verify(&token).await {
            Ok(verified) => verified,
            Err(VerifyError::Expired) => {
                tracing::trace!("token expired");
                return Err(AuthRejection::ExpiredToken);
            }
            Err(VerifyError::Invalid) => {
                tracing::trace!("token rejected");
                cookies.remove(Cookie::from(AUTH_COOKIE));
                return Err(AuthRejection::InvalidToken);
            }
            Err(VerifyError::Unavailable) => return Err(AuthRejection::VerifierUnavailable),
        };

        let user = AuthUser {
            token,
            role: verified.role,
            user_name: verified.user_name,
        };
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
