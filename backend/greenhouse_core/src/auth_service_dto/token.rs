use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct TokenRequestDto {
    pub token: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct TokenResponseDto {
    pub role: String,
}
