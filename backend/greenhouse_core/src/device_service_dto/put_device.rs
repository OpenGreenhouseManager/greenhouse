use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct PutDeviceDtoRequest {
    pub name: String,
    pub description: String,
    pub address: String,
    pub can_script: bool,
    pub scraping: bool,
}
