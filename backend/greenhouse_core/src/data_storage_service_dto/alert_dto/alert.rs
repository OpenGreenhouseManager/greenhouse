use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Fatal,
}

/// How long an alert lasts.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub enum AlertKind {
    /// A single event, e.g. "door opened". Starts and ends at the same moment
    /// and is never merged with other alerts.
    #[default]
    OneTime,
    /// A condition that lasts, e.g. "device unreachable". Stays open while the
    /// sender keeps firing it: every fire extends `expires_at` by the alert's
    /// keep-alive. It ends when stopped, or when it is not fired again before
    /// `expires_at`, so a sender that dies cannot leave it open forever.
    Concurrent,
}

/// Why a [`AlertKind::Concurrent`] alert ended.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub enum EndReason {
    /// The sender reported that the condition is over.
    Stopped,
    /// Nobody fired it again before `expires_at`. The condition may still be
    /// present; it just was not confirmed any more.
    Expired,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub struct AlertDto {
    pub id: String,
    pub severity: Severity,
    pub identifier: String,
    pub value: String,
    pub note: Option<String>,
    /// When the alert started (first fire).
    pub created_at: String,
    pub datasource_id: String,
    // The fields below default when missing so callers still parse responses
    // from older data storage builds.
    #[serde(default)]
    pub kind: AlertKind,
    /// Most recent fire. Equal to `created_at` for one-time alerts.
    #[serde(default)]
    pub last_seen_at: String,
    /// When the alert ends, or ended. It is open while this is in the future.
    /// Equal to `created_at` for one-time alerts.
    #[serde(default)]
    pub expires_at: String,
    /// Whether the alert is open, as of the response. Always `false` for
    /// one-time alerts.
    #[serde(default)]
    pub open: bool,
    /// Keep-alive of a concurrent alert; `None` for one-time alerts.
    #[serde(default)]
    pub keep_alive_secs: Option<u32>,
    /// Why a concurrent alert ended; `None` while open and for one-time alerts.
    #[serde(default)]
    pub end_reason: Option<EndReason>,
    /// How often the alert was fired.
    #[serde(default)]
    pub occurrences: i64,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/alert/")
)]
pub struct AlertsDto {
    pub alerts: Vec<AlertDto>,
}

impl From<Vec<AlertDto>> for AlertsDto {
    fn from(alerts: Vec<AlertDto>) -> Self {
        Self { alerts }
    }
}
