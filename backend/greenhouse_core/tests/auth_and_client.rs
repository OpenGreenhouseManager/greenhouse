//! Tests for `AuthUser`, `RequireRole` and `ServiceClient` against a fake
//! service on a local port.
#![cfg(all(feature = "auth", feature = "service_client"))]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Json, Router,
    body::Body,
    extract::{FromRef, State},
    http::{Request, StatusCode, header},
    middleware::from_extractor_with_state,
    routing::{get, post},
};
use greenhouse_core::{
    auth::{Admin, AuthUser, AuthVerifier, RequireRole},
    auth_service_dto::token::{TokenRequestDto, TokenResponseDto},
    service_client::{AuthServiceClient, ClientError, ServiceClient, http_client},
};
use serde_json::json;
use tower::ServiceExt;
use tower_cookies::CookieManagerLayer;

/// Fake auth_service: token "admin-token" is admin, "user-token" is user,
/// "broken" makes the service fail, anything else is rejected.
async fn fake_auth_service(calls: Arc<AtomicUsize>) -> String {
    let app = Router::new()
        .route(
            "/check_token",
            post(
                |State(calls): State<Arc<AtomicUsize>>, Json(req): Json<TokenRequestDto>| async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    match req.token.as_str() {
                        "admin-token" => Ok(Json(TokenResponseDto { role: "admin".into() })),
                        "user-token" => Ok(Json(TokenResponseDto { role: "user".into() })),
                        "broken" => Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": "db down" })),
                        )),
                        _ => Err((
                            StatusCode::UNAUTHORIZED,
                            Json(json!({ "error": "Token invalid" })),
                        )),
                    }
                },
            ),
        )
        .route("/json", get(|| async { Json(json!({ "role": "x" })) }))
        .route(
            "/plain_error",
            get(|| async { (StatusCode::BAD_REQUEST, "not json") }),
        )
        .with_state(calls);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[derive(Clone, FromRef)]
struct TestState {
    verifier: AuthVerifier,
}

async fn test_app() -> (Router, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let url = fake_auth_service(calls.clone()).await;
    let state = TestState {
        verifier: AuthVerifier::AuthService(AuthServiceClient::new(http_client(), url)),
    };
    let app = Router::new()
        .route("/me", get(|user: AuthUser| async move { user.role }))
        .route("/admin", get(|_: RequireRole<Admin>| async { "ok" }))
        .with_state(state.clone())
        .layer(from_extractor_with_state::<AuthUser, _>(state))
        .layer(CookieManagerLayer::new());
    (app, calls)
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
    let (app, calls) = test_app().await;
    let (status, _, _) = call(&app, "/me", None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_token_is_forbidden_and_cookie_removed() {
    let (app, _) = test_app().await;
    let (status, _, removes_cookie) = call(&app, "/me", Some("forged")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(removes_cookie);
}

#[tokio::test]
async fn valid_token_is_verified_once_per_request() {
    let (app, calls) = test_app().await;
    let (status, body, _) = call(&app, "/me", Some("user-token")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "user");
    // The layer and the handler both extract AuthUser; the second hit is cached.
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn verifier_failure_is_unavailable_not_logout() {
    let (app, _) = test_app().await;
    let (status, _, removes_cookie) = call(&app, "/me", Some("broken")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(!removes_cookie);
}

#[tokio::test]
async fn require_role_checks_role() {
    let (app, _) = test_app().await;
    assert_eq!(
        call(&app, "/admin", Some("admin-token")).await.0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "/admin", Some("user-token")).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn client_decodes_json_and_errors() {
    let url = fake_auth_service(Arc::default()).await;
    let client = ServiceClient::new(http_client(), format!("{url}/"));

    let ok: TokenResponseDto = client.get("/json").send_json().await.unwrap();
    assert_eq!(ok.role, "x");

    match client.get("/plain_error").send_empty().await {
        Err(ClientError::Api { status, message }) => {
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(message, "not json");
        }
        other => panic!("unexpected {other:?}"),
    }

    match client
        .post("/check_token")
        .json(&TokenRequestDto {
            token: "nope".into(),
            token_type: "Bearer".into(),
        })
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
