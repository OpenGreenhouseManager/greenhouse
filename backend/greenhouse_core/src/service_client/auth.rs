use crate::auth_service_dto::{
    endpoints,
    generate_one_time_token::{GenerateOneTimeTokenRequestDto, GenerateOneTimeTokenResponseDto},
    login::LoginRequestDto,
    register::RegisterRequestDto,
    session::{LogoutRequestDto, RefreshRequestDto, SessionDto},
    user_preferences::{
        GetPreferencesRequestDto, SetPreferencesRequestDto, UserPreferencesRequestDto,
        UserPreferencesResponseDto,
    },
};

use super::{ClientResult, ServiceClient};

/// Typed client for `auth_service`.
///
/// Access tokens are verified locally by [`crate::auth::JwtVerifier`]; this
/// client is only needed for login, register, refresh and preferences.
#[derive(Clone, Debug)]
pub struct AuthServiceClient(ServiceClient);

impl AuthServiceClient {
    pub fn new(http: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self(ServiceClient::new(http, base_url))
    }

    pub async fn register(&self, request: &RegisterRequestDto) -> ClientResult<SessionDto> {
        self.0
            .post(endpoints::REGISTER)
            .json(request)
            .send_json()
            .await
    }

    /// Logs in and starts a new session, replacing the user's previous one.
    pub async fn login(&self, request: &LoginRequestDto) -> ClientResult<SessionDto> {
        self.0
            .post(endpoints::LOGIN)
            .json(request)
            .send_json()
            .await
    }

    /// Issues a new access token if `refresh_token` is the user's current,
    /// unexpired session.
    pub async fn refresh(&self, refresh_token: &str) -> ClientResult<SessionDto> {
        self.0
            .post(endpoints::REFRESH)
            .json(&RefreshRequestDto {
                refresh_token: refresh_token.to_string(),
            })
            .send_json()
            .await
    }

    /// Ends the session of `refresh_token`, so it can no longer be refreshed.
    pub async fn logout(&self, refresh_token: &str) -> ClientResult<()> {
        self.0
            .post(endpoints::LOGOUT)
            .json(&LogoutRequestDto {
                refresh_token: refresh_token.to_string(),
            })
            .send_empty()
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

    /// Preferences of `user_name`, who must come from a verified token.
    pub async fn get_preferences(
        &self,
        user_name: &str,
    ) -> ClientResult<UserPreferencesResponseDto> {
        self.0
            .post(&format!("{}/get", endpoints::PREFERENCES))
            .json(&GetPreferencesRequestDto {
                user_name: user_name.to_string(),
            })
            .send_json()
            .await
    }

    /// Stores preferences of `user_name`, who must come from a verified token.
    pub async fn set_preferences(
        &self,
        user_name: &str,
        preferences: UserPreferencesRequestDto,
    ) -> ClientResult<UserPreferencesResponseDto> {
        self.0
            .post(&format!("{}/set", endpoints::PREFERENCES))
            .json(&SetPreferencesRequestDto {
                user_name: user_name.to_string(),
                preferences,
            })
            .send_json()
            .await
    }
}
