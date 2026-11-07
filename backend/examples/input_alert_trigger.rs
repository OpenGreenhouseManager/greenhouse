use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use greenhouse_core::{
    alert_debounce::{DebounceConfig, Signal, Threshold, ThresholdDebouncer},
    data_storage_service_dto::alert_dto::alert::{AlertKind, Severity},
    smart_device_dto::{
        Type,
        config::{ConfigRequestDto, TypeOption},
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    },
    smart_device_interface::{
        config::{Config, read_config_file_with_path, update_config_file_with_path},
        device_builder::DeviceBuilder,
        device_service::{AlertCreation, stop_alert, trigger_alert},
        hybrid_device::init_hybrid_router,
    },
};
use serde_derive::{Deserialize, Serialize};

const LOW_ALERT_IDENTIFIER: &str = "low_alert";
const HIGH_ALERT_IDENTIFIER: &str = "high_alert";
/// A limit counts as cleared only once the value is this far back inside it.
const HYSTERESIS: f64 = 1.0;
/// Consecutive out-of-range (or back-in-range) writes before an alert starts (or stops).
const DEBOUNCE: DebounceConfig = DebounceConfig {
    start_after: 3,
    stop_after: 3,
};
/// Writes arrive irregularly, so an open alert is only dropped after five
/// minutes without a write.
const KEEP_ALIVE_SECS: u32 = 300;

struct AlertState {
    low: ThresholdDebouncer,
    high: ThresholdDebouncer,
}

impl AlertState {
    fn new() -> Self {
        Self {
            low: ThresholdDebouncer::new(Threshold::below(0.0, 0.0), DEBOUNCE),
            high: ThresholdDebouncer::new(Threshold::above(0.0, 0.0), DEBOUNCE),
        }
    }

    /// Updates the limits from the (possibly changed) config and feeds the value.
    fn update(&mut self, number: f64, config: &ExampleDeviceConfig) -> [(Signal, &'static str); 2] {
        let (min, max) = (config.min as f64, config.max as f64);
        self.low
            .set_threshold(Threshold::below(min, min + HYSTERESIS));
        self.high
            .set_threshold(Threshold::above(max, max - HYSTERESIS));
        [
            (self.low.update(number), LOW_ALERT_IDENTIFIER),
            (self.high.update(number), HIGH_ALERT_IDENTIFIER),
        ]
    }
}

static ALERT_STATE: Mutex<Option<AlertState>> = Mutex::new(None);

#[derive(Serialize, Deserialize, Clone, Default)]
struct ExampleDeviceConfig {
    pub min: i32,
    pub max: i32,
}

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    // Get config path from command line arguments, default to "./config/config.json"
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/input_alert_trigger/config.json".to_string());

    let config = match read_config_file_with_path(&config_path) {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config {
                port: 6002,

                datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
                additional_config: ExampleDeviceConfig { min: 0, max: 10 },
                scripting_api: None,
            };

            // Create config directory if it doesn't exist
            if let Some(parent) = std::path::Path::new(&config_path).parent()
                && !parent.exists()
            {
                std::fs::create_dir_all(parent).unwrap();
            }

            // Create empty config file if it doesn't exist
            if !std::path::Path::new(&config_path).exists() {
                std::fs::write(&config_path, "{}").unwrap();
            }

            update_config_file_with_path(&default_config, &config_path).unwrap();
            default_config
        }
    };

    let device_service = DeviceBuilder::new_input_device_with_config_path(
        write_handler,
        status_handler,
        config_interceptor_handler,
        &config_path,
        TypeOption::Number,
    )
    .unwrap();
    let router = init_hybrid_router(device_service);

    let url = format!("0.0.0.0:{}", config.port);

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    println!("using config file: {config_path}");
    axum::serve(listener, router).await.unwrap();
}

async fn write_handler(data: Type, config: Arc<Config<ExampleDeviceConfig>>) -> StatusCode {
    let number = match data {
        Type::Number(number) => number,
        _ => return StatusCode::BAD_REQUEST,
    };
    let signals = ALERT_STATE
        .lock()
        .unwrap()
        .get_or_insert_with(AlertState::new)
        .update(number, &config.additional_config);

    for (signal, identifier) in signals {
        let result = match signal {
            Signal::Idle => continue,
            // Fire on every out-of-range write: the first starts the alert, the
            // rest keep it alive. A failed fire is simply retried next write.
            Signal::Fire { .. } => {
                trigger_alert(
                    config.clone(),
                    AlertCreation {
                        severity: Severity::Error,
                        identifier: identifier.to_string(),
                        value: Some(number.to_string()),
                        note: None,
                        kind: AlertKind::Concurrent,
                        keep_alive_secs: Some(KEEP_ALIVE_SECS),
                    },
                )
                .await
            }
            Signal::Stop => {
                stop_alert(
                    config.clone(),
                    identifier.to_string(),
                    Some(format!("value back in range: {number}")),
                )
                .await
            }
        };
        if let Err(e) = result {
            println!("Error sending {identifier} alert: {e}");
        }
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
        additional_config: {
            ExampleDeviceConfig {
                min: config.additional_config.min,
                max: config.additional_config.max,
            }
        },
        scripting_api: old_config.scripting_api.clone(),
    }
}
