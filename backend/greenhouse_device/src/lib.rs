//! Building blocks for OpenGreenhouseManager smart devices.
//!
//! One API on every platform: async handlers taking `Arc<Config<T>>`, a
//! [`DeviceBuilder`] (`read` / `write` handlers, optional `status` /
//! `config_interceptor`) that builds a [`Device`], [`Device::run`], and
//! [`device_service::trigger_alert`]. Every device serves the same HTTP contract
//! (`/read`, `/write`, `/status`, `/config`, `/activate`).
//!
//! The runtime is chosen by the build target:
//! - **Host** (Linux, …): axum + tokio, the config lives in a JSON file
//!   ([`Platform::new`] takes its path), alerts go out via reqwest.
//! - **ESP32-C3** (`target_os = "none"`): esp-hal + embassy, `no_std`.
//!   [`Platform::start`] brings up Wi-Fi, TLS and the config store in the `nvs`
//!   flash partition; alerts go out via mbedtls.
//!
//! Only [`Platform`]'s constructor differs between the two. [`Arc`] and
//! [`StatusCode`] are re-exported so handlers compile unchanged on both.
#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

pub mod config;
pub mod device_builder;
pub mod device_service;
mod error;
mod handler;
mod platform;

pub use config::{Config, ScriptingApi};
pub use device_builder::{ConfigHandle, Device, DeviceBuilder};
pub use error::{Error, Result};
pub use platform::{Arc, MaybeSend, MaybeSync, Platform, StatusCode};

#[cfg(target_os = "none")]
pub use platform::{PlatformPeripherals, WifiCredentials};
