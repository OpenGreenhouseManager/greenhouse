use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/diary/")
)]
pub struct DiaryEntryResponseDto {
    pub id: String,
    pub date: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
