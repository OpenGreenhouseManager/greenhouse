use std::sync::Arc;

pub use crate::smart_device_common::AlertCreation;

use super::config::Config;
use super::{Error, Result};

pub async fn trigger_alert<T>(config: Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
    if let Some(scripting_api) = &config.scripting_api {
        let client = reqwest::Client::new();

        let alert = alert.into_dto(&config);

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
