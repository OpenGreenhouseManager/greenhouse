use greenhouse_core::data_storage_service_dto::alert_dto::alert::{
    AlertKind, AlertsDto, EndReason, Severity,
};
use greenhouse_core::data_storage_service_dto::alert_dto::get_aggrigated_alert::AggrigatedAlertsDto;
use greenhouse_core::data_storage_service_dto::alert_dto::post_create_alert::CreateAlertDto;
use greenhouse_core::data_storage_service_dto::alert_dto::post_stop_alert::StopAlertDto;
use greenhouse_core::data_storage_service_dto::alert_dto::query::IntervalQuery;
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_create_and_filter_alert() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let scripting_token = test_helper::scripting_login().await;
    let web_token = test_helper::admin_login().await;
    // Create alert
    let client = reqwest::Client::new();
    let create_alert = CreateAlertDto {
        severity: Severity::Error,
        identifier: String::from("temperature"),
        value: Some(String::from("42.0")),
        note: Some(String::from("High temperature detected")),
        datasource_id: String::from(uuid::Uuid::new_v4()),
        kind: AlertKind::OneTime,
        keep_alive_secs: None,
    };
    let response = client
        .post("http://localhost:3100/alert")
        .json(&create_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={scripting_token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to create alert: {}",
        response.status()
    );

    // Filter alerts
    let response = client
        .get("http://localhost:3000/api/alert/filter?alert_type=temperature")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={web_token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to filter alerts");
    let alerts: AlertsDto = response.json().await.unwrap();
    assert!(!alerts.alerts.is_empty(), "No alerts returned");

    context.stop().await;
}

#[tokio::test]
async fn test_alert_subset() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let scripting_token = test_helper::scripting_login().await;
    let web_token = test_helper::admin_login().await;

    // Create alert
    let client = reqwest::Client::new();
    let create_alert = CreateAlertDto {
        severity: Severity::Warning,
        identifier: String::from("humidity"),
        value: Some(String::from("15.5")),
        note: Some(String::from("Low humidity detected")),
        datasource_id: String::from(uuid::Uuid::new_v4()),
        kind: AlertKind::OneTime,
        keep_alive_secs: None,
    };
    let response = client
        .post("http://localhost:3100/alert")
        .json(&create_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={scripting_token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to create alert");

    let q = IntervalQuery {
        start: Some(chrono::Utc::now() - chrono::Duration::days(1)),
        end: Some(chrono::Utc::now()),
    };

    // Get alert subset by interval
    let response = client
        .get("http://localhost:3000/api/alert")
        .query(&q)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={web_token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to get alert subset");
    let alerts: AggrigatedAlertsDto = response.json().await.unwrap();
    assert!(!alerts.alerts.is_empty(), "No alerts returned in subset");

    context.stop().await;
}

#[tokio::test]
async fn test_create_alert_missing_fields() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Missing required field: identifier
    let client = reqwest::Client::new();
    let invalid_alert = serde_json::json!({
        "severity": "Error",
        // "identifier" is missing
        "value": "100.0",
        "note": "Missing identifier",
        "datasource_id": uuid::Uuid::new_v4().to_string()
    });
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&invalid_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        !response.status().is_success(),
        "Should fail to create alert with missing fields"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_create_alert_invalid_severity() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Invalid severity value
    let client = reqwest::Client::new();
    let invalid_alert = serde_json::json!({
        "severity": "NotASeverity",
        "identifier": "temperature",
        "value": "100.0",
        "note": "Invalid severity",
        "datasource_id": uuid::Uuid::new_v4().to_string()
    });
    let response = client
        .post("http://localhost:3000/api/alert")
        .json(&invalid_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        !response.status().is_success(),
        "Should fail to create alert with invalid severity"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_create_alert_unauthorized() {
    let mut context = TestContext::new();
    context.start_all_services().await;

    // No auth token
    let client = reqwest::Client::new();
    let create_alert = serde_json::json!({
        "severity": "Error",
        "identifier": "unauthorized",
        "value": "1.0",
        "note": "No auth",
        "datasource_id": uuid::Uuid::new_v4().to_string()
    });
    let response = client
        .post("http://localhost:3100/alert")
        .json(&create_alert)
        .header("Access-Control-Allow-Credentials", "true")
        .send()
        .await
        .unwrap();
    assert!(
        !response.status().is_success(),
        "Should not allow creating alert without authentication"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_filter_alerts_no_results() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Query for an identifier that does not exist
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/api/alert/filter?identifier=doesnotexist")
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Filter request should succeed"
    );
    let alerts: AlertsDto = response.json().await.unwrap();
    assert!(
        alerts.alerts.is_empty(),
        "No alerts should be returned for unknown identifier"
    );

    context.stop().await;
}

const IDENTIFIER: &str = "temperature_low";

fn concurrent(datasource_id: &str, value: &str, keep_alive_secs: Option<u32>) -> CreateAlertDto {
    CreateAlertDto {
        severity: Severity::Warning,
        identifier: String::from(IDENTIFIER),
        value: Some(value.to_string()),
        note: None,
        datasource_id: datasource_id.to_string(),
        kind: AlertKind::Concurrent,
        keep_alive_secs,
    }
}

async fn fire(token: &str, alert: &CreateAlertDto) -> reqwest::Response {
    reqwest::Client::new()
        .post("http://localhost:3100/alert")
        .json(alert)
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap()
}

async fn stop(token: &str, datasource_id: &str) -> reqwest::Response {
    reqwest::Client::new()
        .post("http://localhost:3100/alert/stop")
        .json(&StopAlertDto {
            identifier: String::from(IDENTIFIER),
            datasource_id: datasource_id.to_string(),
            note: Some(String::from("recovered")),
        })
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap()
}

async fn filter(token: &str, datasource_id: &str, open: Option<bool>) -> AlertsDto {
    let mut url = format!("http://localhost:3000/api/alert/filter?datasource_id={datasource_id}");
    if let Some(open) = open {
        url.push_str(&format!("&open={open}"));
    }
    reqwest::Client::new()
        .get(url)
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn test_concurrent_alert_lifecycle() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let scripting_token = test_helper::scripting_login().await;
    let web_token = test_helper::admin_login().await;
    let source = uuid::Uuid::new_v4().to_string();

    // Firing repeatedly keeps one open alert and extends it.
    for value in ["4.5", "3.9", "3.1"] {
        let response = fire(&scripting_token, &concurrent(&source, value, None)).await;
        assert!(response.status().is_success(), "Failed to fire alert");
    }
    let open = filter(&web_token, &source, Some(true)).await;
    assert_eq!(open.alerts.len(), 1, "Expected exactly one open alert");
    let alert = &open.alerts[0];
    assert_eq!(alert.kind, AlertKind::Concurrent);
    assert_eq!(alert.occurrences, 3);
    assert_eq!(alert.value, "3.1");
    assert!(alert.ended_at.is_none());
    assert!(alert.expires_at > alert.last_seen_at);

    // Stopping ends it; stopping again is a no-op.
    for _ in 0..2 {
        let response = stop(&scripting_token, &source).await;
        assert!(response.status().is_success(), "Failed to stop alert");
    }
    assert!(
        filter(&web_token, &source, Some(true))
            .await
            .alerts
            .is_empty()
    );
    let ended = filter(&web_token, &source, Some(false)).await;
    assert_eq!(ended.alerts.len(), 1, "Expected one ended alert");
    assert_eq!(ended.alerts[0].end_reason, Some(EndReason::Stopped));
    assert_eq!(ended.alerts[0].note.as_deref(), Some("recovered"));

    // Firing after a stop opens a new alert.
    let response = fire(&scripting_token, &concurrent(&source, "4.8", None)).await;
    assert!(response.status().is_success(), "Failed to fire alert");
    assert_eq!(filter(&web_token, &source, None).await.alerts.len(), 2);

    context.stop().await;
}

#[tokio::test]
async fn test_concurrent_alert_expires() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let scripting_token = test_helper::scripting_login().await;
    let web_token = test_helper::admin_login().await;
    let source = uuid::Uuid::new_v4().to_string();

    let response = fire(&scripting_token, &concurrent(&source, "4.5", Some(1))).await;
    assert!(response.status().is_success(), "Failed to fire alert");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Not fired again within its keep-alive: it ended on its own.
    assert!(
        filter(&web_token, &source, Some(true))
            .await
            .alerts
            .is_empty()
    );
    let ended = filter(&web_token, &source, Some(false)).await;
    assert_eq!(ended.alerts.len(), 1);
    assert_eq!(ended.alerts[0].end_reason, Some(EndReason::Expired));
    assert_eq!(
        ended.alerts[0].ended_at.as_ref(),
        Some(&ended.alerts[0].expires_at)
    );

    // The next fire opens a new alert instead of reviving the expired one.
    let response = fire(&scripting_token, &concurrent(&source, "4.1", None)).await;
    assert!(response.status().is_success(), "Failed to fire alert");
    let open = filter(&web_token, &source, Some(true)).await;
    assert_eq!(open.alerts.len(), 1);
    assert_eq!(open.alerts[0].occurrences, 1);

    context.stop().await;
}

#[tokio::test]
async fn test_one_time_alerts_are_not_merged() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let scripting_token = test_helper::scripting_login().await;
    let web_token = test_helper::admin_login().await;
    let source = uuid::Uuid::new_v4().to_string();

    let one_time = CreateAlertDto {
        kind: AlertKind::OneTime,
        ..concurrent(&source, "1", None)
    };
    for _ in 0..2 {
        let response = fire(&scripting_token, &one_time).await;
        assert!(response.status().is_success(), "Failed to fire alert");
    }
    let alerts = filter(&web_token, &source, None).await;
    assert_eq!(alerts.alerts.len(), 2, "One-time alerts must not be merged");
    for alert in &alerts.alerts {
        assert_eq!(alert.kind, AlertKind::OneTime);
        assert_eq!(alert.ended_at.as_ref(), Some(&alert.created_at));
        assert!(alert.end_reason.is_none());
    }

    context.stop().await;
}

#[tokio::test]
async fn test_invalid_keep_alive_is_rejected() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let scripting_token = test_helper::scripting_login().await;
    let source = uuid::Uuid::new_v4().to_string();

    for alert in [
        concurrent(&source, "1", Some(0)),
        concurrent(&source, "1", Some(86_401)),
        CreateAlertDto {
            kind: AlertKind::OneTime,
            ..concurrent(&source, "1", Some(60))
        },
    ] {
        let response = fire(&scripting_token, &alert).await;
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    }

    context.stop().await;
}
