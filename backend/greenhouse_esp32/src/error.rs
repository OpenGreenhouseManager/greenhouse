pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The stored or received config could not be (de)serialized.
    IllFormattedConfig,
    /// The config does not fit into a storage slot.
    ConfigTooLarge,
    /// No `nvs` partition, or it is too small for the config store.
    MissingConfigPartition,
    /// Reading/writing the flash failed.
    Storage,
    /// `trigger_alert` was called before the device was activated.
    ScriptingApiNotConfigured,
    /// `trigger_alert` was called before [`crate::Platform::start`].
    NotInitialized,
    /// The scripting API URL is not `http(s)://host[:port][/path]`.
    InvalidUrl,
    /// Resolving the host failed or returned no address.
    Dns,
    /// TCP connect/read/write failed.
    Io,
    /// TLS handshake or record error; holds the MbedTLS error code (0 = IO error).
    Tls(i32),
    /// The server answered with a non-2xx status.
    HttpStatus(u16),
    /// The response could not be parsed.
    InvalidResponse,
    /// The operation did not finish in time.
    Timeout,
    /// Wi-Fi setup failed.
    Wifi,
    /// Board bring-up (TRNG, TLS context, spawning a task) failed.
    Platform,
    /// Serializing a request body failed.
    SerializationError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for Error {}

impl From<mbedtls_rs::SessionError> for Error {
    fn from(e: mbedtls_rs::SessionError) -> Self {
        match e {
            mbedtls_rs::SessionError::MbedTls(e) => Error::Tls(e.code()),
            mbedtls_rs::SessionError::Io(_) => Error::Io,
        }
    }
}
