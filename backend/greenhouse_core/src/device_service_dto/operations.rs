use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct OperationsDto {
    pub operations: Vec<String>,
}

impl From<Vec<String>> for OperationsDto {
    fn from(operations: Vec<String>) -> Self {
        Self { operations }
    }
}
