//! Board bring-up shared by every device: Wi-Fi + DHCP, TRNG, TLS and the
//! config store.

use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::{ADC1, FLASH, RNG, WIFI};
use esp_hal::rng::{Trng, TrngSource};
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, ControllerConfig, Interface, WifiController,
};
use esp_storage::FlashStorage;
use mbedtls_rs::Tls;
use static_cell::StaticCell;

use crate::device_service::alert_task;
use crate::error::{Error, Result};
use crate::storage::ConfigStore;

/// Sockets: HTTP server workers + DNS + one alert connection, with headroom.
const SOCKETS: usize = 7;

/// Peripherals the platform takes ownership of.
pub struct PlatformPeripherals {
    pub wifi: WIFI<'static>,
    pub rng: RNG<'static>,
    pub adc1: ADC1<'static>,
    pub flash: FLASH<'static>,
}

pub struct WifiCredentials {
    pub ssid: &'static str,
    /// Empty for an open network; otherwise WPA2-Personal.
    pub password: &'static str,
}

/// A running platform: connected network stack plus config storage.
/// Hand it to a [`crate::DeviceBuilder`] constructor.
pub struct Platform {
    pub stack: Stack<'static>,
    pub(crate) storage: ConfigStore,
}

impl Platform {
    /// Connects to Wi-Fi, waits for a DHCP lease and starts the background
    /// tasks (Wi-Fi reconnect, network stack, alert sender).
    ///
    /// Call once, after `esp_hal::init`, the heap allocator and `esp_rtos::start`.
    pub async fn start(
        spawner: Spawner,
        peripherals: PlatformPeripherals,
        credentials: WifiCredentials,
    ) -> Result<Self> {
        static TRNG_SOURCE: StaticCell<TrngSource<'static>> = StaticCell::new();
        static TRNG: StaticCell<Trng> = StaticCell::new();
        static RESOURCES: StaticCell<StackResources<SOCKETS>> = StaticCell::new();

        // Entropy from the SAR ADC; makes `Trng` cryptographically secure even
        // before the radio is up. Kept alive for the rest of the program.
        TRNG_SOURCE.init(TrngSource::new(peripherals.rng, peripherals.adc1));
        let trng = TRNG.init(Trng::try_new().map_err(|_| Error::NotInitialized)?);
        let seed = u64::from(trng.random()) << 32 | u64::from(trng.random());
        let tls = Tls::new(trng).map_err(|_| Error::NotInitialized)?;

        let storage = ConfigStore::new(FlashStorage::new(peripherals.flash))?;

        let authentication = if credentials.password.is_empty() {
            AuthenticationMethodConfig::Open
        } else {
            AuthenticationMethodConfig::Wpa2Personal(
                credentials.password.try_into().map_err(|_| Error::Wifi)?,
            )
        };
        let station = WifiConfig::Station(
            StationConfig::default()
                .with_ssid(credentials.ssid.try_into().map_err(|_| Error::Wifi)?)
                .with_authentication(authentication),
        );
        let controller = WifiController::new(
            peripherals.wifi,
            ControllerConfig::default().with_initial_config(station),
        )
        .map_err(|e| {
            log::error!("Wi-Fi init failed: {e:?}");
            Error::Wifi
        })?;

        let (stack, runner) = embassy_net::new(
            Interface::station(),
            embassy_net::Config::dhcpv4(Default::default()),
            RESOURCES.init(StackResources::new()),
            seed,
        );

        spawner.spawn(wifi_task(controller).map_err(|_| Error::NotInitialized)?);
        spawner.spawn(net_task(runner).map_err(|_| Error::NotInitialized)?);
        spawner.spawn(alert_task(stack, tls).map_err(|_| Error::NotInitialized)?);

        stack.wait_config_up().await;
        if let Some(config) = stack.config_v4() {
            log::info!("Got IP: {}", config.address);
        }

        Ok(Self { stack, storage })
    }
}

#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>) {
    loop {
        match controller.connect_async().await {
            Ok(_) => {
                log::info!("Wi-Fi connected");
                let reason = controller.wait_for_disconnect_async().await;
                log::warn!("Wi-Fi disconnected: {reason:?}");
            }
            Err(e) => log::warn!("Wi-Fi connect failed: {e:?}"),
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}
