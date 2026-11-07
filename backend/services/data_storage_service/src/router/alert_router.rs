use super::error::HttpResult;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::{AlertDto, AlertsDto},
    endpoints::STOP,
    get_aggrigated_alert::{AggrigatedAlertDto, AggrigatedAlertsDto},
    post_create_alert::CreateAlertDto,
    post_stop_alert::StopAlertDto,
    query::{AlertQuery, IntervalQuery},
};

use crate::{AppState, database::alert_models::Alert};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(fire_alert))
        .route(STOP, post(stop_alert))
        .route("/", get(alert_subset))
        .route("/filter", get(filter))
        .with_state(state)
}

async fn filter(
    State(AppState { config: _, pool }): State<AppState>,
    Query(query): Query<AlertQuery>,
) -> HttpResult<AlertsDto> {
    let a = Alert::query(query, &pool)
        .await?
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<AlertDto>>();
    Ok(a.into())
}

async fn alert_subset(
    State(AppState { config: _, pool }): State<AppState>,
    Query(query): Query<IntervalQuery>,
) -> HttpResult<AggrigatedAlertsDto> {
    let a = Alert::aggrigate(query, &pool)
        .await?
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<AggrigatedAlertDto>>()
        .into();
    Ok(a)
}

async fn fire_alert(
    State(AppState { config: _, pool }): State<AppState>,
    Json(alert): Json<CreateAlertDto>,
) -> HttpResult<AlertDto> {
    let alert = Alert::fire(alert, &pool).await?.into();
    Ok(alert)
}

/// Responds with the stopped alert, or `204 No Content` if none was open.
async fn stop_alert(
    State(AppState { config: _, pool }): State<AppState>,
    Json(stop): Json<StopAlertDto>,
) -> HttpResult<Response> {
    let response = match Alert::stop(stop, &pool).await? {
        Some(alert) => AlertDto::from(alert).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    };
    Ok(response)
}
