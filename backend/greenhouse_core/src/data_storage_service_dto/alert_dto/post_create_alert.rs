use serde::{Deserialize, Serialize};

use super::alert::Severity;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub struct CreateAlertDto {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
    pub datasource_id: String,
}
