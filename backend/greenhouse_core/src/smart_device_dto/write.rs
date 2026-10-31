use serde::{Deserialize, Serialize};

use crate::smart_device_dto::Type;

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct WriteRequestDto {
    pub data: Type,
}
