use std::sync::LazyLock;

use crate::error::{Error, Result};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// POSTs `body` to `url` and returns the HTTP status code.
pub(crate) async fn http_post(
    url: String,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
) -> Result<u16> {
    let request = headers
        .into_iter()
        .fold(CLIENT.post(url), |request, (name, value)| {
            request.header(name, value)
        });
    let response = request.body(body).send().await.map_err(Error::Request)?;
    Ok(response.status().as_u16())
}
