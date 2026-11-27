pub(crate) use self::error::Error;
mod error;

pub(crate) mod one_time_token {
    pub(crate) use super::error::{Error, Result};

    use std::hash::{DefaultHasher, Hash, Hasher};

    pub(crate) fn generate_one_time_token(user_name: &str, secret: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        let str = String::from(user_name) + secret;
        str.hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn check_one_time_token(user_name: &str, token: u64, secret: &str) -> Result<()> {
        let new_token = generate_one_time_token(user_name, secret);
        if new_token == token {
            return Ok(());
        }
        Err(Error::RegisterToken)
    }
}

pub(crate) mod user_token {
    pub(crate) use super::error::{Error, Result};
    use std::{path::Path, sync::Arc};

    use chrono::Utc;
    use greenhouse_core::auth_service_dto::user_token::{
        ACCESS_TOKEN_TTL_SECS, AUDIENCE, ISSUER, UserToken,
    };
    use jsonwebtoken::{Algorithm, EncodingKey, Header};

    /// Signs user access tokens with auth_service's Ed25519 private key.
    #[derive(Clone)]
    pub(crate) struct TokenSigner {
        key: Arc<EncodingKey>,
        kid: String,
    }

    impl TokenSigner {
        /// Builds a signer from a PEM encoded Ed25519 private key (PKCS#8).
        pub(crate) fn from_pem(pem: &[u8], kid: &str) -> Result<Self> {
            Ok(Self {
                key: Arc::new(EncodingKey::from_ed_pem(pem).map_err(|_| Error::KeyLoad)?),
                kid: kid.to_string(),
            })
        }

        pub(crate) fn from_file(path: impl AsRef<Path>, kid: &str) -> Result<Self> {
            let pem = std::fs::read(path).map_err(|_| Error::KeyLoad)?;
            Self::from_pem(&pem, kid)
        }

        /// Issues a short-lived access token for session `sid`.
        pub(crate) fn issue(&self, user_name: &str, role: &str, sid: &str) -> Result<String> {
            let now = Utc::now().timestamp();
            let payload = UserToken {
                iat: now,
                exp: now + ACCESS_TOKEN_TTL_SECS,
                user_name: String::from(user_name),
                role: String::from(role),
                sid: String::from(sid),
                iss: String::from(ISSUER),
                aud: String::from(AUDIENCE),
            };

            let mut header = Header::new(Algorithm::EdDSA);
            header.kid = Some(self.kid.clone());

            jsonwebtoken::encode(&header, &payload, &self.key).map_err(|e| {
                sentry::configure_scope(|scope| {
                    scope.set_user(Some(sentry::User {
                        username: Some(String::from(user_name)),
                        ..Default::default()
                    }));
                });
                sentry::capture_error(&e);
                Error::JwtEncode
            })
        }
    }
}

pub(crate) mod session {
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    /// A new random refresh token (244 bits of entropy).
    pub(crate) fn new_refresh_token() -> String {
        format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
    }

    /// What is stored for a refresh token, so a database leak does not
    /// hand out live sessions.
    pub(crate) fn hash_refresh_token(token: &str) -> String {
        Sha256::digest(token.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use greenhouse_core::{
        auth::{JwtVerifier, VerifyError},
        auth_service_dto::user_token::ACCESS_TOKEN_TTL_SECS,
    };

    use super::*;
    const SECRET: &str = "XOJ~uQ7&AlPVs?1tm~4bD5nU$~E$iI702st]l|im:p8uTTj+dZX,R_QFvx4`*{r";
    const PRIVATE_KEY: &[u8] = include_bytes!("../../../../test-fixtures/jwt/test_jwt_private.pem");
    const OTHER_PRIVATE_KEY: &[u8] =
        include_bytes!("../../../../test-fixtures/jwt/test_jwt_other_private.pem");
    const PUBLIC_KEY: &[u8] = include_bytes!("../../../../test-fixtures/jwt/test_jwt_public.pem");

    fn signer() -> user_token::TokenSigner {
        user_token::TokenSigner::from_pem(PRIVATE_KEY, "test").unwrap()
    }

    fn verifier() -> JwtVerifier {
        JwtVerifier::from_public_key_pem(PUBLIC_KEY).unwrap()
    }

    #[test]
    fn issued_token_verifies_with_public_key() {
        let token = signer().issue("testUser1", "test", "sid-1").unwrap();
        let claims = verifier().verify(&token).unwrap();
        assert_eq!(claims.user_name, "testUser1");
        assert_eq!(claims.role, "test");
        assert_eq!(claims.sid, "sid-1");
        assert_eq!(claims.exp - claims.iat, ACCESS_TOKEN_TTL_SECS);
    }

    #[test]
    fn issued_token_carries_kid() {
        let token = signer().issue("testUser1", "test", "sid-1").unwrap();
        let header = jsonwebtoken::decode_header(&token).unwrap();
        assert_eq!(header.kid.as_deref(), Some("test"));
        assert_eq!(header.alg, jsonwebtoken::Algorithm::EdDSA);
    }

    #[test]
    fn token_from_other_key_is_rejected() {
        let other = user_token::TokenSigner::from_pem(OTHER_PRIVATE_KEY, "test").unwrap();
        let token = other.issue("testUser1", "admin", "sid-1").unwrap();
        assert_eq!(verifier().verify(&token).unwrap_err(), VerifyError::Invalid);
    }

    #[test]
    fn tampered_token_is_rejected() {
        let token = signer().issue("testUser1", "guest", "sid-1").unwrap();
        let mut parts: Vec<String> = token.split('.').map(String::from).collect();
        let forged = signer().issue("testUser1", "admin", "sid-1").unwrap();
        parts[1] = forged.split('.').nth(1).unwrap().to_string();
        // Payload of the admin token with the signature of the guest token.
        let tampered = parts.join(".");
        assert_ne!(tampered, forged);
        assert_eq!(
            verifier().verify(&tampered).unwrap_err(),
            VerifyError::Invalid
        );
    }

    #[test]
    fn hs256_token_is_rejected() {
        // Old-style token signed with a shared secret.
        let claims = serde_json::json!({
            "iat": 0, "exp": i64::MAX / 2, "user_name": "u", "role": "admin",
            "sid": "s", "iss": "greenhouse-auth", "aud": "greenhouse-web"
        });
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .unwrap();
        assert_eq!(verifier().verify(&token).unwrap_err(), VerifyError::Invalid);
    }

    #[test]
    fn refresh_token_hash_is_stable_and_distinct() {
        let a = session::new_refresh_token();
        let b = session::new_refresh_token();
        assert_ne!(a, b);
        assert_eq!(
            session::hash_refresh_token(&a),
            session::hash_refresh_token(&a)
        );
        assert_ne!(
            session::hash_refresh_token(&a),
            session::hash_refresh_token(&b)
        );
        assert_eq!(session::hash_refresh_token(&a).len(), 64);
    }

    #[test]
    fn check_one_time_token_test() {
        let token = one_time_token::generate_one_time_token("testUser1", SECRET);
        one_time_token::check_one_time_token("testUser1", token, SECRET).unwrap();
    }

    #[test]
    fn check_one_time_token_fail_test() {
        let token = one_time_token::generate_one_time_token("testUser1", SECRET);
        assert_eq!(
            one_time_token::check_one_time_token("testUser1", token + 1, SECRET).unwrap_err(),
            (Error::RegisterToken)
        );
    }

    #[test]
    fn check_one_time_token_fail_different_user_test() {
        let token = one_time_token::generate_one_time_token("testUser1", SECRET);
        assert_eq!(
            one_time_token::check_one_time_token("testUser2", token, SECRET).unwrap_err(),
            (Error::RegisterToken)
        );
    }

    #[test]
    fn check_one_time_token_fail_different_secret_test() {
        let token = one_time_token::generate_one_time_token("testUser1", SECRET);
        assert_eq!(
            one_time_token::check_one_time_token("testUser1", token, "broken").unwrap_err(),
            (Error::RegisterToken)
        );
    }

    #[test]
    fn check_one_time_token_fail_different_user_and_secret_test() {
        let token = one_time_token::generate_one_time_token("testUser1", SECRET);
        assert_eq!(
            one_time_token::check_one_time_token("testUser2", token, "broken").unwrap_err(),
            (Error::RegisterToken)
        );
    }

    #[test]
    fn check_one_time_token_different_user_different_token() {
        let token = one_time_token::generate_one_time_token("testUser1", SECRET);
        let token2 = one_time_token::generate_one_time_token("testUser2", SECRET);
        assert_ne!(token, token2);
    }
}
