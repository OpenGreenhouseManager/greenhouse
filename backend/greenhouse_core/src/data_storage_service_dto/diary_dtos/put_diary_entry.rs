use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/diary/")
)]
pub struct PutDiaryEntryDtoRequest {
    pub date: String,
    pub title: String,
    pub content: String,
}
