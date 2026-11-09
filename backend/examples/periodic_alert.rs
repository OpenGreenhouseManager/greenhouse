use std::{collections::BTreeMap, sync::LazyLock};

use greenhouse_core::{
    data_storage_service_dto::alert_dto::alert::Severity,
    smart_device_dto::{
        Type,
        config::{ConfigRequestDto, TypeOption},
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
};
use greenhouse_device::{
    Arc, Config, ConfigHandle, DeviceBuilder, Error, Platform,
    device_service::{AlertCreation, trigger_alert},
};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

static ALERTS_MUTEX: LazyLock<RwLock<[AlertCounter; 5]>> = LazyLock::new(|| {
    RwLock::new([
        AlertCounter {
            identifier: "periodic_alert_1",
            count: 0,
        },
        AlertCounter {
            identifier: "periodic_alert_2",
            count: 0,
        },
        AlertCounter {
            identifier: "periodic_alert_3",
            count: 0,
        },
        AlertCounter {
            identifier: "periodic_alert_4",
            count: 0,
        },
        AlertCounter {
            identifier: "periodic_alert_5",
            count: 0,
        },
    ])
});

struct AlertCounter {
    identifier: &'static str,
    count: u64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub interval: u64,
    pub random_jitter: u64,
}

#[tokio::main]
async fn main() {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/periodic_alert/config.json".to_string());
    println!("using config file: {config_path}");

    let default_config = Config {
        port: 6003,
        datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
        additional_config: ExampleDeviceConfig {
            interval: 10,
            random_jitter: 5,
        },
        scripting_api: None,
    };

    let device = DeviceBuilder::new_output_device(
        Platform::new(config_path),
        default_config,
        read_handler,
        status_handler,
        config_interceptor_handler,
        TypeOption::Object,
    )
    .unwrap();

    tokio::spawn(start_periodic_alerts(device.config_handle()));

    device.run().await
}

async fn read_handler(_: Arc<Config<ExampleDeviceConfig>>) -> Type {
    Type::Object(BTreeMap::from_iter(ALERTS_MUTEX.read().await.iter().map(
        |alert| {
            (
                alert.identifier.to_string(),
                Type::Number(alert.count as f64),
            )
        },
    )))
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
            interval: config.additional_config.interval,
            random_jitter: config.additional_config.random_jitter,
        },
        scripting_api: old_config.scripting_api.clone(),
    }
}

/// Sends a random alert every `interval` (+ up to `random_jitter`) seconds,
/// always with the device's current config (it changes on `/config` and
/// `/activate`).
async fn start_periodic_alerts(config: ConfigHandle<ExampleDeviceConfig>) {
    loop {
        let ExampleDeviceConfig {
            interval,
            random_jitter,
        } = config.get().additional_config;
        let range = ALERTS_MUTEX.read().await.len();
        let random_index = rand::rng().random_range(0..range);
        let random_severity = rand::rng().random_range(0..4);
        let wait_time = interval + rand::rng().random_range(0..random_jitter.max(1));
        tokio::time::sleep(std::time::Duration::from_secs(wait_time)).await;
        let res = trigger_alert(
            config.get(),
            AlertCreation {
                identifier: ALERTS_MUTEX.read().await[random_index]
                    .identifier
                    .to_string(),
                severity: match random_severity {
                    0 => Severity::Info,
                    1 => Severity::Warning,
                    2 => Severity::Error,
                    3 => Severity::Fatal,
                    _ => panic!("Invalid severity"),
                },
                value: None,
                note: None,
            },
        )
        .await;
        match res {
            Ok(_) => {
                println!("Alert triggered successfully after {wait_time} seconds");
                ALERTS_MUTEX.write().await[random_index].count += 1;
            }
            Err(Error::ScriptingApiNotConfigured) => {
                println!("Scripting API not configured yet (device not activated)");
            }
            Err(e) => println!("Error triggering alert: {e}"),
        }
    }
}
