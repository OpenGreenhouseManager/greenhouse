//! ESP32-C3 version of `backend/examples/input_output_int_saver.rs`.
//!
//! Build and flash (Wi-Fi credentials are baked in at compile time):
//!
//! ```fish
//! WIFI_SSID=my-net WIFI_PASSWORD=secret cargo esp32-run --example esp32_input_output_int_saver
//! ```
//!
//! Only `main` differs from the host version: board bring-up (`esp_hal::init`,
//! heap, `esp_rtos::start`) and `Platform::start` (Wi-Fi + DHCP, TLS, config
//! flash) instead of `Platform::new(config_path)`. The handlers are the same;
//! the write handler additionally sends an out-of-range alert like
//! `input_alert_trigger` does.
//!
//! On a host this example builds to a stub, so `cargo clippy --all-targets`
//! works for the whole workspace.

#![cfg_attr(target_os = "none", no_std, no_main)]

#[cfg(not(target_os = "none"))]
fn main() {
    eprintln!(
        "ESP32 firmware: build it with `cargo esp32-build --example esp32_input_output_int_saver`"
    );
}

#[cfg(target_os = "none")]
mod firmware {
    extern crate alloc;

    use alloc::string::ToString;
    use core::sync::atomic::{AtomicI32, Ordering};

    use embassy_executor::Spawner;
    use esp_backtrace as _;
    use esp_hal::clock::CpuClock;
    use esp_hal::timer::timg::TimerGroup;
    use greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity;
    use greenhouse_core::smart_device_dto::{
        Type,
        config::{ConfigRequestDto, TypeOption},
        status::{DeviceStatusDto, DeviceStatusResponseDto},
    };
    use greenhouse_device::device_service::{AlertCreation, trigger_alert};
    use greenhouse_device::{
        Arc, Config, DeviceBuilder, Platform, PlatformPeripherals, StatusCode, WifiCredentials,
    };
    use serde::{Deserialize, Serialize};

    esp_bootloader_esp_idf::esp_app_desc!();

    const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
        Some(ssid) => ssid,
        None => "",
    };
    const WIFI_PASSWORD: &str = match option_env!("WIFI_PASSWORD") {
        Some(password) => password,
        None => "",
    };
    const OUT_OF_RANGE_ALERT_IDENTIFIER: &str = "out_of_range";

    static SAVED_NUMBER: AtomicI32 = AtomicI32::new(20);

    #[derive(Serialize, Deserialize, Clone, Default)]
    struct ExampleDeviceConfig {
        pub min: i32,
        pub max: i32,
    }

    #[esp_hal::main]
    async fn main(spawner: Spawner) -> ! {
        esp_println::logger::init_logger_from_env();
        let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

        // Wi-Fi and a TLS session each need tens of KiB of heap.
        esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
        esp_alloc::heap_allocator!(size: 96 * 1024);

        let timg0 = TimerGroup::new(peripherals.TIMG0);
        esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

        assert!(
            !WIFI_SSID.is_empty(),
            "build with WIFI_SSID=... (and WIFI_PASSWORD=...) set"
        );
        let platform = Platform::start(
            spawner,
            PlatformPeripherals {
                wifi: peripherals.WIFI,
                rng: peripherals.RNG,
                adc1: peripherals.ADC1,
                flash: peripherals.FLASH,
            },
            WifiCredentials {
                ssid: WIFI_SSID,
                password: WIFI_PASSWORD,
            },
        )
        .await
        .unwrap();

        let default_config = Config {
            port: 6001,
            datasource_id: "7a224a14-6e07-45a3-91da-b7584a5731c1".to_string(),
            additional_config: ExampleDeviceConfig { min: 0, max: 100 },
            scripting_api: None,
        };

        let device = DeviceBuilder::new_hybrid_device(
            platform,
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
        if config.additional_config.min > number as i32
            || config.additional_config.max < number as i32
        {
            let alert = AlertCreation {
                severity: Severity::Warning,
                identifier: OUT_OF_RANGE_ALERT_IDENTIFIER.to_string(),
                value: Some(number.to_string()),
                note: None,
            };
            if let Err(e) = trigger_alert(config.clone(), alert).await {
                log::warn!("failed to send out-of-range alert: {e}");
            }
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
}
