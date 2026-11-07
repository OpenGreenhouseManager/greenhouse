use serde::{Deserialize, Serialize};

/// Stops the open concurrent alert with this `datasource_id` and `identifier`
/// before it expires.
///
/// Stopping when no alert is open is not an error, so senders can stop
/// without tracking whether they fired before (e.g. after a restart).
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub struct StopAlertDto {
    pub identifier: String,
    pub datasource_id: String,
    /// Optional note stored on the stopped alert, replacing the previous one.
    pub note: Option<String>,
}
