use serde::{Deserialize, Serialize};

use super::Type;

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct ReadResponseDto {
    pub data: Type,
}
