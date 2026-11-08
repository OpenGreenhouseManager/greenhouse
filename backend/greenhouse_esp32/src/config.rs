use alloc::string::String;

use greenhouse_core::smart_device_dto::config::TypeOption;
use serde::{Deserialize, Serialize};

/// Device configuration; same shape and JSON as
/// `greenhouse_core::smart_device_interface::config::Config`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config<T>
where
    T: Clone + Default,
{
    pub port: u16,
    pub datasource_id: String,
    pub additional_config: T,
    pub scripting_api: Option<ScriptingApi>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScriptingApi {
    pub url: String,
    pub token: String,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Input(TypeOption),
    Output(TypeOption),
    InputOutput(TypeOption, TypeOption),
}
