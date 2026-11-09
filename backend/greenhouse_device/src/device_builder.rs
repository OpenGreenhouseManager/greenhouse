//! Building a [`Device`]: its handlers, the current config and where the
//! config is stored.

use alloc::boxed::Box;
use core::future::Future;

use greenhouse_core::smart_device_dto::Type;
use greenhouse_core::smart_device_dto::config::{ConfigRequestDto, TypeOption};
use greenhouse_core::smart_device_dto::status::{DeviceStatusDto, DeviceStatusResponseDto};
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

pub(crate) struct DeviceState<T> {
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

impl<T> DeviceState<T> {
    pub fn config(&self) -> Arc<Config<T>> {
        self.config.get()
    }
}

/// Collects a device's handlers; [`build`](Self::build) turns them into a
/// [`Device`].
///
/// ```ignore
/// let device = DeviceBuilder::new(platform, default_config)
///     .read(TypeOption::Number, read_handler)   // `/read`: a sensor
///     .write(TypeOption::Number, write_handler) // `/write`: an actuator
///     .build()?;
/// device.run().await
/// ```
///
/// At least one of [`read`](Self::read) / [`write`](Self::write) is required.
/// [`status`](Self::status) defaults to "online", and
/// [`config_interceptor`](Self::config_interceptor) to taking the requested
/// `additional_config` as is.
pub struct DeviceBuilder<T> {
    platform: Platform,
    default_config: Config<T>,
    read: Option<(TypeOption, ReadHandler<T>)>,
    write: Option<(TypeOption, WriteHandler<T>)>,
    status: Option<StatusHandler<T>>,
    config_interceptor: Option<ConfigInterceptor<T>>,
}

impl<T> DeviceBuilder<T>
where
    T: Clone + Serialize + DeserializeOwned + MaybeSend + MaybeSync + 'static,
{
    /// `default_config` is used (and stored) until the device has a config of
    /// its own; see [`Platform`] for where that is kept.
    pub fn new(platform: Platform, default_config: Config<T>) -> Self {
        Self {
            platform,
            default_config,
            read: None,
            write: None,
            status: None,
            config_interceptor: None,
        }
    }

    /// Serves `GET /read`: the device outputs data of `output_type` (a sensor).
    pub fn read<H, F>(mut self, output_type: TypeOption, handler: H) -> Self
    where
        H: Fn(Arc<Config<T>>) -> F + MaybeSend + MaybeSync + 'static,
        F: Future<Output = Type> + MaybeSend + 'static,
    {
        let handler = move |config| -> BoxFuture<Type> { Box::pin(handler(config)) };
        self.read = Some((output_type, Box::new(handler)));
        self
    }

    /// Serves `POST /write`: the device takes input of `input_type` (an actuator).
    pub fn write<H, F>(mut self, input_type: TypeOption, handler: H) -> Self
    where
        H: Fn(Type, Arc<Config<T>>) -> F + MaybeSend + MaybeSync + 'static,
        F: Future<Output = StatusCode> + MaybeSend + 'static,
    {
        let handler =
            move |data, config| -> BoxFuture<StatusCode> { Box::pin(handler(data, config)) };
        self.write = Some((input_type, Box::new(handler)));
        self
    }

    /// Answers `GET /status`. Default: online, with the configured `datasource_id`.
    pub fn status<H, F>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<Config<T>>) -> F + MaybeSend + MaybeSync + 'static,
        F: Future<Output = DeviceStatusResponseDto> + MaybeSend + 'static,
    {
        let handler =
            move |config| -> BoxFuture<DeviceStatusResponseDto> { Box::pin(handler(config)) };
        self.status = Some(Box::new(handler));
        self
    }

    /// Turns a `POST /config` request and the current config into the new
    /// config. Default: [`Config::with_additional_config`] with the request's
    /// `additional_config`.
    pub fn config_interceptor<H, F>(mut self, handler: H) -> Self
    where
        H: Fn(ConfigRequestDto<T>, Arc<Config<T>>) -> F + MaybeSend + MaybeSync + 'static,
        F: Future<Output = Config<T>> + MaybeSend + 'static,
    {
        let handler =
            move |request, config| -> BoxFuture<Config<T>> { Box::pin(handler(request, config)) };
        self.config_interceptor = Some(Box::new(handler));
        self
    }

    /// Loads the stored config (or stores the default one).
    ///
    /// Fails with [`Error::MissingHandler`] if neither [`read`](Self::read)
    /// nor [`write`](Self::write) was set.
    pub fn build(self) -> Result<Device<T>> {
        let (read_handler, write_handler, mode) = match (self.read, self.write) {
            (Some((output, read)), Some((input, write))) => {
                (Some(read), Some(write), Mode::InputOutput(input, output))
            }
            (Some((output, read)), None) => (Some(read), None, Mode::Output(output)),
            (None, Some((input, write))) => (None, Some(write), Mode::Input(input)),
            (None, None) => return Err(Error::MissingHandler),
        };
        let status_handler = self.status.unwrap_or_else(|| {
            Box::new(
                |config: Arc<Config<T>>| -> BoxFuture<DeviceStatusResponseDto> {
                    Box::pin(async move {
                        DeviceStatusResponseDto {
                            status: DeviceStatusDto::Online,
                            datasource_id: config.datasource_id.clone(),
                        }
                    })
                },
            )
        });
        let config_interceptor = self.config_interceptor.unwrap_or_else(|| {
            Box::new(
                |request: ConfigRequestDto<T>, config: Arc<Config<T>>| -> BoxFuture<Config<T>> {
                    Box::pin(
                        async move { config.with_additional_config(request.additional_config) },
                    )
                },
            )
        });

        let (mut storage, server) = self.platform.into_parts();
        let config = load_or_init_config(&mut storage, self.default_config)?;
        Ok(Device {
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
}

/// A device, ready to [`run`](Self::run). Made by [`DeviceBuilder`].
pub struct Device<T: 'static> {
    pub(crate) state: Shared<DeviceState<T>>,
    /// What the server needs besides the state (the network stack on the
    /// ESP32; nothing on the host, which binds its own socket).
    pub(crate) server: platform::Server,
}

impl<T> Device<T>
where
    T: Clone + Serialize + DeserializeOwned + MaybeSend + MaybeSync + 'static,
{
    /// See [`ConfigHandle`].
    pub fn config_handle(&self) -> ConfigHandle<T> {
        ConfigHandle(platform::clone_shared(&self.state))
    }

    /// Serves the device API on the configured port. Never returns; await it at
    /// the end of `main` (or spawn it). On the host it panics if the port
    /// cannot be bound.
    pub async fn run(self) -> ! {
        platform::serve(self.state, self.server).await
    }
}

/// Read access to a device's current config from outside its handlers (e.g. a
/// background task that sends alerts). Cheap to clone.
pub struct ConfigHandle<T: 'static>(Shared<DeviceState<T>>);

impl<T> Clone for ConfigHandle<T> {
    fn clone(&self) -> Self {
        Self(platform::clone_shared(&self.0))
    }
}

impl<T> ConfigHandle<T> {
    /// The config as it is right now (it changes on `/config` and `/activate`).
    pub fn get(&self) -> Arc<Config<T>> {
        self.0.config()
    }
}

/// Uses the stored config, or stores the default one.
///
/// If the stored config no longer matches `Config<T>` (the device's
/// `additional_config` changed shape), the default is used but `port`,
/// `datasource_id` and `scripting_api` are kept, so the device stays activated.
fn load_or_init_config<T>(storage: &mut ConfigStore, default_config: Config<T>) -> Result<Config<T>>
where
    T: Clone + Serialize + DeserializeOwned,
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
    let bytes = serde_json::to_vec(&config).map_err(|_| Error::Serialization)?;
    storage.store(&bytes)?;
    Ok(config)
}
