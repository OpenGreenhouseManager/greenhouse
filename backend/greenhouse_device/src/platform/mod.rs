//! The per-platform half of the crate. Both modules export the same items:
//!
//! - public: `Platform` (+ `into_parts() -> (ConfigStore, Server)`), `Arc`,
//!   `StatusCode`, `MaybeSend`, `MaybeSync`
//! - `BoxFuture<O>`, `DynFn1<A, O>`, `DynFn2<A, B, O>`: boxed handler types
//!   (`Send + Sync` on the host, plain on the single-threaded ESP32 executor)
//! - `Shared<V>` + `share()` + `clone_shared()`: how the device state is shared with
//!   the server (`clone_shared` instead of `.clone()`, which lints differently on
//!   `Arc` and on the ESP32's `&'static`)
//! - `ConfigCell<V>`: the current config, swapped atomically
//! - `AsyncMutex<V>`: serializes config read-modify-write
//! - `ConfigStore`: `load()` / `store()` of the serialized config
//! - `http_post(url, headers, body)`: POSTs a request, returns the status code
//! - `Server` + `serve(state, server)`: runs the HTTP server, never returns

#[cfg(not(target_os = "none"))]
mod host;
#[cfg(not(target_os = "none"))]
pub use host::*;

#[cfg(target_os = "none")]
mod esp32;
#[cfg(target_os = "none")]
pub use esp32::*;
