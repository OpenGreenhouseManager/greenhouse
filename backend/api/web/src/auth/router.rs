use crate::{AppState, helper::error::HttpResult};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use greenhouse_core::{
    auth::{AuthUser, JwtVerifier, REFRESH_COOKIE},
    auth_service_dto::{
        login::{LoginRequestDto, LoginResponseDto},
        register::RegisterRequestDto,
        session::SessionDto,
        user_preferences::{UserPreferencesRequestDto, UserPreferencesResponseDto},
    },
    service_client::{AuthServiceClient, ClientError},
};
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

use super::AUTH_TOKEN;

/// Path the refresh cookie is scoped to, so it is only sent to the refresh
/// endpoint and not with every request.
pub(crate) const REFRESH_PATH: &str = "/api/refresh";

pub(crate) fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/register", post(api_register_handler))
        .route("/api/login", post(api_login_handler))
        .route(REFRESH_PATH, post(api_refresh_handler))
        .route("/api/logout", post(api_logout_handler))
        .with_state(state)
}

pub(crate) fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/preferences", get(api_get_user_preferences_handler))
        .route("/preferences", post(api_set_user_preferences_handler))
        .with_state(state)
}

fn auth_cookie(value: String) -> Cookie<'static> {
    let mut c = Cookie::new(AUTH_TOKEN, value);
    c.set_same_site(SameSite::None);
    // The frontend reads the role from it.
    c.set_http_only(false);
    c.set_secure(true);
    c.set_path("/");
    c
}

fn refresh_cookie(value: String) -> Cookie<'static> {
    let mut c = Cookie::new(REFRESH_COOKIE, value);
    c.set_same_site(SameSite::None);
    c.set_http_only(true);
    c.set_secure(true);
    c.set_path(REFRESH_PATH);
    c
}

/// Puts the access token and the refresh token of `session` into cookies
/// and returns the body the frontend expects.
fn start_session(cookies: &Cookies, session: SessionDto) -> Json<LoginResponseDto> {
    cookies.add(auth_cookie(session.token.clone()));

    let mut refresh = refresh_cookie(session.refresh_token);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default();
    refresh.set_max_age(Duration::seconds((session.refresh_expires_at - now).max(0)));
    cookies.add(refresh);

    Json(LoginResponseDto {
        token: session.token,
        token_type: session.token_type,
    })
}

/// Expires both cookies. `Cookies::remove` only emits a removal for cookies
/// the request carried, and the refresh cookie is not sent outside
/// `/api/refresh`, so the removals are added explicitly.
fn end_session(cookies: &Cookies) {
    for mut cookie in [auth_cookie(String::new()), refresh_cookie(String::new())] {
        cookie.make_removal();
        cookies.add(cookie);
    }
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn api_register_handler(
    State(auth): State<AuthServiceClient>,
    cookies: Cookies,
    Json(register_request): Json<RegisterRequestDto>,
) -> HttpResult<Response> {
    tracing::trace!(
        "Registering user with username: {}",
        register_request.username,
    );
    match auth.register(&register_request).await {
        Ok(session) => Ok(start_session(&cookies, session).into_response()),
        Err(e) => {
            end_session(&cookies);
            tracing::error!("Error during registration: {:?}", e);
            Err(e.into())
        }
    }
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn api_login_handler(
    State(auth): State<AuthServiceClient>,
    cookies: Cookies,
    Json(login_request): Json<LoginRequestDto>,
) -> HttpResult<Response> {
    tracing::trace!("Logging in user with username: {}", login_request.username,);
    match auth.login(&login_request).await {
        Ok(session) => Ok(start_session(&cookies, session).into_response()),
        Err(e) => {
            end_session(&cookies);
            tracing::error!("Error during login: {:?}", e);
            Err(e.into())
        }
    }
}

/// Swaps the refresh cookie for a new access token. Answers 403 when the
/// session expired or a newer login replaced it, so the frontend logs out.
#[axum::debug_handler(state = AppState)]
pub(crate) async fn api_refresh_handler(
    State(auth): State<AuthServiceClient>,
    cookies: Cookies,
) -> HttpResult<Response> {
    let Some(refresh_token) = cookies.get(REFRESH_COOKIE).map(|c| c.value().to_string()) else {
        end_session(&cookies);
        return Ok(StatusCode::FORBIDDEN.into_response());
    };
    match auth.refresh(&refresh_token).await {
        Ok(session) => Ok(start_session(&cookies, session).into_response()),
        Err(ClientError::Api { status, .. }) if !status.is_server_error() => {
            tracing::trace!(%status, "refresh rejected");
            end_session(&cookies);
            Ok(StatusCode::FORBIDDEN.into_response())
        }
        Err(e) => {
            tracing::error!("Error during refresh: {:?}", e);
            Err(e.into())
        }
    }
}

/// Revokes the session named by the access token (expired or not) and
/// clears both cookies, so neither this browser nor a refresh that was still
/// in flight can continue it.
#[axum::debug_handler(state = AppState)]
pub(crate) async fn api_logout_handler(
    State(auth): State<AuthServiceClient>,
    State(jwt): State<JwtVerifier>,
    cookies: Cookies,
) -> HttpResult<StatusCode> {
    let claims = cookies
        .get(AUTH_TOKEN)
        .and_then(|c| jwt.verify_ignoring_expiry(c.value()).ok());
    if let Some(claims) = claims
        && let Err(e) = auth.logout(&claims.user_name, &claims.sid).await
    {
        tracing::error!("Error during logout: {:?}", e);
        return Err(e.into());
    }
    end_session(&cookies);
    Ok(StatusCode::OK)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn api_get_user_preferences_handler(
    State(auth): State<AuthServiceClient>,
    user: AuthUser,
) -> HttpResult<UserPreferencesResponseDto> {
    Ok(auth.get_preferences(user_name(&user)).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn api_set_user_preferences_handler(
    State(auth): State<AuthServiceClient>,
    user: AuthUser,
    Json(user_preferences): Json<UserPreferencesRequestDto>,
) -> HttpResult<UserPreferencesResponseDto> {
    Ok(auth
        .set_preferences(user_name(&user), user_preferences)
        .await?)
}

/// web_api only accepts user JWTs, which always name their user.
fn user_name(user: &AuthUser) -> &str {
    user.user_name.as_deref().unwrap_or_default()
}
