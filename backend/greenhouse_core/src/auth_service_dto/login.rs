use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct LoginRequestDto {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct LoginResponseDto {
    pub token: String,
    pub token_type: String,
}
