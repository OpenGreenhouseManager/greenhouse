use super::{Error, HttpResult};
use crate::database::schema::preferences::{alert_preferences, dashboard_preferences, user_id};
use crate::{
    AppState,
    database::{self, models::Preferences, models::User},
};
use crate::{
    Pool, database::schema::preferences::dsl::preferences, database::schema::users::dsl::users,
    token::one_time_token::check_one_time_token, token::session::hash_refresh_token,
    token::user_token::TokenSigner,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State, response::Response};
use database::schema::users::{id, login_session, session_expires_at, session_id, username};
use diesel::{ExpressionMethods, query_dsl::methods::FilterDsl};
use diesel_async::RunQueryDsl;
use greenhouse_core::auth_service_dto::generate_one_time_token::{
    GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto,
};
use greenhouse_core::auth_service_dto::user_preferences::{
    GetPreferencesRequestDto, SetPreferencesRequestDto, UserPreferencesResponseDto,
};
use greenhouse_core::auth_service_dto::{
    login::LoginRequestDto,
    register::RegisterRequestDto,
    register_admin::RegisterAdminRequestDto,
    session::{LogoutRequestDto, RefreshRequestDto, SessionDto},
};

#[axum::debug_handler]
pub(crate) async fn register(
    State(AppState {
        config,
        pool,
        signer,
    }): State<AppState>,
    Json(user): Json<RegisterRequestDto>,
) -> HttpResult<Json<SessionDto>> {
    let role = "user";
    check_one_time_token(
        &user.username,
        user.one_time_token
            .parse::<u64>()
            .map_err(|_| Error::OneTimeToken)?,
        &config.jwt_secret,
    )?;
    let session = register_user(&user.username, &user.password, role, &signer, &pool).await?;
    Ok(Json(session))
}

#[axum::debug_handler]
pub(crate) async fn register_admin(
    State(AppState { pool, signer, .. }): State<AppState>,
    Json(user): Json<RegisterAdminRequestDto>,
) -> HttpResult<Json<SessionDto>> {
    let role = "admin";
    let session = register_user(&user.username, &user.password, role, &signer, &pool).await?;
    Ok(Json(session))
}

#[axum::debug_handler]
pub(crate) async fn register_guest(
    State(AppState { pool, signer, .. }): State<AppState>,
    Json(user): Json<RegisterAdminRequestDto>,
) -> HttpResult<Json<SessionDto>> {
    let role = "guest";
    let session = register_user(&user.username, &user.password, role, &signer, &pool).await?;
    Ok(Json(session))
}

#[axum::debug_handler]
pub(crate) async fn generate_one_time_token(
    State(AppState { config, .. }): State<AppState>,
    Json(user): Json<GenerateOneTimeTokenRequestDto>,
) -> HttpResult<Response> {
    let token =
        crate::token::one_time_token::generate_one_time_token(&user.username, &config.jwt_secret);
    Ok(Json(GenerateOneTimeTokenResponseDto {
        token: token.to_string(),
    })
    .into_response())
}

pub(crate) async fn register_user(
    name: &str,
    password: &str,
    role: &str,
    signer: &TokenSigner,
    pool: &Pool,
) -> HttpResult<SessionDto> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let mut new_user = User::new(name, password, role)?;
    let session = new_user.start_session(signer)?;
    let _ = diesel::insert_into(database::schema::users::table)
        .values(new_user)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(String::from(name)),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), name.into());
                map.insert(String::from("role"), role.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::UsernameTaken
        })?;
    Ok(session)
}

#[axum::debug_handler]
pub(crate) async fn login(
    State(AppState {
        config,
        pool,
        signer,
    }): State<AppState>,
    Json(login): Json<LoginRequestDto>,
) -> HttpResult<Json<SessionDto>> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::configure_scope(|scope| {
            let mut map = std::collections::BTreeMap::new();
            map.insert(String::from("db_url"), config.database_url.into());

            scope.set_context("db", sentry::protocol::Context::Other(map));
        });
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let mut user = users
        .filter(username.eq(&login.username))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(login.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), login.username.clone().into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            match e {
                diesel::result::Error::NotFound => Error::UserNotFound,
                _ => Error::DatabaseConnection,
            }
        })?;

    if !user.check_login(&login.password).await? {
        return Err(Error::PasswordIncorrect.into());
    }

    // Replaces the previous session: its refresh token stops working now and
    // its access token at the latest when it expires.
    let session = user.start_session(&signer)?;

    diesel::update(users)
        .filter(id.eq(user.id))
        .set((
            login_session.eq(&user.login_session),
            session_id.eq(user.session_id),
            session_expires_at.eq(user.session_expires_at),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(login.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), login.username.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::UserNotFound
        })?;

    Ok(Json(session))
}

/// Issues a new access token for a live refresh session. This is where a
/// session replaced by a newer login ends.
#[axum::debug_handler]
pub(crate) async fn refresh(
    State(AppState { pool, signer, .. }): State<AppState>,
    Json(request): Json<RefreshRequestDto>,
) -> HttpResult<Json<SessionDto>> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    let user = users
        .filter(login_session.eq(hash_refresh_token(&request.refresh_token)))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::SessionInvalid,
            e => {
                sentry::capture_error(&e);
                Error::DatabaseConnection
            }
        })?;

    if !user.session_is_valid(&request.refresh_token) {
        return Err(Error::SessionInvalid.into());
    }
    let (Some(sid), Some(expires_at)) = (user.session_id, user.session_expires_at) else {
        return Err(Error::SessionInvalid.into());
    };

    Ok(Json(SessionDto {
        token: signer.issue(&user.username, &user.role, &sid.to_string())?,
        token_type: String::from("Bearer"),
        refresh_token: request.refresh_token,
        refresh_expires_at: expires_at.timestamp(),
    }))
}

/// Ends the session of a refresh token so it can no longer be refreshed.
/// Does nothing if the session is already gone.
#[axum::debug_handler]
pub(crate) async fn logout(
    State(AppState { pool, .. }): State<AppState>,
    Json(request): Json<LogoutRequestDto>,
) -> HttpResult<StatusCode> {
    let mut conn = pool.get().await.map_err(|e| {
        sentry::capture_error(&e);
        Error::DatabaseConnection
    })?;

    diesel::update(users)
        .filter(login_session.eq(hash_refresh_token(&request.refresh_token)))
        .set((
            login_session.eq(""),
            session_id.eq(None::<uuid::Uuid>),
            session_expires_at.eq(None::<chrono::DateTime<chrono::Utc>>),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_error(&e);
            Error::DatabaseConnection
        })?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler]
pub(crate) async fn get_preferences(
    State(AppState { pool, .. }): State<AppState>,
    Json(request): Json<GetPreferencesRequestDto>,
) -> HttpResult<UserPreferencesResponseDto> {
    let mut conn = pool.get().await.map_err(|_| Error::DatabaseConnection)?;

    // web_api verified the access token and passes its user.
    let user = users
        .filter(username.eq(&request.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::UserNotFound)?;

    let pref = preferences
        .filter(user_id.eq(&user.id))
        .get_result::<Preferences>(&mut conn)
        .await
        .map_err(|_| Error::DatabaseConnection)?;
    Ok(UserPreferencesResponseDto {
        dashboard_preferences: pref.dashboard_preferences,
        alert_preferences: pref.alert_preferences,
    })
}

#[axum::debug_handler]
pub(crate) async fn set_preferences(
    State(AppState { pool, .. }): State<AppState>,
    Json(request): Json<SetPreferencesRequestDto>,
) -> HttpResult<UserPreferencesResponseDto> {
    let mut conn = pool.get().await.map_err(|e| {
        tracing::error!("Error getting database connection: {:?}", e);
        Error::DatabaseConnection
    })?;

    // web_api verified the access token and passes its user.
    let user = users
        .filter(username.eq(&request.user_name))
        .get_result::<User>(&mut conn)
        .await
        .map_err(|_| Error::UserNotFound)?;

    diesel::insert_into(preferences)
        .values(Preferences::new(
            user.id,
            request.preferences.dashboard_preferences.clone(),
            request.preferences.alert_preferences.clone(),
        ))
        .on_conflict(user_id)
        .do_update()
        .set((
            dashboard_preferences.eq(request.preferences.dashboard_preferences.clone()),
            alert_preferences.eq(request.preferences.alert_preferences.clone()),
        ))
        .execute(&mut conn)
        .await
        .map_err(|e: diesel::result::Error| {
            tracing::error!("Error setting preferences: {:?}", e);
            Error::DatabaseConnection
        })?;

    Ok(UserPreferencesResponseDto {
        dashboard_preferences: request.preferences.dashboard_preferences,
        alert_preferences: request.preferences.alert_preferences,
    })
}
