use greenhouse_core::auth_service_dto::generate_one_time_token::GenerateOneTimeTokenRequestDto;
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_login() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    assert!(!token.is_empty(), "Login token should not be empty");
    context.stop().await;
}

#[tokio::test]
async fn test_generate_one_time_token() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let one_time_token = get_one_time_token(token, String::from("normaluser")).await;

    assert!(
        !one_time_token.is_empty(),
        "One-time token should not be empty"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_register() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    let one_time_token = get_one_time_token(token, username.clone()).await;

    let user_token = register_user(username.clone(), password.clone(), one_time_token).await;

    assert!(
        !user_token.is_empty(),
        "User token should not be empty after registration"
    );

    context.stop().await;
}

#[tokio::test]
async fn login_with_registered_user() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Register the user first
    let one_time_token =
        get_one_time_token(test_helper::admin_login().await, username.clone()).await;
    let _ = register_user(username.clone(), password.clone(), one_time_token).await;

    // Now login with the registered user
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto { username, password })
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success(), "Login should be successful");

    let login_response: greenhouse_core::auth_service_dto::login::LoginResponseDto =
        response.json().await.unwrap();

    assert!(
        !login_response.token.is_empty(),
        "Login token should not be empty"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let username = String::from("invaliduser");
    let password = String::from("invalidpassword");

    // Attempt to login with invalid credentials
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto { username, password })
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Login should fail with invalid credentials"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_register_with_invalid_one_time_token() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Attempt to register with an invalid one-time token
    let invalid_one_time_token = String::from("invalidtoken");
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/register")
        .json(
            &greenhouse_core::auth_service_dto::register::RegisterRequestDto {
                username: username.clone(),
                password: password.clone(),
                one_time_token: invalid_one_time_token,
            },
        )
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Registration should fail with invalid one-time token"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_register_with_existing_username() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Register the user first
    let one_time_token = get_one_time_token(token, username.clone()).await;
    let user_token =
        register_user(username.clone(), password.clone(), one_time_token.clone()).await;

    assert!(
        !user_token.is_empty(),
        "User token should not be empty after registration"
    );

    // Attempt to register again with the same username
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/register")
        .json(
            &greenhouse_core::auth_service_dto::register::RegisterRequestDto {
                username: username.clone(),
                password: password.clone(),
                one_time_token: one_time_token.clone(),
            },
        )
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Registration should fail with existing username"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_registered_login_with_wrong_password() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let username = String::from("normaluser");
    let password = String::from("normalpassword");

    // Register the user first
    let one_time_token =
        get_one_time_token(test_helper::admin_login().await, username.clone()).await;
    let user_token = register_user(username.clone(), password.clone(), one_time_token).await;

    assert!(
        !user_token.is_empty(),
        "User token should not be empty after registration"
    );

    // Now attempt to login with the registered user but wrong password
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/api/login")
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto {
            username,
            password: String::from("wrongpassword"),
        })
        .send()
        .await
        .unwrap();

    assert!(
        !response.status().is_success(),
        "Login should fail with wrong password"
    );

    context.stop().await;
}

async fn get_one_time_token(admin_jwt: String, username: String) -> String {
    let client = reqwest::Client::new();
    let result = client
        .post("http://localhost:3000/api/settings/generate_one_time_token")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={admin_jwt}"))
        .json(&GenerateOneTimeTokenRequestDto { username })
        .send()
        .await
        .unwrap();

    assert!(
        result.status().is_success(),
        "API call failed with status: {}",
        result.status()
    );

    result.json().await.unwrap()
}

async fn register_user(username: String, password: String, one_time_token: String) -> String {
    let client = reqwest::Client::new();
    let result = client
        .post("http://localhost:3000/api/register")
        .json(
            &greenhouse_core::auth_service_dto::register::RegisterRequestDto {
                username,
                password,
                one_time_token,
            },
        )
        .send()
        .await
        .unwrap();

    assert!(
        result.status().is_success(),
        "API call failed with status: {}",
        result.status()
    );

    result
        .json::<greenhouse_core::auth_service_dto::login::LoginResponseDto>()
        .await
        .unwrap()
        .token
}

async fn refresh(refresh_token: Option<&str>) -> reqwest::Response {
    let mut request = reqwest::Client::new().post("http://localhost:3000/api/session/refresh");
    if let Some(token) = refresh_token {
        request = request.header("Cookie", format!("refresh-token={token}"));
    }
    request.send().await.unwrap()
}

#[tokio::test]
async fn refresh_issues_working_access_token() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    test_helper::register_admin().await;
    let (_, refresh_token) = test_helper::api_login_session().await;

    let response = refresh(Some(&refresh_token)).await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let access = test_helper::set_cookie(&response, "auth-token").expect("no auth cookie");
    assert_eq!(
        test_helper::set_cookie(&response, "refresh-token").as_deref(),
        Some(refresh_token.as_str())
    );

    // The refreshed token passes the local check and the admin role check.
    let one_time_token = get_one_time_token(access, String::from("normaluser")).await;
    assert!(!one_time_token.is_empty());

    context.stop().await;
}

#[tokio::test]
async fn new_login_ends_old_session_at_refresh() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    test_helper::register_admin().await;
    let (_, old_refresh) = test_helper::api_login_session().await;
    let (_, new_refresh) = test_helper::api_login_session().await;

    assert_eq!(
        refresh(Some(&old_refresh)).await.status(),
        reqwest::StatusCode::FORBIDDEN
    );
    assert_eq!(
        refresh(Some(&new_refresh)).await.status(),
        reqwest::StatusCode::OK
    );

    context.stop().await;
}

#[tokio::test]
async fn logout_revokes_session_and_expires_cookies() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    test_helper::register_admin().await;
    let (_, refresh_token) = test_helper::api_login_session().await;

    // The refresh cookie is scoped to /api/session, so logout gets it even
    // without a valid access cookie.
    let response = reqwest::Client::new()
        .post("http://localhost:3000/api/session/logout")
        .header(
            "Cookie",
            format!("auth-token=garbage; refresh-token={refresh_token}"),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let removals: Vec<String> = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    assert!(
        removals
            .iter()
            .any(|c| c.starts_with("refresh-token=;") && c.contains("Path=/api/session"))
    );
    assert!(removals.iter().any(|c| c.starts_with("auth-token=;")));

    assert_eq!(
        refresh(Some(&refresh_token)).await.status(),
        reqwest::StatusCode::FORBIDDEN
    );

    context.stop().await;
}

#[tokio::test]
async fn failed_login_keeps_existing_session_cookies() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    test_helper::register_admin().await;
    let (access, refresh_token) = test_helper::api_login_session().await;

    // A failed login (say, in another tab) must not expire the cookies of the
    // live session, or logout could no longer revoke it.
    let response = reqwest::Client::new()
        .post("http://localhost:3000/api/login")
        .header(
            "Cookie",
            format!("auth-token={access}; refresh-token={refresh_token}"),
        )
        .json(&greenhouse_core::auth_service_dto::login::LoginRequestDto {
            username: test_helper::TEST_USERNAME.to_string(),
            password: "wrong-password".to_string(),
        })
        .send()
        .await
        .unwrap();
    assert!(!response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .count(),
        0
    );

    // The session still works and logout still revokes it.
    assert_eq!(
        refresh(Some(&refresh_token)).await.status(),
        reqwest::StatusCode::OK
    );
    let response = reqwest::Client::new()
        .post("http://localhost:3000/api/session/logout")
        .header("Cookie", format!("refresh-token={refresh_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        refresh(Some(&refresh_token)).await.status(),
        reqwest::StatusCode::FORBIDDEN
    );

    context.stop().await;
}

#[tokio::test]
async fn refresh_without_or_with_unknown_cookie_is_forbidden() {
    let mut context = TestContext::new();
    context.start_all_services().await;

    assert_eq!(refresh(None).await.status(), reqwest::StatusCode::FORBIDDEN);
    assert_eq!(
        refresh(Some("not-a-session")).await.status(),
        reqwest::StatusCode::FORBIDDEN
    );

    context.stop().await;
}

#[tokio::test]
async fn forged_access_token_is_forbidden() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    // Swap the payload for one claiming another role; the signature no
    // longer matches.
    let mut parts: Vec<&str> = token.split('.').collect();
    let payload = "eyJpYXQiOjAsImV4cCI6OTk5OTk5OTk5OSwidXNlcl9uYW1lIjoieCIsInJvbGUiOiJhZG1pbiIsInNpZCI6InMiLCJpc3MiOiJncmVlbmhvdXNlLWF1dGgiLCJhdWQiOiJncmVlbmhvdXNlLXdlYiJ9";
    parts[1] = payload;
    let forged = parts.join(".");

    let response = reqwest::Client::new()
        .post("http://localhost:3000/api/settings/generate_one_time_token")
        .header("Cookie", format!("auth-token={forged}"))
        .json(&GenerateOneTimeTokenRequestDto {
            username: String::from("normaluser"),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);

    context.stop().await;
}

#[tokio::test]
async fn preferences_follow_the_verified_user() {
    use greenhouse_core::auth_service_dto::user_preferences::{
        UserPreferencesRequestDto, UserPreferencesResponseDto,
    };

    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3000/api/user/preferences")
        .header("Cookie", format!("auth-token={token}"))
        .json(&UserPreferencesRequestDto {
            dashboard_preferences: serde_json::json!({ "cards": [1] }),
            alert_preferences: serde_json::json!({}),
        })
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "{}", response.status());

    let response = client
        .get("http://localhost:3000/api/user/preferences")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "{}", response.status());
    let prefs: UserPreferencesResponseDto = response.json().await.unwrap();
    assert_eq!(
        prefs.dashboard_preferences,
        serde_json::json!({ "cards": [1] })
    );

    context.stop().await;
}
