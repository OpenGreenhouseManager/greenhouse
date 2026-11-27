use auth::guard::require_auth;
use axum::extract::FromRef;
use axum::routing::get;
use axum::{Router, middleware};
use greenhouse_core::{
    auth::{AuthVerifier, JwtVerifier},
    service_client::{
        AuthServiceClient, DataStorageServiceClient, DeviceServiceClient, http_client,
    },
};
use reqwest::{
    Method,
    header::{ACCEPT, AUTHORIZATION},
};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub(crate) mod alert;
pub(crate) mod auth;
pub(crate) mod device;
pub(crate) mod diary;
pub(crate) mod helper;
pub(crate) mod settings;

#[derive(Clone, Deserialize)]
pub struct ServiceAddresses {
    #[serde(rename = "AUTH_SERVICE")]
    pub auth_service: String,
    #[serde(rename = "DEVICE_SERVICE")]
    pub device_service: String,
    #[serde(rename = "DATA_STORAGE_SERVICE")]
    pub data_storage_service: String,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "API_PORT")]
    pub api_port: u32,
    #[serde(rename = "SERVICE_ADDRESSES")]
    pub service_addresses: ServiceAddresses,
    #[serde(rename = "SENTRY_URL")]
    pub sentry_url: String,
    #[serde(rename = "ENVIRONMENT", default = "default_environment")]
    pub environment: String,
    /// PEM file with auth_service's Ed25519 public key. Access tokens are
    /// verified with it, without calling auth_service.
    #[serde(rename = "JWT_PUBLIC_KEY_PATH", default = "default_public_key_path")]
    pub jwt_public_key_path: String,
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_public_key_path() -> String {
    if cfg!(debug_assertions) {
        "api/web/config/jwt_public.pem".to_string()
    } else {
        "config/jwt_public.pem".to_string()
    }
}

#[derive(FromRef, Clone)]
struct AppState {
    auth: AuthServiceClient,
    device: DeviceServiceClient,
    data_storage: DataStorageServiceClient,
    verifier: AuthVerifier,
    jwt: JwtVerifier,
}

impl AppState {
    fn new(config: &Config) -> Self {
        let addresses = &config.service_addresses;
        let jwt = JwtVerifier::from_public_key_file(&config.jwt_public_key_path)
            .unwrap_or_else(|e| panic!("{e}"));
        // One pooled client shared by every service client.
        let http = http_client();
        Self {
            verifier: AuthVerifier::Jwt(jwt.clone()),
            jwt,
            auth: AuthServiceClient::new(http.clone(), &addresses.auth_service),
            device: DeviceServiceClient::new(http.clone(), &addresses.device_service),
            data_storage: DataStorageServiceClient::new(http, &addresses.data_storage_service),
        }
    }
}

pub fn app(config: Config) -> Router {
    let state = AppState::new(&config);

    let cors = CorsLayer::new()
        .allow_headers([AUTHORIZATION, ACCEPT, reqwest::header::CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin([
            "0.0.0.0".parse().unwrap(),
            "http://localhost:4200".parse().unwrap(),
            "https://localhost:5001".parse().unwrap(),
            "https://stag.grn.mauderer.work".parse().unwrap(),
            "https://prod.grn.mauderer.work".parse().unwrap(),
        ]);
    Router::new()
        .nest("/api/settings", settings::router::routes(state.clone()))
        .nest("/api/diary", diary::router::routes(state.clone()))
        .nest("/api/alert", alert::router::routes(state.clone()))
        .nest("/api/device", device::router::routes(state.clone()))
        .nest("/api/user", auth::router::user_routes(state.clone()))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .merge(auth::router::auth_routes(state))
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .route("/health", get(|| async {}))
        .layer(TraceLayer::new_for_http())
}
