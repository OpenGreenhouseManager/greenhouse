//! The int saver device, shared by `input_output_int_saver` (host) and
//! `esp32_input_output_int_saver` (ESP32). Must stay `no_std`-clean.
//!
//! Stores a written number and reads it back; `min..=max` from the device's
//! `additional_config` marks the accepted range.

use core::sync::atomic::{AtomicI32, Ordering};

use greenhouse_core::smart_device_dto::Type;
use greenhouse_device::{Arc, Config};
use serde::{Deserialize, Serialize};

static SAVED_NUMBER: AtomicI32 = AtomicI32::new(20);

#[derive(Serialize, Deserialize, Clone)]
pub struct IntSaverConfig {
    pub min: i32,
    pub max: i32,
}

pub fn default_config() -> Config<IntSaverConfig> {
    Config {
        port: 6001,
        datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".into(),
        additional_config: IntSaverConfig { min: 0, max: 100 },
        scripting_api: None,
    }
}

pub async fn read_handler(_: Arc<Config<IntSaverConfig>>) -> Type {
    Type::Number(SAVED_NUMBER.load(Ordering::Relaxed) as f64)
}

/// Stores `data` if it is a number (even outside the range) and returns it.
pub fn save(data: Type) -> Option<i32> {
    let Type::Number(number) = data else {
        return None;
    };
    let number = number as i32;
    SAVED_NUMBER.store(number, Ordering::Relaxed);
    Some(number)
}

pub fn in_range(number: i32, config: &IntSaverConfig) -> bool {
    (config.min..=config.max).contains(&number)
}
