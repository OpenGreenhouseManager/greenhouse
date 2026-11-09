use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    smart_device_dto::{
        Type,
        activation::ActivateRequestDto,
        config::{ConfigRequestDto, ConfigResponseDto},
        read::ReadResponseDto,
        status::DeviceStatusResponseDto,
        write::WriteRequestDto,
    },
    smart_device_interface::config::Config,
};

use super::{
    config::{read_config_file_with_path, update_config_file_with_path},
    device_builder::DeviceBuilder,
};

fn current_config<T: Clone + Default>(device_service: &DeviceBuilder<T>) -> Arc<Config<T>> {
    device_service
        .config
        .read()
        .map(|c| c.clone())
        .unwrap_or_else(|_| Arc::new(Config::<T>::default()))
}

pub(crate) async fn write_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(payload): Json<WriteRequestDto>,
) -> StatusCode
where
    T: Clone + Default + DeserializeOwned,
{
    let config = current_config(&device_service);

    match device_service.write_handler {
        Some(handler) => handler(payload.data, config).await,
        None => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn read_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<ReadResponseDto>
where
    T: Clone + Default + DeserializeOwned,
{
    let config = current_config(&device_service);

    match device_service.read_handler {
        None => Json(ReadResponseDto { data: Type::None }),
        Some(handler) => {
            let data = handler(config).await;
            Json(ReadResponseDto { data })
        }
    }
}

pub(crate) async fn get_config_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<Option<ConfigResponseDto<T>>>
where
    T: DeserializeOwned + Clone + Default,
{
    // This refreshes the in-memory config from the file, so it must not run
    // between another update's file write and memory swap.
    let _update = device_service.config_update_lock.lock().await;
    match read_config_file_with_path::<T>(&device_service.config_path) {
        Ok(config) => {
            let response = config.to_response_dto(&device_service.mode);
            if let Ok(mut guard) = device_service.config.write() {
                *guard = Arc::new(config);
            }
            Json(Some(response))
        }
        Err(_) => Json(None),
    }
}

pub(crate) async fn status_device_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
) -> Json<DeviceStatusResponseDto>
where
    T: Clone + Default + DeserializeOwned,
{
    let config = current_config(&device_service);

    Json((device_service.status_handler)(config).await)
}

pub(crate) async fn config_update_handler<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(config): Json<ConfigRequestDto<T>>,
) -> StatusCode
where
    T: Serialize + Clone + Default,
{
    let _update = device_service.config_update_lock.lock().await;
    let config =
        (device_service.config_interceptor_handler)(config, current_config(&device_service)).await;

    persist(&device_service, config)
}

pub(crate) async fn activate_device<T>(
    State(device_service): State<DeviceBuilder<T>>,
    Json(request): Json<ActivateRequestDto>,
) -> StatusCode
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    let _update = device_service.config_update_lock.lock().await;
    let base_config: Arc<Config<T>> = device_service
        .config
        .read()
        .ok()
        .map(|c| c.clone())
        .unwrap_or_else(|| {
            Arc::new(read_config_file_with_path(&device_service.config_path).unwrap_or_default())
        });

    persist(&device_service, base_config.activated(request))
}

/// Writes the config file, then makes the config current.
fn persist<T>(device_service: &DeviceBuilder<T>, config: Config<T>) -> StatusCode
where
    T: Serialize + Clone + Default,
{
    if update_config_file_with_path(&config, &device_service.config_path).is_ok() {
        if let Ok(mut guard) = device_service.config.write() {
            *guard = Arc::new(config);
        }
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
