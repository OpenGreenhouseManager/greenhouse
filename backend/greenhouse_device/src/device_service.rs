//! Alerts to the scripting API.

use alloc::string::String;
use alloc::{format, vec};

use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::Severity, post_create_alert::CreateAlertDto,
};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::platform::{Arc, http_post};

/// An alert raised by a device; see [`trigger_alert`].
pub struct AlertCreation {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

/// POSTs an alert to the configured scripting API (`{url}/alert`).
///
/// Fails with [`Error::ScriptingApiNotConfigured`] until the device has been
/// activated. On the ESP32, `https://` URLs are verified against the root CAs in
/// `certs/roots.pem` (chain, signature and hostname, but not validity dates: the
/// device has no wall clock).
pub async fn trigger_alert<T>(config: Arc<Config<T>>, alert: AlertCreation) -> Result<()> {
    let scripting_api = config.scripting_api.as_ref().ok_or_else(|| {
        log::warn!("scripting_api missing; cannot send alert");
        Error::ScriptingApiNotConfigured
    })?;
    let dto = CreateAlertDto {
        severity: alert.severity,
        identifier: alert.identifier,
        value: alert.value,
        note: alert.note,
        datasource_id: config.datasource_id.clone(),
    };
    let body = serde_json::to_vec(&dto).map_err(|_| Error::Serialization)?;
    let url = format!("{}/alert", scripting_api.url.trim_end_matches('/'));
    let headers = vec![
        ("Content-Type", String::from("application/json")),
        ("Cookie", format!("auth-token={}", scripting_api.token)),
    ];

    let result = match http_post(url.clone(), headers, body).await {
        Ok(status) if (200..300).contains(&status) => Ok(()),
        Ok(status) => Err(Error::HttpStatus(status)),
        Err(e) => Err(e),
    };
    if let Err(e) = &result {
        log::warn!("alert POST to {url} failed: {e}");
    }
    result
}
