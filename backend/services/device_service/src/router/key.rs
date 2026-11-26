use crate::{
    AppState, Config, Pool,
    database::{device::Device, device_key::DeviceKey},
    router::{
        error::{Error, HttpResult, Result},
        service::request_device_activate,
    },
};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use greenhouse_core::{
    device_service_dto::{endpoints::CHECK_KEY, key::DeviceKeyDto},
    service_client::SmartDeviceClient,
    smart_device_dto::activation::ActivateRequestDto,
};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route(CHECK_KEY, post(check_key))
        .with_state(state)
}

#[axum::debug_handler]
pub(crate) async fn check_key(
    State(AppState { pool, .. }): State<AppState>,
    Json(DeviceKeyDto { key }): Json<DeviceKeyDto>,
) -> HttpResult<StatusCode> {
    if DeviceKey::exists(&key, &pool).await? {
        Ok(StatusCode::OK)
    } else {
        Err(Error::UnknownKey.into())
    }
}

/// Sends `device` a fresh scripting key. Once the device confirmed it, its
/// older keys are revoked. If the call failed, every key stays valid: the
/// device may still have received the new one, or still use an old one. The
/// next successful activation or the device's deletion cleans up.
pub(crate) async fn activate_with_new_key(
    config: &Config,
    pool: &Pool,
    devices: &SmartDeviceClient,
    device: &Device,
) -> Result<()> {
    let key = DeviceKey::issue(device.id, pool).await?;

    let activation = request_device_activate(
        devices,
        &device.address,
        ActivateRequestDto {
            url: config.scripting_api.clone(),
            token: key.key.clone(),
        },
    )
    .await;

    activation?;
    key.revoke_others(pool).await?;
    Ok(())
}
