//! ESP32-C3 version of `input_output_int_saver`.
//!
//! Build and flash (Wi-Fi credentials are baked in at compile time):
//!
//! ```fish
//! WIFI_SSID=my-net WIFI_PASSWORD=secret cargo esp32-run --example esp32_input_output_int_saver
//! ```
//!
//! The device logic is shared with the host version (`common/int_saver.rs`).
//! Only `main` differs: board bring-up (`esp_hal::init`, heap,
//! `esp_rtos::start`) and `Platform::start` (Wi-Fi + DHCP, TLS, config flash)
//! instead of `Platform::new(config_path)`. The write handler additionally
//! sends an out-of-range alert like `input_alert_trigger` does.
//!
//! On a host this example builds to a stub, so `cargo clippy --all-targets`
//! works for the whole workspace.

#![cfg_attr(target_os = "none", no_std, no_main)]

#[cfg(target_os = "none")]
#[path = "common/int_saver.rs"]
mod int_saver;

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

    use embassy_executor::Spawner;
    use esp_backtrace as _;
    use esp_hal::clock::CpuClock;
    use esp_hal::timer::timg::TimerGroup;
    use greenhouse_core::data_storage_service_dto::alert_dto::alert::Severity;
    use greenhouse_core::smart_device_dto::{Type, config::TypeOption};
    use greenhouse_device::device_service::{AlertCreation, trigger_alert};
    use greenhouse_device::{
        Arc, Config, DeviceBuilder, Platform, PlatformPeripherals, StatusCode, WifiCredentials,
    };

    use crate::int_saver::{self, IntSaverConfig};

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

        DeviceBuilder::new(platform, int_saver::default_config())
            .read(TypeOption::Number, int_saver::read_handler)
            .write(TypeOption::Number, write_handler)
            .build()
            .unwrap()
            .run()
            .await
    }

    async fn write_handler(data: Type, config: Arc<Config<IntSaverConfig>>) -> StatusCode {
        let Some(number) = int_saver::save(data) else {
            return StatusCode::BAD_REQUEST;
        };
        if int_saver::in_range(number, &config.additional_config) {
            return StatusCode::OK;
        }
        let alert = AlertCreation {
            severity: Severity::Warning,
            identifier: OUT_OF_RANGE_ALERT_IDENTIFIER.to_string(),
            value: Some(number.to_string()),
            note: None,
        };
        // `trigger_alert` logs why it failed; the write is answered 500 either way.
        let _ = trigger_alert(config, alert).await;
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
