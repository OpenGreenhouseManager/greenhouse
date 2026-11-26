use super::error::{Error, Result};
use greenhouse_core::{
    service_client::{ClientError, SmartDeviceClient},
    smart_device_dto::activation::ActivateRequestDto,
};

pub(crate) async fn request_device_config(
    devices: &SmartDeviceClient,
    device_address: &str,
) -> Result<String> {
    devices
        .get_config(device_address)
        .await
        .map_err(|e| device_error(&e, device_address))
}

pub(crate) async fn request_device_config_update(
    devices: &SmartDeviceClient,
    device_address: &str,
    body: serde_json::Value,
) -> Result<String> {
    devices
        .update_config(device_address, &body)
        .await
        .map_err(|e| device_error(&e, device_address))
}

pub(crate) async fn request_device_status(
    devices: &SmartDeviceClient,
    device_address: &str,
) -> Result<String> {
    devices
        .get_status(device_address)
        .await
        .map_err(|e| device_error(&e, device_address))
}

pub(crate) async fn request_device_activate(
    devices: &SmartDeviceClient,
    device_address: &str,
    scripting_api: ActivateRequestDto,
) -> Result<String> {
    devices
        .activate(device_address, &scripting_api)
        .await
        .map_err(|e| device_error(&e, device_address))
}

/// Maps a failed device call: no connection means the device is not
/// reachable, anything else is a bad response.
fn device_error(e: &ClientError, device_address: &str) -> Error {
    sentry::capture_error(e);
    tracing::error!("Error in call to smart device {}: {:?}", device_address, e);

    match e {
        ClientError::Request(_) => Error::SmartDeviceNotReachable,
        ClientError::Decode(_) | ClientError::Api { .. } => Error::SmartDeviceResponse,
    }
}
