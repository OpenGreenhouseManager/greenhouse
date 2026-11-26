use crate::AppState;
use crate::helper::error::HttpResult;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use greenhouse_core::data_storage_service_dto::alert_dto::alert::AlertsDto;
use greenhouse_core::data_storage_service_dto::alert_dto::get_aggrigated_alert::AggrigatedAlertsDto;
use greenhouse_core::data_storage_service_dto::alert_dto::query::{AlertQuery, IntervalQuery};
use greenhouse_core::service_client::DataStorageServiceClient;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(data_storage): State<DataStorageServiceClient>,
    Query(query): Query<AlertQuery>,
) -> HttpResult<AlertsDto> {
    Ok(data_storage.get_filtered_alerts(&query).await?)
}

async fn alert_subset(
    State(data_storage): State<DataStorageServiceClient>,
    Query(query): Query<IntervalQuery>,
) -> HttpResult<AggrigatedAlertsDto> {
    Ok(data_storage.get_alert_subset(&query).await?)
}
