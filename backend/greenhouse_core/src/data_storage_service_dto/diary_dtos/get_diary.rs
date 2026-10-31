use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

use super::get_diary_entry::DiaryEntryResponseDto;

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "data_storage_service/diary/")
)]
pub struct GetDiaryResponseDto {
    pub entries: Vec<DiaryEntryResponseDto>,
}
