use crate::scripting_service_dto::{endpoints, token::TokenDto};

use super::{ClientResult, ServiceClient};

/// Typed client for `scripting_service`.
#[derive(Clone, Debug)]
pub struct ScriptingServiceClient(ServiceClient);

impl ScriptingServiceClient {
    pub fn new(http: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self(ServiceClient::new(http, base_url))
    }

    /// Succeeds when `token` is a valid scripting token.
    pub async fn check_token(&self, token: &str) -> ClientResult<()> {
        self.0
            .post(endpoints::CHECK_TOKEN)
            .json(&TokenDto {
                token: token.to_string(),
            })
            .send_empty()
            .await
    }
}
