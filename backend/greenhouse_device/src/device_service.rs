//! Alerts to the scripting API.

use alloc::format;

pub use crate::config::AlertCreation;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::platform::{Arc, send_alert};

/// POSTs an alert to the configured scripting API (`{url}/alert`).
///
/// Fails with [`Error::ScriptingApiNotConfigured`] until the device has been
/// activated. On the ESP32, `https://` URLs are verified against the root CAs in
/// `certs/roots.pem` (chain, signature and hostname, but not validity dates: the
/// device has no wall clock).
pub async fn trigger_alert<T>(config: Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
    let scripting_api = config.scripting_api.as_ref().ok_or_else(|| {
        log::warn!("scripting_api missing; cannot send alert");
        Error::ScriptingApiNotConfigured
    })?;
    let body =
        serde_json::to_vec(&alert.into_dto(&config)).map_err(|_| Error::SerializationError)?;
    let url = format!("{}/alert", scripting_api.url.trim_end_matches('/'));
    send_alert(url, scripting_api.token.clone(), body).await
}
