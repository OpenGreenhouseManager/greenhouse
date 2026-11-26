use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

/// A device's scripting key. Devices send it as their `auth-token` when they
/// call `scripting_api`.
#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct DeviceKeyDto {
    pub key: String,
}
