use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

use super::alert::Severity;

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub struct AggrigatedAlertDto {
    /// Number of alerts in the group that were active during the interval.
    pub count: i64,
    /// Number of those alerts that are still open.
    #[serde(default)]
    pub open: i64,
    pub identifier: String,
    pub severity: Severity,
    pub source: String,
    /// Start of the earliest alert.
    pub first: String,
    /// Most recent time any alert in the group was fired.
    pub last: String,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub struct AggrigatedAlertsDto {
    pub alerts: Vec<AggrigatedAlertDto>,
}

impl From<Vec<AggrigatedAlertDto>> for AggrigatedAlertsDto {
    fn from(alerts: Vec<AggrigatedAlertDto>) -> Self {
        Self { alerts }
    }
}
