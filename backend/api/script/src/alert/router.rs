use crate::{AppState, helper::error::HttpResult};
use axum::{Json, Router, extract::State, routing::post};
use greenhouse_core::{
    data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto,
    service_client::DataStorageServiceClient,
};
use reqwest::StatusCode;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_alert))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn create_alert(
    State(data_storage): State<DataStorageServiceClient>,
    Json(alert): Json<CreateAlertDto>,
) -> HttpResult<StatusCode> {
    data_storage.create_alert(&alert).await?;
    Ok(StatusCode::OK)
}
