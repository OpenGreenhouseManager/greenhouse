use derive_more::From;

use crate::database;

pub(crate) type Result<T> = core::result::Result<T, Error>;

#[allow(dead_code)]
#[derive(Debug, From)]
pub(crate) enum Error {
    /// No response from the device (connection failed, timeout, ...).
    Unreachable(String),
    /// The device answered with an error status.
    Request,
    Json,
    ServiceIsTooSlow,
    #[from]
    Database(database::Error),
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
