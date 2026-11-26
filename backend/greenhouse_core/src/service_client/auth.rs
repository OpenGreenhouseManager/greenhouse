use crate::auth_service_dto::{
    endpoints,
    generate_one_time_token::{GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto},
    login::{LoginRequestDto, LoginResponseDto},
    register::{RegisterRequestDto, RegisterResponseDto},
    token::{TokenRequestDto, TokenResponseDto},
    user_preferences::{
        SetPreferencesRequestDto, UserPreferencesRequestDto, UserPreferencesResponseDto,
    },
};

use super::{ClientResult, ServiceClient};

/// Typed client for `auth_service`.
#[derive(Clone, Debug)]
pub struct AuthServiceClient(ServiceClient);

impl AuthServiceClient {
    pub fn new(http: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self(ServiceClient::new(http, base_url))
    }

    pub async fn register(
        &self,
        request: &RegisterRequestDto,
    ) -> ClientResult<RegisterResponseDto> {
        self.0
            .post(endpoints::REGISTER)
            .json(request)
            .send_json()
            .await
    }

    pub async fn login(&self, request: &LoginRequestDto) -> ClientResult<LoginResponseDto> {
        self.0
            .post(endpoints::LOGIN)
            .json(request)
            .send_json()
            .await
    }

    /// Verifies the signature and expiry of a user JWT and returns its role.
    pub async fn check_token(&self, token: &str) -> ClientResult<TokenResponseDto> {
        self.0
            .post(endpoints::CHECK_TOKEN)
            .json(&bearer(token))
            .send_json()
            .await
    }

    pub async fn generate_one_time_token(&self, username: &str) -> ClientResult<String> {
        let resp: GenerateOneTimeTokenResponseDto = self
            .0
            .post(endpoints::GENERATE_ONE_TIME_TOKEN)
            .json(&GenerateOneTimeTokenRequestDto {
                username: username.to_string(),
            })
            .send_json()
            .await?;
        Ok(resp.token)
    }

    pub async fn get_preferences(&self, token: &str) -> ClientResult<UserPreferencesResponseDto> {
        self.0
            .post(&format!("{}/get", endpoints::PREFERENCES))
            .json(&bearer(token))
            .send_json()
            .await
    }

    pub async fn set_preferences(
        &self,
        token: &str,
        preferences: UserPreferencesRequestDto,
    ) -> ClientResult<UserPreferencesResponseDto> {
        self.0
            .post(&format!("{}/set", endpoints::PREFERENCES))
            .json(&SetPreferencesRequestDto {
                token: token.to_string(),
                preferences,
            })
            .send_json()
            .await
    }
}

fn bearer(token: &str) -> TokenRequestDto {
    TokenRequestDto {
        token: token.to_string(),
        token_type: String::from("Bearer"),
    }
}
