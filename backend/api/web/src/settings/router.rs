use crate::AppState;
use crate::helper::error::HttpResult;
use axum::routing::post;
use axum::{
    Json, Router,
    extract::State,
    response::{IntoResponse, Response},
};
use greenhouse_core::{
    auth::{Admin, RequireRole},
    auth_service_dto::generate_one_time_token::GenerateOneTimeTokenRequestDto,
    service_client::AuthServiceClient,
};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/generate_one_time_token", post(generate_one_time_token))
        .with_state(state)
}

#[axum::debug_handler(state = AppState)]
pub(crate) async fn generate_one_time_token(
    _: RequireRole<Admin>,
    State(auth): State<AuthServiceClient>,
    Json(request): Json<GenerateOneTimeTokenRequestDto>,
) -> HttpResult<Response> {
    let token = auth.generate_one_time_token(&request.username).await?;
    Ok(Json(token).into_response())
}
