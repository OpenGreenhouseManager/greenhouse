use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct UserPreferencesRequestDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct UserPreferencesResponseDto {
    pub dashboard_preferences: serde_json::Value,
    pub alert_preferences: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct SetPreferencesRequestDto {
    pub token: String,
    pub preferences: UserPreferencesRequestDto,
}
