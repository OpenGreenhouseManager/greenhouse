pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Serializing the config or a request body failed.
    Serialization,
    /// [`crate::DeviceBuilder::build`] without a read or a write handler.
    MissingHandler,
    /// Reading/writing the config (file or flash) failed.
    Storage,
    /// `trigger_alert` was called before the device was activated.
    ScriptingApiNotConfigured,
    /// The scripting API answered with a non-2xx status.
    HttpStatus(u16),
    /// The alert request failed before a status was received.
    #[cfg(not(target_os = "none"))]
    Request(reqwest::Error),
    /// The config does not fit into a storage slot.
    #[cfg(target_os = "none")]
    ConfigTooLarge,
    /// No `nvs` partition, or it is too small for the config store.
    #[cfg(target_os = "none")]
    MissingConfigPartition,
    /// `trigger_alert` was called before [`crate::Platform::start`].
    #[cfg(target_os = "none")]
    NotInitialized,
    /// The scripting API URL is not `http(s)://host[:port][/path]`.
    #[cfg(target_os = "none")]
    InvalidUrl,
    /// Resolving the host failed or returned no address.
    #[cfg(target_os = "none")]
    Dns,
    /// TCP connect/read/write failed.
    #[cfg(target_os = "none")]
    Io,
    /// TLS handshake or record error; holds the MbedTLS error code (0 = IO error).
    #[cfg(target_os = "none")]
    Tls(i32),
    /// The server response could not be parsed.
    #[cfg(target_os = "none")]
    InvalidResponse,
    /// The operation did not finish in time.
    #[cfg(target_os = "none")]
    Timeout,
    /// Wi-Fi setup failed.
    #[cfg(target_os = "none")]
    Wifi,
    /// Board bring-up (TRNG, TLS context, spawning a task) failed.
    #[cfg(target_os = "none")]
    Platform,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for Error {}

#[cfg(target_os = "none")]
impl From<mbedtls_rs::SessionError> for Error {
    fn from(e: mbedtls_rs::SessionError) -> Self {
        match e {
            mbedtls_rs::SessionError::MbedTls(e) => Error::Tls(e.code()),
            mbedtls_rs::SessionError::Io(_) => Error::Io,
        }
    }
}
