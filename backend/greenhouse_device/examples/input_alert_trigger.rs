//! Sends `low_alert` / `high_alert` when a written number is outside `min..=max`.

use greenhouse_core::{
    data_storage_service_dto::alert_dto::alert::Severity,
    smart_device_dto::{Type, config::TypeOption},
};
use greenhouse_device::{
    Arc, Config, DeviceBuilder, Platform, StatusCode,
    device_service::{AlertCreation, trigger_alert},
};
use serde::{Deserialize, Serialize};

const LOW_ALERT_IDENTIFIER: &str = "low_alert";
const HIGH_ALERT_IDENTIFIER: &str = "high_alert";

#[derive(Serialize, Deserialize, Clone)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/input_alert_trigger/config.json".to_string());
    log::info!("using config file: {config_path}");

    let default_config = Config {
        port: 6002,
        datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
        additional_config: ExampleDeviceConfig { min: 0, max: 10 },
        scripting_api: None,
    };

    DeviceBuilder::new(Platform::new(config_path), default_config)
        .write(TypeOption::Number, write_handler)
        .build()
        .unwrap()
        .run()
        .await
}

async fn write_handler(data: Type, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    let Type::Number(number) = data else {
        return StatusCode::BAD_REQUEST;
    };
    let identifier = if number > config.additional_config.max as f64 {
        HIGH_ALERT_IDENTIFIER
    } else if number < config.additional_config.min as f64 {
        LOW_ALERT_IDENTIFIER
    } else {
        return StatusCode::OK;
    };
    let alert = AlertCreation {
        severity: Severity::Error,
        identifier: identifier.to_string(),
        value: Some(number.to_string()),
        note: None,
    };
    // `trigger_alert` logs why it failed.
    match trigger_alert(config, alert).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
