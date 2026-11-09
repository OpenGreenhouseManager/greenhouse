//! Stores a number (`/write`) and reads it back (`/read`); `/write` answers 500
//! outside `min..=max`. The device logic is in `common/int_saver.rs`, shared
//! with `esp32_input_output_int_saver`.

#[path = "common/int_saver.rs"]
mod int_saver;

use greenhouse_core::smart_device_dto::{Type, config::TypeOption};
use greenhouse_device::{Arc, Config, DeviceBuilder, Platform, StatusCode};

use int_saver::IntSaverConfig;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./config/input_output_int_saver/config.json".to_string());
    log::info!("using config file: {config_path}");

    DeviceBuilder::new(Platform::new(config_path), int_saver::default_config())
        .read(TypeOption::Number, int_saver::read_handler)
        .write(TypeOption::Number, write_handler)
        .build()
        .unwrap()
        .run()
        .await
}

async fn write_handler(data: Type, config: Arc<Config<IntSaverConfig>>) -> StatusCode {
    match int_saver::save(data) {
        None => StatusCode::BAD_REQUEST,
        Some(number) if int_saver::in_range(number, &config.additional_config) => StatusCode::OK,
        Some(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
