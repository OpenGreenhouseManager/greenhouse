use greenhouse_core::scripting_service_dto::token::TokenDto;
use test_helper::TestContext;
#[allow(dead_code)]
mod test_helper;

const SCRIPTING_SERVICE: &str = "http://localhost:3004";

async fn check_token(client: &reqwest::Client, token: &str) -> reqwest::StatusCode {
    client
        .post(format!("{SCRIPTING_SERVICE}/token/check"))
        .json(&TokenDto {
            token: token.to_string(),
        })
        .send()
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn test_check_scripting_key() {
    let mut context = TestContext::new();
    context.start_all_services().await;

    let client = reqwest::Client::new();

    // Unknown key is rejected
    let status = check_token(&client, &uuid::Uuid::new_v4().to_string()).await;
    assert_eq!(status, reqwest::StatusCode::UNAUTHORIZED);

    // ... and the scripting API turns it away
    let response = client
        .post("http://localhost:3100/alert")
        .header("Cookie", format!("auth-token={}", uuid::Uuid::new_v4()))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);

    // Generated key is accepted
    let response = client
        .post(format!("{SCRIPTING_SERVICE}/token"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to generate scripting key"
    );
    let TokenDto { token } = response.json().await.unwrap();
    assert_eq!(check_token(&client, &token).await, reqwest::StatusCode::OK);

    // Deleted key is rejected again
    let response = client
        .delete(format!("{SCRIPTING_SERVICE}/token"))
        .json(&TokenDto {
            token: token.clone(),
        })
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to delete scripting key"
    );
    assert_eq!(
        check_token(&client, &token).await,
        reqwest::StatusCode::UNAUTHORIZED
    );

    context.stop().await;
}
