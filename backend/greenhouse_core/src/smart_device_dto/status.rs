use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub enum DeviceStatusDto {
    Online,
    Panic,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct DeviceStatusResponseDto {
    pub status: DeviceStatusDto,
    pub datasource_id: String,
}
