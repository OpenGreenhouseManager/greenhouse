//! Tests for `AuthUser`, `RequireRole` and `ServiceClient` against locally
//! signed tokens and a fake service on a local port.
#![cfg(all(feature = "auth", feature = "service_client"))]

use axum::{
    Json, Router,
    body::Body,
    extract::FromRef,
    http::{Request, StatusCode, header},
    middleware::from_extractor_with_state,
    routing::{get, post},
};
use greenhouse_core::{
    auth::{Admin, AuthUser, AuthVerifier, JwtVerifier, RequireRole},
    auth_service_dto::user_token::{AUDIENCE, ISSUER, UserToken},
    service_client::{ClientError, DeviceServiceClient, ServiceClient, http_client},
};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Deserialize;
use serde_json::json;
use tower::ServiceExt;
use tower_cookies::CookieManagerLayer;

const PRIVATE_KEY: &[u8] = include_bytes!("../../test-fixtures/jwt/test_jwt_private.pem");
const OTHER_PRIVATE_KEY: &[u8] =
    include_bytes!("../../test-fixtures/jwt/test_jwt_other_private.pem");
const PUBLIC_KEY: &[u8] = include_bytes!("../../test-fixtures/jwt/test_jwt_public.pem");

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn claims(role: &str) -> UserToken {
    UserToken {
        iat: now(),
        exp: now() + 600,
        user_name: "alice".into(),
        role: role.into(),
        sid: "session".into(),
        iss: ISSUER.into(),
        aud: AUDIENCE.into(),
    }
}

fn sign_with(claims: &UserToken, key: &[u8]) -> String {
    jsonwebtoken::encode(
        &Header::new(Algorithm::EdDSA),
        claims,
        &EncodingKey::from_ed_pem(key).unwrap(),
    )
    .unwrap()
}

fn sign(claims: &UserToken) -> String {
    sign_with(claims, PRIVATE_KEY)
}

/// Fake service for the `ServiceClient` tests.
async fn fake_service() -> String {
    let app = Router::new()
        .route("/json", get(|| async { Json(json!({ "role": "x" })) }))
        .route(
            "/plain_error",
            get(|| async { (StatusCode::BAD_REQUEST, "not json") }),
        )
        .route(
            "/json_error",
            post(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Token invalid" })),
                )
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[derive(Clone, FromRef)]
struct TestState {
    verifier: AuthVerifier,
}

fn app_with(verifier: AuthVerifier) -> Router {
    let state = TestState { verifier };
    Router::new()
        .route(
            "/me",
            get(|user: AuthUser| async move {
                format!("{}:{}", user.user_name.unwrap_or_default(), user.role)
            }),
        )
        .route("/admin", get(|_: RequireRole<Admin>| async { "ok" }))
        .with_state(state.clone())
        .layer(from_extractor_with_state::<AuthUser, _>(state))
        .layer(CookieManagerLayer::new())
}

fn test_app() -> Router {
    app_with(AuthVerifier::Jwt(
        JwtVerifier::from_public_key_pem(PUBLIC_KEY).unwrap(),
    ))
}

async fn call(app: &Router, path: &str, token: Option<&str>) -> (StatusCode, String, bool) {
    let mut req = Request::get(path);
    if let Some(token) = token {
        req = req.header(header::COOKIE, format!("auth-token={token}"));
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let removes_cookie = resp
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().unwrap().starts_with("auth-token=;"));
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        String::from_utf8(body.to_vec()).unwrap(),
        removes_cookie,
    )
}

#[tokio::test]
async fn missing_cookie_is_forbidden() {
    let (status, _, _) = call(&test_app(), "/me", None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn invalid_token_is_forbidden_and_cookie_removed() {
    let (status, _, removes_cookie) = call(&test_app(), "/me", Some("forged")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(removes_cookie);
}

#[tokio::test]
async fn token_signed_with_other_key_is_forbidden() {
    let token = sign_with(&claims("admin"), OTHER_PRIVATE_KEY);
    let (status, _, removes_cookie) = call(&test_app(), "/admin", Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(removes_cookie);
}

#[tokio::test]
async fn token_for_other_audience_is_forbidden() {
    let mut c = claims("admin");
    c.aud = "someone-else".into();
    let (status, _, _) = call(&test_app(), "/me", Some(&sign(&c))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn valid_token_is_verified_locally() {
    let (status, body, _) = call(&test_app(), "/me", Some(&sign(&claims("user")))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "alice:user");
}

#[tokio::test]
async fn expired_token_asks_for_refresh_and_keeps_cookie() {
    let mut c = claims("user");
    c.iat = now() - 1200;
    c.exp = now() - 600;
    let (status, _, removes_cookie) = call(&test_app(), "/me", Some(&sign(&c))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(!removes_cookie);
}

#[tokio::test]
async fn verifier_failure_is_unavailable_not_logout() {
    let app = app_with(AuthVerifier::DeviceKey(DeviceServiceClient::new(
        http_client(),
        "http://127.0.0.1:1",
    )));
    let (status, _, removes_cookie) = call(&app, "/me", Some("some-key")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(!removes_cookie);
}

#[tokio::test]
async fn require_role_checks_verified_role() {
    let app = test_app();
    assert_eq!(
        call(&app, "/admin", Some(&sign(&claims("admin")))).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "/admin", Some(&sign(&claims("user")))).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[derive(Deserialize)]
struct RoleDto {
    role: String,
}

#[tokio::test]
async fn client_decodes_json_and_errors() {
    let url = fake_service().await;
    let client = ServiceClient::new(http_client(), format!("{url}/"));

    let ok: RoleDto = client.get("/json").send_json().await.unwrap();
    assert_eq!(ok.role, "x");

    match client.get("/plain_error").send_empty().await {
        Err(ClientError::Api { status, message }) => {
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(message, "not json");
        }
        other => panic!("unexpected {other:?}"),
    }

    match client
        .post("/json_error")
        .json(&json!({ "token": "nope" }))
        .send_empty()
        .await
    {
        Err(ClientError::Api { status, message }) => {
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(message, "Token invalid");
        }
        other => panic!("unexpected {other:?}"),
    }

    let unreachable = ServiceClient::new(http_client(), "http://127.0.0.1:1");
    assert!(matches!(
        unreachable.get("/").send_empty().await,
        Err(ClientError::Request(_))
    ));
}
