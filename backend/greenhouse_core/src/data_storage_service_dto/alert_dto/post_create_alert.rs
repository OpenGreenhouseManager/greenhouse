use serde::{Deserialize, Serialize};

use super::alert::{AlertKind, Severity};

/// Keep-alive of a concurrent alert when the sender does not set one.
pub const DEFAULT_KEEP_ALIVE_SECS: u32 = 300;
/// Longest keep-alive a concurrent alert may have (one day).
pub const MAX_KEEP_ALIVE_SECS: u32 = 86_400;

/// Fires an alert.
///
/// - [`AlertKind::OneTime`] (the default) records a single event.
/// - [`AlertKind::Concurrent`] opens an alert, or, if one with the same
///   `datasource_id` and `identifier` is still open, updates it (`severity`,
///   `value`, `note`, `last_seen_at`, `occurrences`) and moves its `expires_at`
///   to now + `keep_alive_secs`.
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
    #[serde(default)]
    pub kind: AlertKind,
    /// Concurrent alerts only: seconds until the alert expires unless fired
    /// again, 1 to [`MAX_KEEP_ALIVE_SECS`]. Defaults to
    /// [`DEFAULT_KEEP_ALIVE_SECS`]. Must be unset for one-time alerts.
    #[serde(default)]
    pub keep_alive_secs: Option<u32>,
}
