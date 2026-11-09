//! A device: its handlers, the current config and where the config is stored.

use alloc::boxed::Box;
use core::future::Future;

use greenhouse_core::smart_device_dto::Type;
use greenhouse_core::smart_device_dto::config::{ConfigRequestDto, TypeOption};
use greenhouse_core::smart_device_dto::status::DeviceStatusResponseDto;
use serde::{Serialize, de::DeserializeOwned};

use crate::config::{Config, Mode, StoredConfig, parse_stored_config};
use crate::error::{Error, Result};
use crate::platform::{
    self, Arc, AsyncMutex, BoxFuture, ConfigCell, ConfigStore, DynFn1, DynFn2, MaybeSend,
    MaybeSync, Platform, Shared, StatusCode,
};

pub(crate) type ReadHandler<T> = Box<DynFn1<Arc<Config<T>>, Type>>;
pub(crate) type WriteHandler<T> = Box<DynFn2<Type, Arc<Config<T>>, StatusCode>>;
pub(crate) type StatusHandler<T> = Box<DynFn1<Arc<Config<T>>, DeviceStatusResponseDto>>;
pub(crate) type ConfigInterceptor<T> = Box<DynFn2<ConfigRequestDto<T>, Arc<Config<T>>, Config<T>>>;

pub(crate) struct DeviceState<T: Clone + Default> {
    pub read_handler: Option<ReadHandler<T>>,
    pub write_handler: Option<WriteHandler<T>>,
    pub status_handler: StatusHandler<T>,
    pub config_interceptor: ConfigInterceptor<T>,
    pub config: ConfigCell<Arc<Config<T>>>,
    /// Held across every config read-modify-write (`/config`, `/activate`), so
    /// one update cannot overwrite another with a stale base config.
    pub storage: AsyncMutex<ConfigStore>,
    pub mode: Mode,
}

impl<T: Clone + Default> DeviceState<T> {
    pub fn config(&self) -> Arc<Config<T>> {
        self.config.get()
    }
}

/// A device, ready to [`run`](Self::run).
pub struct DeviceBuilder<T: Clone + Default + 'static> {
    pub(crate) state: Shared<DeviceState<T>>,
    /// What the server needs besides the state (the network stack on the
    /// ESP32; nothing on the host, which binds its own socket).
    #[cfg_attr(not(target_os = "none"), allow(dead_code))]
    pub(crate) server: platform::Server,
}

/// Read access to a device's current config from outside its handlers (e.g. a
/// background task that sends alerts). Cheap to clone.
pub struct ConfigHandle<T: Clone + Default + 'static>(Shared<DeviceState<T>>);

impl<T: Clone + Default + 'static> Clone for ConfigHandle<T> {
    fn clone(&self) -> Self {
        Self(platform::clone_shared(&self.0))
    }
}

impl<T: Clone + Default + 'static> ConfigHandle<T> {
    /// The config as it is right now (it changes on `/config` and `/activate`).
    pub fn get(&self) -> Arc<Config<T>> {
        self.0.config()
    }
}

fn boxed_read<T, RH, RF>(handler: RH) -> ReadHandler<T>
where
    T: Clone + Default + 'static,
    RH: Fn(Arc<Config<T>>) -> RF + MaybeSend + MaybeSync + 'static,
    RF: Future<Output = Type> + MaybeSend + 'static,
{
    Box::new(move |cfg: Arc<Config<T>>| -> BoxFuture<Type> { Box::pin(handler(cfg)) })
}

fn boxed_write<T, WH, WF>(handler: WH) -> WriteHandler<T>
where
    T: Clone + Default + 'static,
    WH: Fn(Type, Arc<Config<T>>) -> WF + MaybeSend + MaybeSync + 'static,
    WF: Future<Output = StatusCode> + MaybeSend + 'static,
{
    Box::new(
        move |data: Type, cfg: Arc<Config<T>>| -> BoxFuture<StatusCode> {
            Box::pin(handler(data, cfg))
        },
    )
}

fn boxed_status<T, SH, SF>(handler: SH) -> StatusHandler<T>
where
    T: Clone + Default + 'static,
    SH: Fn(Arc<Config<T>>) -> SF + MaybeSend + MaybeSync + 'static,
    SF: Future<Output = DeviceStatusResponseDto> + MaybeSend + 'static,
{
    Box::new(
        move |cfg: Arc<Config<T>>| -> BoxFuture<DeviceStatusResponseDto> { Box::pin(handler(cfg)) },
    )
}

fn boxed_interceptor<T, CIH, CIF>(handler: CIH) -> ConfigInterceptor<T>
where
    T: Clone + Default + 'static,
    CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + MaybeSend + MaybeSync + 'static,
    CIF: Future<Output = Config<T>> + MaybeSend + 'static,
{
    Box::new(
        move |req: ConfigRequestDto<T>, cfg: Arc<Config<T>>| -> BoxFuture<Config<T>> {
            Box::pin(handler(req, cfg))
        },
    )
}

impl<T> DeviceBuilder<T>
where
    T: Clone + Default + Serialize + DeserializeOwned + MaybeSend + MaybeSync + 'static,
{
    /// A device that can be read (`/read`) and written (`/write`).
    ///
    /// `default_config` is used (and stored) until the device has a config of
    /// its own; see [`Platform`] for where that is kept.
    #[allow(clippy::too_many_arguments)]
    pub fn new_hybrid_device<RH, RF, WH, WF, SH, SF, CIH, CIF>(
        platform: Platform,
        default_config: Config<T>,
        read_handler: RH,
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        input_type: TypeOption,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + MaybeSend + MaybeSync + 'static,
        RF: Future<Output = Type> + MaybeSend + 'static,
        WH: Fn(Type, Arc<Config<T>>) -> WF + MaybeSend + MaybeSync + 'static,
        WF: Future<Output = StatusCode> + MaybeSend + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + MaybeSend + MaybeSync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + MaybeSend + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + MaybeSend + MaybeSync + 'static,
        CIF: Future<Output = Config<T>> + MaybeSend + 'static,
    {
        Self::new(
            platform,
            default_config,
            Some(boxed_read(read_handler)),
            Some(boxed_write(write_handler)),
            boxed_status(status_handler),
            boxed_interceptor(config_interceptor_handler),
            Mode::InputOutput(input_type, output_type),
        )
    }

    /// A device that can only be read (`/read`), e.g. a sensor.
    pub fn new_output_device<RH, RF, SH, SF, CIH, CIF>(
        platform: Platform,
        default_config: Config<T>,
        read_handler: RH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + MaybeSend + MaybeSync + 'static,
        RF: Future<Output = Type> + MaybeSend + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + MaybeSend + MaybeSync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + MaybeSend + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + MaybeSend + MaybeSync + 'static,
        CIF: Future<Output = Config<T>> + MaybeSend + 'static,
    {
        Self::new(
            platform,
            default_config,
            Some(boxed_read(read_handler)),
            None,
            boxed_status(status_handler),
            boxed_interceptor(config_interceptor_handler),
            Mode::Output(output_type),
        )
    }

    /// A device that can only be written (`/write`), e.g. an actuator.
    pub fn new_input_device<WH, WF, SH, SF, CIH, CIF>(
        platform: Platform,
        default_config: Config<T>,
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        input_type: TypeOption,
    ) -> Result<Self>
    where
        WH: Fn(Type, Arc<Config<T>>) -> WF + MaybeSend + MaybeSync + 'static,
        WF: Future<Output = StatusCode> + MaybeSend + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + MaybeSend + MaybeSync + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + MaybeSend + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + MaybeSend + MaybeSync + 'static,
        CIF: Future<Output = Config<T>> + MaybeSend + 'static,
    {
        Self::new(
            platform,
            default_config,
            None,
            Some(boxed_write(write_handler)),
            boxed_status(status_handler),
            boxed_interceptor(config_interceptor_handler),
            Mode::Input(input_type),
        )
    }

    fn new(
        platform: Platform,
        default_config: Config<T>,
        read_handler: Option<ReadHandler<T>>,
        write_handler: Option<WriteHandler<T>>,
        status_handler: StatusHandler<T>,
        config_interceptor: ConfigInterceptor<T>,
        mode: Mode,
    ) -> Result<Self> {
        let (mut storage, server) = platform.into_parts();
        let config = load_or_init_config(&mut storage, default_config)?;
        Ok(Self {
            state: platform::share(DeviceState {
                read_handler,
                write_handler,
                status_handler,
                config_interceptor,
                config: ConfigCell::new(Arc::new(config)),
                storage: AsyncMutex::new(storage),
                mode,
            }),
            server,
        })
    }

    /// See [`ConfigHandle`].
    pub fn config_handle(&self) -> ConfigHandle<T> {
        ConfigHandle(platform::clone_shared(&self.state))
    }

    /// Serves the device API on the configured port. Never returns; await it at
    /// the end of `main` (or spawn it). On the host it panics if the port
    /// cannot be bound.
    pub async fn run(self) -> ! {
        platform::serve(self).await
    }
}

/// Uses the stored config, or stores the default one.
///
/// If the stored config no longer matches `Config<T>` (the device's
/// `additional_config` changed shape), the default is used but `port`,
/// `datasource_id` and `scripting_api` are kept, so the device stays activated.
fn load_or_init_config<T>(storage: &mut ConfigStore, default_config: Config<T>) -> Result<Config<T>>
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    let config = match storage.load()? {
        Some(bytes) => match parse_stored_config(&bytes, default_config.clone()) {
            StoredConfig::Valid(config) => return Ok(config),
            StoredConfig::Migrated(config) => {
                log::warn!(
                    "stored additional_config does not match this device; \
                     using the default, keeping port/datasource_id/scripting_api"
                );
                config
            }
            StoredConfig::Invalid => {
                log::warn!("stored config is not valid JSON; using the default");
                default_config
            }
        },
        None => {
            log::info!("no stored config; using the default");
            default_config
        }
    };
    let bytes = serde_json::to_vec(&config).map_err(|_| Error::IllFormattedConfig)?;
    storage.store(&bytes)?;
    Ok(config)
}
