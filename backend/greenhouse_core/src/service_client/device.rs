use uuid::Uuid;

use crate::device_service_dto::{
    batch_timeseries::{BatchTimeseriesRequestDto, BatchTimeseriesResponseDto},
    endpoints,
    get_device::{DeviceResponseDto, DevicesResponseDto},
    get_timeseries::GetTimeseriesDto,
    key::DeviceKeyDto,
    operations::OperationsDto,
    post_device::PostDeviceDtoRequest,
    put_device::PutDeviceDtoRequest,
    query::PromQuery,
};

use super::{ClientResult, ServiceClient};

/// Typed client for `device_service`.
#[derive(Clone, Debug)]
pub struct DeviceServiceClient(ServiceClient);

impl DeviceServiceClient {
    pub fn new(http: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self(ServiceClient::new(http, base_url))
    }

    pub async fn create_device(
        &self,
        request: &PostDeviceDtoRequest,
    ) -> ClientResult<DeviceResponseDto> {
        self.0.post("").json(request).send_json().await
    }

    pub async fn get_devices(&self) -> ClientResult<DevicesResponseDto> {
        self.0.get("").send_json().await
    }

    pub async fn get_device(&self, id: Uuid) -> ClientResult<DeviceResponseDto> {
        self.0.get(&format!("/{id}")).send_json().await
    }

    pub async fn update_device(
        &self,
        id: Uuid,
        request: &PutDeviceDtoRequest,
    ) -> ClientResult<DeviceResponseDto> {
        self.0
            .put(&format!("/{id}"))
            .json(request)
            .send_json()
            .await
    }

    pub async fn delete_device(&self, id: Uuid) -> ClientResult<()> {
        self.0.delete(&format!("/{id}")).send_empty().await
    }

    pub async fn activate_device(&self, id: Uuid) -> ClientResult<()> {
        self.0
            .put(&format!("/{id}/{}", endpoints::ACTIVATE))
            .send_empty()
            .await
    }

    /// Raw device config JSON, passed through unchanged.
    pub async fn get_device_config(&self, id: Uuid) -> ClientResult<String> {
        self.0
            .get(&format!("/{id}/{}", endpoints::CONFIG))
            .send_text()
            .await
    }

    pub async fn update_device_config(
        &self,
        id: Uuid,
        body: &serde_json::Value,
    ) -> ClientResult<()> {
        self.0
            .put(&format!("/{id}/{}", endpoints::CONFIG))
            .json(body)
            .send_empty()
            .await
    }

    /// Raw device status JSON, passed through unchanged.
    pub async fn get_device_status(&self, id: Uuid) -> ClientResult<String> {
        self.0
            .get(&format!("/{id}/{}", endpoints::STATUS))
            .send_text()
            .await
    }

    pub async fn get_device_timeseries(
        &self,
        id: Uuid,
        query: &PromQuery,
    ) -> ClientResult<GetTimeseriesDto> {
        self.0
            .get(&format!("/{id}/timeseries"))
            .query(query)
            .send_json()
            .await
    }

    pub async fn get_batch_timeseries(
        &self,
        request: &BatchTimeseriesRequestDto,
    ) -> ClientResult<BatchTimeseriesResponseDto> {
        self.0
            .post("/timeseries/batch")
            .json(request)
            .send_json()
            .await
    }

    pub async fn get_device_operations(&self, id: Uuid) -> ClientResult<OperationsDto> {
        self.0.get(&format!("/{id}/options")).send_json().await
    }

    /// Succeeds when `key` is a known device scripting key.
    pub async fn check_key(&self, key: &str) -> ClientResult<()> {
        self.0
            .post(endpoints::CHECK_KEY)
            .json(&DeviceKeyDto {
                key: key.to_string(),
            })
            .send_empty()
            .await
    }
}
