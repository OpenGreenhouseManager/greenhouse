//! Device configuration: the same types (and JSON) on every platform.

mod common;

pub use common::AlertCreation;
pub use common::{Config, Mode, ScriptingApi};
pub(crate) use common::{StoredConfig, parse_stored_config};
