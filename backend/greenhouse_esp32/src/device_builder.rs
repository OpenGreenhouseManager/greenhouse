//! Mirrors `greenhouse_core::smart_device_interface::device_builder`.

use alloc::boxed::Box;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;

use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use greenhouse_core::smart_device_dto::Type;
use greenhouse_core::smart_device_dto::config::{ConfigRequestDto, TypeOption};
use greenhouse_core::smart_device_dto::status::DeviceStatusResponseDto;
use serde::{Serialize, de::DeserializeOwned};

use crate::config::{Config, Mode};
use crate::error::Result;
use crate::platform::Platform;
use crate::storage::ConfigStore;
use crate::{Arc, StatusCode};

type BoxFuture<O> = Pin<Box<dyn Future<Output = O>>>;
pub(crate) type ReadHandler<T> = Box<dyn Fn(Arc<Config<T>>) -> BoxFuture<Type>>;
pub(crate) type WriteHandler<T> = Box<dyn Fn(Type, Arc<Config<T>>) -> BoxFuture<StatusCode>>;
pub(crate) type StatusHandler<T> =
    Box<dyn Fn(Arc<Config<T>>) -> BoxFuture<DeviceStatusResponseDto>>;
pub(crate) type ConfigInterceptor<T> =
    Box<dyn Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> BoxFuture<Config<T>>>;

pub(crate) struct DeviceState<T: Clone + Default> {
    pub read_handler: Option<ReadHandler<T>>,
    pub write_handler: Option<WriteHandler<T>>,
    pub status_handler: StatusHandler<T>,
    pub config_interceptor: ConfigInterceptor<T>,
    /// Never borrowed across an `.await`.
    pub config: RefCell<Arc<Config<T>>>,
    pub storage: Mutex<NoopRawMutex, ConfigStore>,
    pub mode: Mode,
}

impl<T: Clone + Default> DeviceState<T> {
    pub fn config(&self) -> Arc<Config<T>> {
        self.config.borrow().clone()
    }
}

pub struct DeviceBuilder<T: Clone + Default> {
    pub(crate) state: DeviceState<T>,
    pub(crate) stack: Stack<'static>,
}

fn boxed_read<T, RH, RF>(handler: RH) -> ReadHandler<T>
where
    T: Clone + Default + 'static,
    RH: Fn(Arc<Config<T>>) -> RF + 'static,
    RF: Future<Output = Type> + 'static,
{
    Box::new(move |cfg| Box::pin(handler(cfg)))
}

fn boxed_write<T, WH, WF>(handler: WH) -> WriteHandler<T>
where
    T: Clone + Default + 'static,
    WH: Fn(Type, Arc<Config<T>>) -> WF + 'static,
    WF: Future<Output = StatusCode> + 'static,
{
    Box::new(move |data, cfg| Box::pin(handler(data, cfg)))
}

fn boxed_status<T, SH, SF>(handler: SH) -> StatusHandler<T>
where
    T: Clone + Default + 'static,
    SH: Fn(Arc<Config<T>>) -> SF + 'static,
    SF: Future<Output = DeviceStatusResponseDto> + 'static,
{
    Box::new(move |cfg| Box::pin(handler(cfg)))
}

fn boxed_interceptor<T, CIH, CIF>(handler: CIH) -> ConfigInterceptor<T>
where
    T: Clone + Default + 'static,
    CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + 'static,
    CIF: Future<Output = Config<T>> + 'static,
{
    Box::new(move |req, cfg| Box::pin(handler(req, cfg)))
}

impl<T> DeviceBuilder<T>
where
    T: Clone + Default + Serialize + DeserializeOwned + 'static,
{
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
        RH: Fn(Arc<Config<T>>) -> RF + 'static,
        RF: Future<Output = Type> + 'static,
        WH: Fn(Type, Arc<Config<T>>) -> WF + 'static,
        WF: Future<Output = StatusCode> + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + 'static,
        CIF: Future<Output = Config<T>> + 'static,
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

    pub fn new_output_device<RH, RF, SH, SF, CIH, CIF>(
        platform: Platform,
        default_config: Config<T>,
        read_handler: RH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        output_type: TypeOption,
    ) -> Result<Self>
    where
        RH: Fn(Arc<Config<T>>) -> RF + 'static,
        RF: Future<Output = Type> + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + 'static,
        CIF: Future<Output = Config<T>> + 'static,
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

    pub fn new_input_device<WH, WF, SH, SF, CIH, CIF>(
        platform: Platform,
        default_config: Config<T>,
        write_handler: WH,
        status_handler: SH,
        config_interceptor_handler: CIH,
        input_type: TypeOption,
    ) -> Result<Self>
    where
        WH: Fn(Type, Arc<Config<T>>) -> WF + 'static,
        WF: Future<Output = StatusCode> + 'static,
        SH: Fn(Arc<Config<T>>) -> SF + 'static,
        SF: Future<Output = DeviceStatusResponseDto> + 'static,
        CIH: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> CIF + 'static,
        CIF: Future<Output = Config<T>> + 'static,
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
        let Platform { stack, mut storage } = platform;
        let config = load_or_init_config(&mut storage, default_config)?;
        Ok(Self {
            state: DeviceState {
                read_handler,
                write_handler,
                status_handler,
                config_interceptor,
                config: RefCell::new(Arc::new(config)),
                storage: Mutex::new(storage),
                mode,
            },
            stack,
        })
    }

    /// Serves the device API on the configured port. Never returns; await it
    /// at the end of `main`.
    pub async fn run(self) -> ! {
        crate::handler::serve(self).await
    }
}

/// Like the Linux version: use the stored config, or persist the default one.
fn load_or_init_config<T>(storage: &mut ConfigStore, default_config: Config<T>) -> Result<Config<T>>
where
    T: Clone + Default + Serialize + DeserializeOwned,
{
    match storage.load()? {
        Some(bytes) => match serde_json::from_slice(&bytes) {
            Ok(config) => return Ok(config),
            Err(e) => log::warn!("stored config is invalid ({e}); using the default"),
        },
        None => log::info!("no stored config; using the default"),
    }
    let bytes =
        serde_json::to_vec(&default_config).map_err(|_| crate::Error::IllFormattedConfig)?;
    storage.store(&bytes)?;
    Ok(default_config)
}
