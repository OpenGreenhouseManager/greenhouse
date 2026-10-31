use std::collections::HashMap;

use greenhouse_macro::IntoJsonResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, IntoJsonResponse)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct GetTimeseriesDto {
    pub timeseries: Vec<TimeseriesDto>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct TimeseriesDto {
    pub timestamp: u64,
    pub value: Type,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub enum Type {
    Number(f64),
    Boolean(bool),
    Object(HashMap<String, Type>),
    Measurement(Measurement),
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "device_service/")
)]
pub struct Measurement {
    pub value: f64,
    pub unit: String,
}

impl From<Vec<TimeseriesDto>> for GetTimeseriesDto {
    fn from(timeseries: Vec<TimeseriesDto>) -> Self {
        Self { timeseries }
    }
}
