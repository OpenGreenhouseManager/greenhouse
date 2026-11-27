use crate::router::auth_router::{
    generate_one_time_token, get_preferences, login, refresh, register, register_admin,
    register_guest, set_preferences,
};
use crate::token::user_token::TokenSigner;

extern crate diesel_migrations;
use axum::extract::FromRef;
use axum::routing::get;
use axum::{Router, routing::post};
use diesel::{Connection, PgConnection};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use greenhouse_core::auth_service_dto::endpoints;
use serde::Deserialize;
use tower_http::trace::TraceLayer;

pub(crate) mod database;
mod router;
pub(crate) mod token;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "SERVICE_PORT")]
    pub service_port: u32,
    #[serde(rename = "DATABASE_URL")]
    pub database_url: String,
    /// Secret for one-time registration tokens.
    #[serde(rename = "JWT_SECRET")]
    pub jwt_secret: String,
    /// PEM file with the Ed25519 private key (PKCS#8) that signs access
    /// tokens. web_api verifies them with the matching public key.
    #[serde(rename = "JWT_PRIVATE_KEY_PATH", default = "default_private_key_path")]
    pub jwt_private_key_path: String,
    /// `kid` header of issued access tokens.
    #[serde(rename = "JWT_KEY_ID", default = "default_key_id")]
    pub jwt_key_id: String,
    #[serde(rename = "SENTRY_URL")]
    pub sentry_url: String,
    #[serde(rename = "ENVIRONMENT", default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_private_key_path() -> String {
    if cfg!(debug_assertions) {
        "services/auth_service/config/jwt_private.pem".to_string()
    } else {
        "config/jwt_private.pem".to_string()
    }
}

fn default_key_id() -> String {
    "1".to_string()
}

pub type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: Pool,
    pub(crate) signer: TokenSigner,
}

pub fn app(config: Config, pool: Pool) -> Router {
    run_migration(&config.database_url);

    let signer = TokenSigner::from_file(&config.jwt_private_key_path, &config.jwt_key_id)
        .unwrap_or_else(|e| {
            panic!(
                "failed to load JWT private key from {}: {e}",
                config.jwt_private_key_path
            )
        });
    let state = AppState {
        config,
        pool,
        signer,
    };

    Router::new()
        .route(endpoints::REGISTER, post(register))
        .route(endpoints::LOGIN, post(login))
        .route(endpoints::REFRESH, post(refresh))
        .route(endpoints::ADMIN_REGISTER, post(register_admin))
        .route(endpoints::GUEST_REGISTER, post(register_guest))
        .route(
            &format!("{}/get", endpoints::PREFERENCES),
            post(get_preferences),
        )
        .route(
            &format!("{}/set", endpoints::PREFERENCES),
            post(set_preferences),
        )
        .route(
            endpoints::GENERATE_ONE_TIME_TOKEN,
            post(generate_one_time_token),
        )
        .route("/health", get(|| async {}))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn run_migration(database_url: &str) {
    let mut conn = PgConnection::establish(database_url).unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
