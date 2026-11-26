use crate::AppState;
use crate::helper::error::HttpResult;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary::GetDiaryResponseDto;
use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary_entry::DiaryEntryResponseDto;
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_entry::PostDiaryEntryDtoRequest;
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_tag::PostDiaryTagDtoRequest;
use greenhouse_core::data_storage_service_dto::diary_dtos::put_diary_entry::PutDiaryEntryDtoRequest;
use greenhouse_core::service_client::DataStorageServiceClient;
use uuid::Uuid;

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(create_diary_entry))
        .route("/{id}", put(update_diary_entry))
        .route("/{id}", get(get_diary_entry))
        .route("/{start}/{end}", get(get_diary))
        .route("/{id}/tags", post(add_tag_to_entry))
        .route("/{id}/tags/{tag_name}", delete(remove_tag_from_entry))
        .route("/tags/{tag_name}", get(search_entries_by_tag))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn create_diary_entry(
    State(data_storage): State<DataStorageServiceClient>,
    Json(entry): Json<PostDiaryEntryDtoRequest>,
) -> HttpResult<()> {
    data_storage.create_diary_entry(&entry).await?;
    Ok(())
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn update_diary_entry(
    State(data_storage): State<DataStorageServiceClient>,
    Path(id): Path<Uuid>,
    Json(update): Json<PutDiaryEntryDtoRequest>,
) -> HttpResult<()> {
    data_storage.update_diary_entry(id, &update).await?;
    Ok(())
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_diary_entry(
    State(data_storage): State<DataStorageServiceClient>,
    Path(id): Path<Uuid>,
) -> HttpResult<DiaryEntryResponseDto> {
    Ok(data_storage.get_diary_entry(id).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn get_diary(
    State(data_storage): State<DataStorageServiceClient>,
    Path((start, end)): Path<(String, String)>,
) -> HttpResult<GetDiaryResponseDto> {
    Ok(data_storage.get_diary(&start, &end).await?)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn add_tag_to_entry(
    State(data_storage): State<DataStorageServiceClient>,
    Path(id): Path<Uuid>,
    Json(body): Json<PostDiaryTagDtoRequest>,
) -> HttpResult<StatusCode> {
    data_storage.add_tag(id, &body).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn remove_tag_from_entry(
    State(data_storage): State<DataStorageServiceClient>,
    Path((id, tag_name)): Path<(Uuid, String)>,
) -> HttpResult<StatusCode> {
    data_storage.remove_tag(id, &tag_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn search_entries_by_tag(
    State(data_storage): State<DataStorageServiceClient>,
    Path(tag_name): Path<String>,
) -> HttpResult<GetDiaryResponseDto> {
    Ok(data_storage.search_by_tag(&tag_name).await?)
}
