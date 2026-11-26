use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::Severity, post_create_alert::CreateAlertDto,
};
use greenhouse_core::device_service_dto::key::DeviceKeyDto;
use reqwest::StatusCode;
use test_helper::TestContext;
#[allow(dead_code)]
mod test_helper;

const DEVICE_SERVICE: &str = "http://localhost:3003";

async fn check_key(client: &reqwest::Client, key: &str) -> StatusCode {
    client
        .post(format!("{DEVICE_SERVICE}/key/check"))
        .json(&DeviceKeyDto {
            key: key.to_string(),
        })
        .send()
        .await
        .unwrap()
        .status()
}

/// Status of an alert posted to the scripting API with `key` as auth token.
async fn post_alert(client: &reqwest::Client, key: &str) -> StatusCode {
    client
        .post("http://localhost:3100/alert")
        .header("Cookie", format!("auth-token={key}"))
        .json(&CreateAlertDto {
            severity: Severity::Info,
            identifier: String::from("key-test"),
            value: None,
            note: None,
            datasource_id: uuid::Uuid::new_v4().to_string(),
        })
        .send()
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn test_device_scripting_key_lifecycle() {
    let mut context = TestContext::new();
    context.start_all_services().await;

    let client = reqwest::Client::new();

    // Unknown key is rejected
    let status = check_key(&client, &uuid::Uuid::new_v4().to_string()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // ... and the scripting API turns it away
    let response = client
        .post("http://localhost:3100/alert")
        .header("Cookie", format!("auth-token={}", uuid::Uuid::new_v4()))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Creating a device activates it with a valid key
    let device_id = test_helper::create_fake_device().await;
    let first_key = test_helper::last_activation_key();
    assert_eq!(check_key(&client, &first_key).await, StatusCode::OK);
    assert!(post_alert(&client, &first_key).await.is_success());

    // Activating again hands out a new key and revokes the old one
    let response = client
        .put(format!("{DEVICE_SERVICE}/{device_id}/activate"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to activate device");
    let second_key = test_helper::last_activation_key();
    assert_ne!(first_key, second_key);
    assert_eq!(check_key(&client, &second_key).await, StatusCode::OK);
    assert_eq!(
        check_key(&client, &first_key).await,
        StatusCode::UNAUTHORIZED
    );
    assert!(post_alert(&client, &second_key).await.is_success());
    assert_eq!(post_alert(&client, &first_key).await, StatusCode::FORBIDDEN);

    // Deleting the device revokes its key
    let response = client
        .delete(format!("{DEVICE_SERVICE}/{device_id}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to delete device");
    assert_eq!(
        check_key(&client, &second_key).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post_alert(&client, &second_key).await,
        StatusCode::FORBIDDEN
    );

    context.stop().await;
}
