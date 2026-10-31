use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct DeviceResponseDto {
    pub id: String,
    pub name: String,
    pub address: String,
    pub description: String,
    pub canscript: bool,
    pub scraping: bool,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct DevicesResponseDto {
    pub devices: Vec<DeviceResponseDto>,
}

impl From<Vec<DeviceResponseDto>> for DevicesResponseDto {
    fn from(devices: Vec<DeviceResponseDto>) -> Self {
        Self { devices }
    }
}
