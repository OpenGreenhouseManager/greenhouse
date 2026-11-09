//! The per-platform half of the crate. Both modules export the same items:
//!
//! - public: `Platform`, `Arc`, `StatusCode`, `MaybeSend`, `MaybeSync`
//! - `BoxFuture<O>`, `DynFn1<A, O>`, `DynFn2<A, B, O>`: boxed handler types
//!   (`Send + Sync` on the host, plain on the single-threaded ESP32 executor)
//! - `Shared<V>` + `share()`: how the device state is shared with the server
//! - `ConfigCell<V>`: the current config, swapped atomically
//! - `AsyncMutex<V>`: serializes config read-modify-write
//! - `ConfigStore`: `load()` / `store()` of the serialized config
//! - `send_alert(url, token, body)`: POSTs an alert
//! - `serve(builder)`: runs the HTTP server, never returns

#[cfg(not(target_os = "none"))]
mod host;
#[cfg(not(target_os = "none"))]
pub use host::*;

#[cfg(target_os = "none")]
mod esp32;
#[cfg(target_os = "none")]
pub use esp32::*;
