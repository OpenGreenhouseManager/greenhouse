use crate::{AppState, helper::error::HttpResult};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderValue,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use greenhouse_core::device_service_dto::{
    batch_timeseries::{BatchTimeseriesRequestDto, BatchTimeseriesResponseDto},
    endpoints::{ACTIVATE, CONFIG, STATUS},
    get_device::{DeviceResponseDto, DevicesResponseDto},
    get_timeseries::GetTimeseriesDto,
    operations::OperationsDto,
    post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
    query::PromQuery,
};
use greenhouse_core::service_client::DeviceServiceClient;
use reqwest::{StatusCode, header};
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_device))
        .route("/", get(get_devices))
        .route("/{id}", put(update_device))
        .route("/{id}", get(get_device))
        .route("/{id}", delete(delete_device))
        .route(&format!("/{{id}}/{ACTIVATE}"), put(activate_device))
        .route(&format!("/{{id}}/{CONFIG}"), put(update_device_config))
        .route(&format!("/{{id}}/{CONFIG}"), get(get_device_config))
        .route(&format!("/{{id}}/{STATUS}"), get(get_device_status))
        .route("/timeseries/batch", post(get_batch_timeseries))
        .route("/{id}/timeseries", get(get_device_timeseries))
        .route("/{id}/options", get(get_device_operations))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn create_device(
    State(device): State<DeviceServiceClient>,
    Json(entry): Json<PostDeviceDtoRequest>,
) -> HttpResult<DeviceResponseDto> {
    Ok(device.create_device(&entry).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_devices(
    State(device): State<DeviceServiceClient>,
) -> HttpResult<DevicesResponseDto> {
    Ok(device.get_devices().await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn update_device(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDeviceDtoRequest>,
) -> HttpResult<DeviceResponseDto> {
    Ok(device.update_device(id, &update).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_device(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<DeviceResponseDto> {
    Ok(device.get_device(id).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn delete_device(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<StatusCode> {
    device.delete_device(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_device_config(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let response = device.get_device_config(id).await?;
    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        response,
    ))
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn update_device_config(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> HttpResult<StatusCode> {
    device.update_device_config(id, &body).await?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_device_status(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<impl IntoResponse> {
    let response = device.get_device_status(id).await?;
    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        response,
    ))
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn activate_device(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<StatusCode> {
    device.activate_device(id).await?;
    Ok(StatusCode::OK)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_device_timeseries(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
    Query(query): Query<PromQuery>,
) -> HttpResult<GetTimeseriesDto> {
    Ok(device.get_device_timeseries(id, &query).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_batch_timeseries(
    State(device): State<DeviceServiceClient>,
    Json(request): Json<BatchTimeseriesRequestDto>,
) -> HttpResult<BatchTimeseriesResponseDto> {
    Ok(device.get_batch_timeseries(&request).await?)
}

pub(crate) async fn get_device_operations(
    State(device): State<DeviceServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<OperationsDto> {
    Ok(device.get_device_operations(id).await?)
}
