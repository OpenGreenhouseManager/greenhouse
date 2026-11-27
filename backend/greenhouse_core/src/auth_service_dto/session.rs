//! Session tokens exchanged between `auth_service` and `web_api`.
//!
//! These never reach the browser as JSON: `web_api` puts the access token in
//! the `auth-token` cookie and the refresh token in an HTTP-only cookie.

use serde::{Deserialize, Serialize};

/// Answer to login, register and refresh.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionDto {
    /// Short-lived access JWT (see [`super::user_token::ACCESS_TOKEN_TTL_SECS`]).
    pub token: String,
    pub token_type: String,
    /// Opaque refresh token for this session.
    pub refresh_token: String,
    /// When the refresh session ends, as a Unix timestamp in seconds.
    pub refresh_expires_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshRequestDto {
    pub refresh_token: String,
}

/// Ends session `sid` of `user_name` if it is still the current one.
#[derive(Serialize, Deserialize, Debug)]
pub struct LogoutRequestDto {
    pub user_name: String,
    pub sid: String,
}
