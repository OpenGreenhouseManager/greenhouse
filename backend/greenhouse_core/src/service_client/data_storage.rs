use std::time::Duration;

use uuid::Uuid;

use crate::data_storage_service_dto::{
    alert_dto::{
        self,
        alert::{AlertDto, AlertsDto},
        get_aggrigated_alert::AggrigatedAlertsDto,
        post_create_alert::CreateAlertDto,
        query::{AlertQuery, IntervalQuery},
    },
    diary_dtos::{
        self, get_diary::GetDiaryResponseDto, get_diary_entry::DiaryEntryResponseDto,
        post_diary_entry::PostDiaryEntryDtoRequest, post_diary_tag::PostDiaryTagDtoRequest,
        put_diary_entry::PutDiaryEntryDtoRequest,
    },
};

use super::{ClientResult, ServiceClient};

const DIARY: &str = diary_dtos::endpoints::DIARY;
const ALERT: &str = alert_dto::endpoints::ALERT;
const TAG_TIMEOUT: Duration = Duration::from_secs(10);

/// Typed client for `data_storage_service` (diary and alerts).
#[derive(Clone, Debug)]
pub struct DataStorageServiceClient(ServiceClient);

impl DataStorageServiceClient {
    pub fn new(http: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self(ServiceClient::new(http, base_url))
    }

    // region: --- Diary

    pub async fn create_diary_entry(&self, entry: &PostDiaryEntryDtoRequest) -> ClientResult<()> {
        self.0.post(DIARY).json(entry).send_empty().await
    }

    pub async fn update_diary_entry(
        &self,
        id: Uuid,
        update: &PutDiaryEntryDtoRequest,
    ) -> ClientResult<()> {
        self.0
            .put(&format!("{DIARY}/{id}"))
            .json(update)
            .send_empty()
            .await
    }

    pub async fn get_diary_entry(&self, id: Uuid) -> ClientResult<DiaryEntryResponseDto> {
        self.0.get(&format!("{DIARY}/{id}")).send_json().await
    }

    pub async fn get_diary(&self, start: &str, end: &str) -> ClientResult<GetDiaryResponseDto> {
        self.0
            .get(&format!("{DIARY}/{start}/{end}"))
            .send_json()
            .await
    }

    pub async fn add_tag(&self, entry_id: Uuid, body: &PostDiaryTagDtoRequest) -> ClientResult<()> {
        self.0
            .post(&format!("{DIARY}/{entry_id}/tags"))
            .timeout(TAG_TIMEOUT)
            .json(body)
            .send_empty()
            .await
    }

    pub async fn remove_tag(&self, entry_id: Uuid, tag_name: &str) -> ClientResult<()> {
        let tag = urlencoding::encode(tag_name);
        self.0
            .delete(&format!("{DIARY}/{entry_id}/tags/{tag}"))
            .timeout(TAG_TIMEOUT)
            .send_empty()
            .await
    }

    pub async fn search_by_tag(&self, tag_name: &str) -> ClientResult<GetDiaryResponseDto> {
        let tag = urlencoding::encode(tag_name);
        self.0
            .get(&format!("{DIARY}/tags/{tag}"))
            .timeout(TAG_TIMEOUT)
            .send_json()
            .await
    }

    // endregion: --- Diary

    // region: --- Alert

    pub async fn create_alert(&self, alert: &CreateAlertDto) -> ClientResult<AlertDto> {
        self.0.post(ALERT).json(alert).send_json().await
    }

    pub async fn get_filtered_alerts(&self, query: &AlertQuery) -> ClientResult<AlertsDto> {
        self.0
            .get(&format!("{ALERT}/filter"))
            .query(query)
            .send_json()
            .await
    }

    pub async fn get_alert_subset(
        &self,
        query: &IntervalQuery,
    ) -> ClientResult<AggrigatedAlertsDto> {
        self.0.get(ALERT).query(query).send_json().await
    }

    // endregion: --- Alert
}
