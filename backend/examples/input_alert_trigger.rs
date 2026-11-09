use greenhouse_core::{
    data_storage_service_dto::alert_dto::alert::Severity,
    smart_device_dto::{
        Type,
        config::{ConfigRequestDto, TypeOption},
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
};
use greenhouse_device::{
    Arc, Config, DeviceBuilder, Platform, StatusCode,
    device_service::{AlertCreation, trigger_alert},
};
use serde::{Deserialize, Serialize};

const LOW_ALERT_IDENTIFIER: &str = "low_alert";
const HIGH_ALERT_IDENTIFIER: &str = "high_alert";

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[tokio::main]
async fn main() {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/input_alert_trigger/config.json".to_string());
    println!("using config file: {config_path}");

    let default_config = Config {
        port: 6002,
        datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
        additional_config: ExampleDeviceConfig { min: 0, max: 10 },
        scripting_api: None,
    };

    let device = DeviceBuilder::new_input_device(
        Platform::new(config_path),
        default_config,
        write_handler,
        status_handler,
        config_interceptor_handler,
        TypeOption::Number,
    )
    .unwrap();

    device.run().await
}

async fn write_handler(data: Type, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    let number = match data {
        Type::Number(number) => number,
        _ => return StatusCode::BAD_REQUEST,
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
    match trigger_alert(config, alert).await {
        Ok(()) => StatusCode::OK,
        Err(e) => {
            println!("failed to send {identifier}: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn status_handler(config: Arc<Config<ExampleDeviceConfig>>) -> DeviceStatusResponseDto {
    DeviceStatusResponseDto {
        status: DeviceStatusDto::Online,
        datasource_id: config.datasource_id.clone(),
    }
}

async fn config_interceptor_handler(
    config: ConfigRequestDto<ExampleDeviceConfig>,
    old_config: Arc<Config<ExampleDeviceConfig>>,
) -> Config<ExampleDeviceConfig> {
    Config {
        port: old_config.port,
        datasource_id: old_config.datasource_id.clone(),
        additional_config: ExampleDeviceConfig {
            min: config.additional_config.min,
            max: config.additional_config.max,
        },
        scripting_api: old_config.scripting_api.clone(),
    }
}
