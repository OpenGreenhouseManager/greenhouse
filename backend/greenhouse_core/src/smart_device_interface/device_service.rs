use crate::data_storage_service_dto::alert_dto::alert::{AlertKind, Severity};
use crate::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use crate::data_storage_service_dto::alert_dto::post_stop_alert::StopAlertDto;
use std::sync::Arc;

use super::config::Config;
use super::{Error, Result};

pub struct AlertCreation {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
    pub kind: AlertKind,
    /// Concurrent alerts only; see `CreateAlertDto::keep_alive_secs`.
    pub keep_alive_secs: Option<u32>,
}

/// Fires an alert through the scripting API.
///
/// A one-time alert records a single event. A concurrent alert stays open
/// while it keeps being fired: fire it on every sample while the condition
/// lasts, and call [`stop_alert`] when it is over. A
/// [`Debouncer`](crate::alert_debounce::Debouncer) tells you when to do which.
pub async fn trigger_alert<T>(config: Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
    if let Some(scripting_api) = &config.scripting_api {
        let client = reqwest::Client::new();

        let alert = CreateAlertDto {
            severity: alert.severity,
            identifier: alert.identifier,
            value: alert.value,
            note: alert.note,
            datasource_id: config.datasource_id.clone(),
            kind: alert.kind,
            keep_alive_secs: alert.keep_alive_secs,
        };

        let response = client
            .post(format!("{}/alert", scripting_api.url))
            .json(&alert)
            .header("Access-Control-Allow-Credentials", "true")
            .header("Cookie", format!("auth-token={}", scripting_api.token))
            .send()
            .await
            .map_err(Error::Request)?;

        if !response.status().is_success() {
            return Err(Error::Request(response.error_for_status().unwrap_err()));
        }
        return Ok(());
    }
    Err(Error::ScriptingApiNotConfigured)
}

/// Stops the open concurrent alert with this identifier, if there is one.
pub async fn stop_alert<T>(
    config: Arc<Config<T>>,
    identifier: String,
    note: Option<String>,
) -> Result<()>
where
    T: Clone + Default,
{
    let Some(scripting_api) = &config.scripting_api else {
        return Err(Error::ScriptingApiNotConfigured);
    };
    let stop = StopAlertDto {
        identifier,
        datasource_id: config.datasource_id.clone(),
        note,
    };
    let response = reqwest::Client::new()
        .post(format!("{}/alert/stop", scripting_api.url))
        .json(&stop)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={}", scripting_api.token))
        .send()
        .await
        .map_err(Error::Request)?;

    if !response.status().is_success() {
        return Err(Error::Request(response.error_for_status().unwrap_err()));
    }
    Ok(())
}
