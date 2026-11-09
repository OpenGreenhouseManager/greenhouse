//! ESP32-C3 runtime: esp-hal + embassy, `no_std`. Config in the `nvs` flash
//! partition, HTTP via picoserve, alerts via embassy-net + mbedtls.

mod alert;
mod bring_up;
mod http_client;
mod server;
mod storage;

use alloc::boxed::Box;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;

pub(crate) use alert::http_post;
pub(crate) use bring_up::Server;
pub use bring_up::{Platform, PlatformPeripherals, WifiCredentials};
pub(crate) use server::serve;
pub(crate) use storage::ConfigStore;

/// `Arc` for the ESP32-C3, which lacks the atomic CAS `alloc::sync::Arc` needs.
/// Use it exactly like `std::sync::Arc`.
pub use portable_atomic_util::Arc;

/// HTTP status code returned by write handlers. Has the constants of
/// `axum::http::StatusCode` (the host's `StatusCode`) that devices use.
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

/// No bound on the ESP32: everything runs on one single-threaded executor.
pub trait MaybeSend {}
impl<T: ?Sized> MaybeSend for T {}

/// No bound on the ESP32: everything runs on one single-threaded executor.
pub trait MaybeSync {}
impl<T: ?Sized> MaybeSync for T {}

pub(crate) type BoxFuture<O> = Pin<Box<dyn Future<Output = O>>>;
pub(crate) type DynFn1<A, O> = dyn Fn(A) -> BoxFuture<O>;
pub(crate) type DynFn2<A, B, O> = dyn Fn(A, B) -> BoxFuture<O>;

/// The device lives for the rest of the program.
pub(crate) type Shared<V> = &'static V;

pub(crate) fn clone_shared<V>(shared: &Shared<V>) -> Shared<V> {
    shared
}

pub(crate) fn share<V>(value: V) -> Shared<V> {
    Box::leak(Box::new(value))
}

pub(crate) type AsyncMutex<V> = embassy_sync::mutex::Mutex<NoopRawMutex, V>;

/// Never borrowed across an `.await`.
pub(crate) struct ConfigCell<V>(RefCell<V>);

impl<V: Clone> ConfigCell<V> {
    pub fn new(value: V) -> Self {
        Self(RefCell::new(value))
    }

    pub fn get(&self) -> V {
        self.0.borrow().clone()
    }

    pub fn set(&self, value: V) {
        *self.0.borrow_mut() = value;
    }
}
