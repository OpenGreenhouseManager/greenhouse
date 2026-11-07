use crate::{AppState, alert::service, helper::error::HttpResult};
use axum::{Json, Router, extract::State, routing::post};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    endpoints::STOP, post_create_alert::CreateAlertDto, post_stop_alert::StopAlertDto,
};
use reqwest::StatusCode;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_alert))
        .route(STOP, post(stop_alert))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn create_alert(
    State(AppState { config }): State<AppState>,
    Json(alert): Json<CreateAlertDto>,
) -> HttpResult<StatusCode> {
    service::create_alert(&config.service_addresses.data_storage_service, alert).await?;
    Ok(StatusCode::OK)
}

pub(crate) async fn stop_alert(
    State(AppState { config }): State<AppState>,
    Json(stop): Json<StopAlertDto>,
) -> HttpResult<StatusCode> {
    service::stop_alert(&config.service_addresses.data_storage_service, stop).await?;
    Ok(StatusCode::OK)
}
