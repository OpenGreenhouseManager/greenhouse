use std::sync::Arc;
use std::time::Duration;

use embedded_io::Write;
use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::Severity, post_create_alert::CreateAlertDto,
};

use crate::config::Config;
use crate::error::{Error, Result};

const ALERT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct AlertCreation {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

/// POSTs an alert to the configured scripting API. Blocking.
///
/// `https://` URLs are verified against ESP-IDF's built-in Mozilla CA bundle
/// (`CONFIG_MBEDTLS_CERTIFICATE_BUNDLE`), so the server needs a certificate
/// from a public CA; self-signed certificates are rejected.
///
/// The TLS handshake needs a fair amount of stack. When calling this from a
/// request handler, the HTTP server task stack set in `DeviceBuilder::run`
/// covers it; from your own threads, give them at least ~10 KiB.
pub fn trigger_alert<T>(config: &Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
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

    let body = serde_json::to_vec(&dto).map_err(|_| Error::SerializationError)?;
    let url = format!("{}/alert", scripting_api.url.trim_end_matches('/'));
    let cookie = format!("auth-token={}", scripting_api.token);
    let content_length = body.len().to_string();

    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Length", content_length.as_str()),
        ("Cookie", cookie.as_str()),
    ];

    let connection = EspHttpConnection::new(&Configuration {
        // Without a CA source esp-tls fails every https handshake.
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        timeout: Some(ALERT_TIMEOUT),
        ..Default::default()
    })?;
    let mut client = Client::wrap(connection);

    let mut request = client.post(&url, &headers).map_err(|e| {
        log::warn!("alert POST to {url} failed to open: {e:?}");
        Error::IoError
    })?;
    request.write_all(&body).map_err(|e| {
        log::warn!("alert POST body write failed: {e:?}");
        Error::IoError
    })?;
    request.flush().map_err(|_| Error::IoError)?;

    let response = request.submit().map_err(|e| {
        log::warn!("alert POST submit failed: {e:?}");
        Error::IoError
    })?;

    let status = response.status();
    if !(200..300).contains(&status) {
        log::warn!("alert POST returned HTTP {status}");
        return Err(Error::HttpStatus(status));
    }
    Ok(())
}
