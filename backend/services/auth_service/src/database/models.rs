use crate::token::{self, user_token::TokenSigner};

use super::{Error, Result};
use bcrypt::Version;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use greenhouse_core::auth_service_dto::{
    session::SessionDto, user_token::REFRESH_SESSION_TTL_SECS,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Deserialize, Insertable)]
#[diesel(table_name = crate::database::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) username: String,
    pub(crate) role: String,
    /// SHA-256 of the current refresh token, empty when logged out.
    pub(crate) login_session: String,
    hash: String,
    pub(crate) session_id: Option<Uuid>,
    pub(crate) session_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Queryable, Selectable, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::database::schema::preferences)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Preferences {
    pub(crate) id: Uuid,
    pub(crate) dashboard_preferences: serde_json::Value,
    pub(crate) alert_preferences: serde_json::Value,
    pub(crate) user_id: Uuid,
}

impl Preferences {
    pub(crate) fn new(
        user_id: Uuid,
        dashboard_preferences: serde_json::Value,
        alert_preferences: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            dashboard_preferences,
            alert_preferences,
            user_id,
        }
    }
}

impl User {
    pub(crate) fn new(username: &str, password: &str, role: &str) -> Result<Self> {
        let password_hash = bcrypt::hash_with_result(password, 12).map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(String::from(username)),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), username.into());
                map.insert(String::from("role"), role.into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });
            sentry::capture_error(&e);
            Error::InvalidHash
        })?;

        Ok(Self {
            id: Uuid::new_v4(),
            username: String::from(username),
            hash: password_hash.format_for_version(Version::TwoB),
            role: String::from(role),
            login_session: String::from(""),
            session_id: None,
            session_expires_at: None,
        })
    }

    /// Starts a new refresh session, replacing any previous one, and issues
    /// its first access token. The caller stores the changed fields.
    pub(crate) fn start_session(&mut self, signer: &TokenSigner) -> Result<SessionDto> {
        let refresh_token = token::session::new_refresh_token();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(REFRESH_SESSION_TTL_SECS);

        self.login_session = token::session::hash_refresh_token(&refresh_token);
        self.session_id = Some(session_id);
        self.session_expires_at = Some(expires_at);

        Ok(SessionDto {
            token: signer.issue(&self.username, &self.role, &session_id.to_string())?,
            token_type: String::from("Bearer"),
            refresh_token,
            refresh_expires_at: expires_at.timestamp(),
        })
    }

    /// Whether `refresh_token` belongs to this user's current, unexpired
    /// session.
    pub(crate) fn session_is_valid(&self, refresh_token: &str) -> bool {
        !self.login_session.is_empty()
            && self.login_session == token::session::hash_refresh_token(refresh_token)
            && self.session_expires_at.is_some_and(|at| at > Utc::now())
    }

    pub(crate) async fn check_login(&self, password: &str) -> Result<bool> {
        bcrypt::verify(password, &self.hash).map_err(|e| {
            sentry::configure_scope(|scope| {
                scope.set_user(Some(sentry::User {
                    username: Some(self.username.clone()),
                    ..Default::default()
                }));
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("username"), self.username.clone().into());
                map.insert(String::from("role"), self.role.clone().into());

                scope.set_context("user_long", sentry::protocol::Context::Other(map));
            });

            sentry::capture_error(&e);
            Error::InvalidHash
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user() {
        let user = User::new("testUser1", "testPassword1", "test").expect("Failed to create user");

        assert_eq!(user.username, "testUser1");
        assert_eq!(user.role, "test");
    }

    fn signer() -> TokenSigner {
        TokenSigner::from_pem(
            include_bytes!("../../../../test-fixtures/jwt/test_jwt_private.pem"),
            "test",
        )
        .unwrap()
    }

    #[test]
    fn new_session_replaces_old_one() {
        let mut user = User::new("testUser1", "testPassword1", "test").unwrap();
        assert!(!user.session_is_valid(""));

        let first = user.start_session(&signer()).unwrap();
        assert!(user.session_is_valid(&first.refresh_token));

        let second = user.start_session(&signer()).unwrap();
        assert!(user.session_is_valid(&second.refresh_token));
        assert!(!user.session_is_valid(&first.refresh_token));
    }

    #[test]
    fn expired_session_is_invalid() {
        let mut user = User::new("testUser1", "testPassword1", "test").unwrap();
        let session = user.start_session(&signer()).unwrap();
        user.session_expires_at = Some(Utc::now() - Duration::seconds(1));
        assert!(!user.session_is_valid(&session.refresh_token));
    }

    #[allow(clippy::needless_return)] // https://github.com/rust-lang/rust-clippy/issues/13458
    #[tokio::test]
    async fn check_login() {
        let user = User::new("testUser1", "testPassword1", "test").expect("Failed to create user");

        assert!(
            user.check_login("testPassword1")
                .await
                .expect("Failed to check login")
        );
        assert!(
            !user
                .check_login("wrongPassword")
                .await
                .expect("Failed to check login")
        );
    }
}
