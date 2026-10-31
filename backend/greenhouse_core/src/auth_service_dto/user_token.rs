use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "auth_service/")
)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    pub user_name: String,
    pub role: String,
}
