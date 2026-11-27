use serde::{Deserialize, Serialize};

/// `iss` claim of user access tokens.
pub const ISSUER: &str = "greenhouse-auth";
/// `aud` claim of user access tokens.
pub const AUDIENCE: &str = "greenhouse-web";
/// Lifetime of an access token. A replaced session keeps working for at most
/// this long, because only `/refresh` checks the session.
pub const ACCESS_TOKEN_TTL_SECS: i64 = 10 * 60;
/// Lifetime of a refresh session, counted from login.
pub const REFRESH_SESSION_TTL_SECS: i64 = 3 * 60 * 60;

/// Claims of a user access token, signed by `auth_service` with Ed25519.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    pub user_name: String,
    pub role: String,
    /// Session the token was issued for.
    pub sid: String,
    pub iss: String,
    pub aud: String,
}
