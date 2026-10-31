use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct RegisterRequestDto {
    pub username: String,
    pub password: String,
    pub one_time_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct RegisterResponseDto {
    pub token: String,
    pub token_type: String,
}
