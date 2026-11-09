//! Host runtime: axum + tokio, config in a JSON file, alerts via reqwest.

mod alert;
mod server;
mod storage;

use core::future::Future;
use core::pin::Pin;
use std::path::PathBuf;
use std::sync::RwLock;

pub use axum::http::StatusCode;
pub use std::sync::Arc;

pub(crate) use alert::send_alert;
pub(crate) use server::serve;
pub(crate) use storage::ConfigStore;

/// `Send` on the host: axum runs handlers on a multi-threaded runtime.
pub trait MaybeSend: Send {}
impl<T: Send + ?Sized> MaybeSend for T {}

/// `Sync` on the host: axum runs handlers on a multi-threaded runtime.
pub trait MaybeSync: Sync {}
impl<T: Sync + ?Sized> MaybeSync for T {}

pub(crate) type BoxFuture<O> = Pin<Box<dyn Future<Output = O> + Send>>;
pub(crate) type DynFn1<A, O> = dyn Fn(A) -> BoxFuture<O> + Send + Sync;
pub(crate) type DynFn2<A, B, O> = dyn Fn(A, B) -> BoxFuture<O> + Send + Sync;

pub(crate) type Shared<V> = Arc<V>;

pub(crate) fn clone_shared<V>(shared: &Shared<V>) -> Shared<V> {
    Arc::clone(shared)
}

pub(crate) fn share<V>(value: V) -> Shared<V> {
    Arc::new(value)
}

pub(crate) type AsyncMutex<V> = tokio::sync::Mutex<V>;

pub(crate) struct ConfigCell<V>(RwLock<V>);

impl<V: Clone> ConfigCell<V> {
    pub fn new(value: V) -> Self {
        Self(RwLock::new(value))
    }

    pub fn get(&self) -> V {
        self.0.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set(&self, value: V) {
        *self.0.write().unwrap_or_else(|e| e.into_inner()) = value;
    }
}

/// Where a host device keeps its config: a JSON file, created with the
/// default config on first start.
#[derive(Debug, Clone)]
pub struct Platform {
    config_path: PathBuf,
}

impl Platform {
    pub fn new(config_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
        }
    }

    pub(crate) fn into_parts(self) -> (ConfigStore, Server) {
        (ConfigStore::new(self.config_path), Server)
    }
}

impl Default for Platform {
    /// `./config/config.json`
    fn default() -> Self {
        Self::new("./config/config.json")
    }
}

/// Nothing to carry: the server binds `0.0.0.0:{config.port}` itself.
pub(crate) struct Server;
