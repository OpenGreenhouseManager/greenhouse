use std::sync::atomic::{AtomicI32, Ordering};

use greenhouse_core::smart_device_dto::{
    Type,
    config::{ConfigRequestDto, TypeOption},
    status::{DeviceStatusDto, DeviceStatusResponseDto},
};
use greenhouse_device::{Arc, Config, DeviceBuilder, Platform, StatusCode};
use serde::{Deserialize, Serialize};

static SAVED_NUMBER: AtomicI32 = AtomicI32::new(20);

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[tokio::main]
async fn main() {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/input_output_int_saver/config.json".to_string());
    println!("using config file: {config_path}");

    let default_config = Config {
        port: 6001,
        datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
        additional_config: ExampleDeviceConfig { min: 0, max: 100 },
        scripting_api: None,
    };

    let device = DeviceBuilder::new_hybrid_device(
        Platform::new(config_path),
        default_config,
        read_handler,
        write_handler,
        status_handler,
        config_interceptor_handler,
        TypeOption::Number,
        TypeOption::Number,
    )
    .unwrap();

    device.run().await
}

async fn read_handler(_: Arc<Config<ExampleDeviceConfig>>) -> Type {
    Type::Number(SAVED_NUMBER.load(Ordering::Relaxed) as f64)
}

async fn write_handler(data: Type, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    let number = match data {
        Type::Number(number) => number,
        _ => return StatusCode::BAD_REQUEST,
    };
    SAVED_NUMBER.store(number as i32, Ordering::Relaxed);
    if config.additional_config.min > number as i32 || config.additional_config.max < number as i32
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
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
