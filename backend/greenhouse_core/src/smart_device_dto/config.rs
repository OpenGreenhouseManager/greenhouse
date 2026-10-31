use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct ConfigResponseDto<T> {
    pub mode: Mode,
    pub input_type: Option<TypeOption>,
    pub output_type: Option<TypeOption>,
    pub scripting_api: Option<ScriptingApi>,
    pub additional_config: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub enum Mode {
    Input,
    Output,
    InputOutput,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct ConfigRequestDto<T> {
    pub additional_config: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub struct ScriptingApi {
    pub url: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "smart_device/")
)]
pub enum TypeOption {
    Number,
    Boolean,
    Object,
    Measurement,
    Stream,
    Unknown,
}
