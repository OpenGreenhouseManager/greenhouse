//! ESP32 smart device interface for OpenGreenhouseManager.
//!
//! Bare-metal (`no_std`) counterpart of `greenhouse_core::smart_device_interface`,
//! built on esp-hal + embassy. It serves the same HTTP contract
//! (`/read`, `/write`, `/status`, `/config`, `/activate`) and mirrors the Linux
//! API: async handlers taking `Arc<Config<T>>`, a `DeviceBuilder` with
//! `new_hybrid_device` / `new_output_device` / `new_input_device`, and
//! `device_service::trigger_alert(config, alert).await`.
//!
//! Differences from the Linux version:
//! - Board bring-up (Wi-Fi, TRNG, flash, TLS) happens in [`Platform::start`],
//!   whose result is handed to the builder instead of a config file path.
//! - The config is persisted to the `nvs` flash partition; the builder takes a
//!   default config that is used until one has been stored.
//! - [`Arc`] is `portable_atomic_util::Arc` (the ESP32-C3 has no atomic CAS)
//!   and [`StatusCode`] is a tiny local type instead of `axum::http::StatusCode`.
#![no_std]

extern crate alloc;

pub mod config;
pub mod device_builder;
pub mod device_service;
pub mod error;
mod handler;
mod http_client;
pub mod platform;
mod storage;

pub use config::{Config, ScriptingApi};
pub use device_builder::DeviceBuilder;
pub use error::{Error, Result};
pub use platform::{Platform, PlatformPeripherals, WifiCredentials};
/// `Arc` for the ESP32-C3, which lacks the atomic CAS `alloc::sync::Arc` needs.
/// Use it exactly like `std::sync::Arc`.
pub use portable_atomic_util::Arc;

/// HTTP status code returned by write handlers. Mirrors the constants of
/// `axum::http::StatusCode` used by the Linux devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub const OK: Self = StatusCode(200);
    pub const BAD_REQUEST: Self = StatusCode(400);
    pub const NOT_FOUND: Self = StatusCode(404);
    pub const UNPROCESSABLE_ENTITY: Self = StatusCode(422);
    pub const INTERNAL_SERVER_ERROR: Self = StatusCode(500);
    pub const SERVICE_UNAVAILABLE: Self = StatusCode(503);

    pub fn as_u16(self) -> u16 {
        self.0
    }
}
