use axum::extract::FromRef;
use axum::routing::get;
use axum::{Router, middleware::from_extractor_with_state};
use greenhouse_core::{
    auth::{AuthUser, AuthVerifier},
    service_client::{DataStorageServiceClient, DeviceServiceClient, http_client},
};
use reqwest::{
    Method,
    header::{ACCEPT, AUTHORIZATION},
};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub(crate) mod alert;
pub(crate) mod helper;

#[derive(Clone, Deserialize)]
pub struct ServiceAddresses {
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
}

fn default_environment() -> String {
    "development".to_string()
}

#[derive(FromRef, Clone)]
struct AppState {
    data_storage: DataStorageServiceClient,
    verifier: AuthVerifier,
}

impl AppState {
    fn new(addresses: &ServiceAddresses) -> Self {
        // One pooled client shared by every service client.
        let http = http_client();
        Self {
            data_storage: DataStorageServiceClient::new(
                http.clone(),
                &addresses.data_storage_service,
            ),
            verifier: AuthVerifier::DeviceKey(DeviceServiceClient::new(
                http,
                &addresses.device_service,
            )),
        }
    }
}

pub fn app(config: Config) -> Router {
    let state = AppState::new(&config.service_addresses);

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
        .nest("/alert", alert::router::routes(state.clone()))
        .layer(from_extractor_with_state::<AuthUser, _>(state))
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .route("/health", get(|| async {}))
        .layer(TraceLayer::new_for_http())
}
