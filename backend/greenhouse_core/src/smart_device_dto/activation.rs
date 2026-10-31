use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct ActivateRequestDto {
    pub url: String,
    pub token: String,
}
