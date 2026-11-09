use std::sync::LazyLock;

use reqwest::header::{CONTENT_TYPE, COOKIE};

use crate::error::{Error, Result};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// POSTs a serialized alert to `url`, authenticated with the device's token.
pub(crate) async fn send_alert(url: String, token: String, body: Vec<u8>) -> Result<()> {
    let response = CLIENT
        .post(&url)
        .header(CONTENT_TYPE, "application/json")
        .header(COOKIE, format!("auth-token={token}"))
        .body(body)
        .send()
        .await
        .map_err(|e| {
            log::warn!("alert POST to {url} failed: {e}");
            Error::Request(e)
        })?;
    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        log::warn!("alert POST to {url} answered {status}");
        Err(Error::HttpStatus(status.as_u16()))
    }
}
