use std::time::Duration;

use crate::smart_device_dto::{activation::ActivateRequestDto, endpoints, read::ReadResponseDto};

use super::{ClientResult, ServiceClient};

/// Typed client for smart devices.
///
/// Unlike the service clients, every device has its own address, so the
/// address is passed per call. All calls share one pooled `reqwest::Client`.
#[derive(Clone, Debug)]
pub struct SmartDeviceClient {
    http: reqwest::Client,
}

impl SmartDeviceClient {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    fn device(&self, address: &str) -> ServiceClient {
        ServiceClient::new(self.http.clone(), address)
    }

    /// Reads the device's current value, giving up after `timeout`.
    pub async fn read(&self, address: &str, timeout: Duration) -> ClientResult<ReadResponseDto> {
        self.device(address)
            .get(endpoints::READ)
            .timeout(timeout)
            .send_json()
            .await
    }

    /// Raw device config JSON, passed through unchanged.
    pub async fn get_config(&self, address: &str) -> ClientResult<String> {
        self.device(address)
            .get(endpoints::CONFIG)
            .send_text()
            .await
    }

    /// Sends a new config to the device and returns its raw response.
    pub async fn update_config(
        &self,
        address: &str,
        body: &serde_json::Value,
    ) -> ClientResult<String> {
        self.device(address)
            .post(endpoints::CONFIG)
            .json(body)
            .send_text()
            .await
    }

    /// Raw device status JSON, passed through unchanged.
    pub async fn get_status(&self, address: &str) -> ClientResult<String> {
        self.device(address)
            .get(endpoints::STATUS)
            .send_text()
            .await
    }

    /// Tells the device where the scripting API is and which token to use.
    pub async fn activate(
        &self,
        address: &str,
        request: &ActivateRequestDto,
    ) -> ClientResult<String> {
        self.device(address)
            .post(endpoints::ACTIVATE)
            .json(request)
            .send_text()
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{
        Json, Router,
        http::StatusCode,
        routing::{get, post},
    };

    use super::SmartDeviceClient;
    use crate::{
        service_client::{ClientError, http_client},
        smart_device_dto::{Type, activation::ActivateRequestDto, endpoints},
    };

    async fn serve(router: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        address
    }

    #[tokio::test]
    async fn calls_device_endpoints() {
        let address = serve(
            Router::new()
                .route(
                    endpoints::READ,
                    get(|| async { Json(serde_json::json!({ "data": { "Number": 1.5 } })) }),
                )
                .route(endpoints::CONFIG, get(|| async { r#"{"a":1}"# }))
                .route(
                    endpoints::CONFIG,
                    post(|Json(body): Json<serde_json::Value>| async move { body.to_string() }),
                )
                .route(endpoints::STATUS, get(|| async { r#"{"ok":true}"# }))
                .route(
                    endpoints::ACTIVATE,
                    post(|Json(body): Json<ActivateRequestDto>| async move { body.token }),
                ),
        )
        .await;
        let client = SmartDeviceClient::new(http_client());

        let read = client.read(&address, Duration::from_secs(4)).await.unwrap();
        assert!(matches!(read.data, Type::Number(v) if v == 1.5));
        assert_eq!(client.get_config(&address).await.unwrap(), r#"{"a":1}"#);
        assert_eq!(
            client
                .update_config(&address, &serde_json::json!({ "b": 2 }))
                .await
                .unwrap(),
            r#"{"b":2}"#
        );
        assert_eq!(client.get_status(&address).await.unwrap(), r#"{"ok":true}"#);
        let activate = ActivateRequestDto {
            url: "http://scripting".into(),
            token: "secret".into(),
        };
        assert_eq!(
            client.activate(&address, &activate).await.unwrap(),
            "secret"
        );
    }

    #[tokio::test]
    async fn maps_device_errors() {
        let address = serve(Router::new().route(
            endpoints::STATUS,
            get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "broken") }),
        ))
        .await;
        let client = SmartDeviceClient::new(http_client());

        let err = client.get_status(&address).await.unwrap_err();
        assert_eq!(err.api_status(), Some(StatusCode::INTERNAL_SERVER_ERROR));

        // Nothing listens on port 1.
        let err = client.get_status("http://127.0.0.1:1").await.unwrap_err();
        assert!(matches!(err, ClientError::Request(_)));
    }
}
