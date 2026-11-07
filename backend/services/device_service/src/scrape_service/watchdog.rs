//! Raises an alert while a scraped device does not respond.
//!
//! Every scrape result is fed into a per-device [`Debouncer`], so one failed
//! request (flaky Wi-Fi) is ignored. After `start_after` consecutive failures a
//! concurrent `device_unreachable` alert is fired, and it is fired again on
//! every scrape while the device stays down, which keeps it alive. After
//! `stop_after` consecutive successful scrapes it is stopped.
//!
//! The keep-alive is short, so the alert ends on its own (as `Expired`) when
//! nobody confirms it any more: if this service goes down, or the device is
//! deleted or no longer scraped. The alert's `datasource_id` is the device id.

use std::collections::HashMap;

use greenhouse_core::{
    alert_debounce::{DebounceConfig, Debouncer, Signal},
    data_storage_service_dto::alert_dto::{
        alert::{AlertKind, Severity},
        endpoints,
        post_create_alert::CreateAlertDto,
        post_stop_alert::StopAlertDto,
    },
};
use metrics::gauge;
use uuid::Uuid;

use crate::database::device::Device;

pub(crate) const UNREACHABLE_IDENTIFIER: &str = "device_unreachable";

/// Result of one scrape, as far as the watchdog cares.
pub(crate) enum Reachability {
    Reachable,
    /// The request failed (connection refused, timeout, ...). Carries a short reason.
    Unreachable(String),
}

pub(crate) struct WatchdogConfig {
    /// Base URL of the data storage service. `None` disables alerting; the
    /// `device_reachable` metric is still exported.
    pub(crate) data_storage_service: Option<String>,
    pub(crate) debounce: DebounceConfig,
    /// Keep-alive of the unreachable alert. Must outlast a scrape interval.
    pub(crate) keep_alive_secs: u32,
}

struct DeviceState {
    debouncer: Debouncer,
    /// Why the last failed scrape failed; re-sent while the alert is kept alive.
    last_failure: String,
}

pub(crate) struct Watchdog {
    client: reqwest::Client,
    config: WatchdogConfig,
    devices: HashMap<Uuid, DeviceState>,
}

impl Watchdog {
    pub(crate) fn new(config: WatchdogConfig) -> Self {
        if config.data_storage_service.is_none() {
            tracing::warn!(
                "DATA_STORAGE_SERVICE is not configured: unreachable devices will not raise alerts"
            );
        }
        Self {
            client: reqwest::Client::new(),
            config,
            devices: HashMap::new(),
        }
    }

    /// Feeds one scrape result for `device`.
    pub(crate) async fn observe(&mut self, device: &Device, result: Reachability) {
        let reachable = matches!(result, Reachability::Reachable);
        gauge!("device_reachable", "device" => device.id.to_string()).set(if reachable {
            1.0
        } else {
            0.0
        });

        let debounce = self.config.debounce;
        let state = self
            .devices
            .entry(device.id)
            .or_insert_with(|| DeviceState {
                debouncer: Debouncer::new(debounce),
                last_failure: String::new(),
            });
        if let Reachability::Unreachable(reason) = result {
            state.last_failure = reason;
        }
        let signal = state.debouncer.update(!reachable);
        let last_failure = state.last_failure.clone();

        let sent = match signal {
            Signal::Idle => return,
            // Also sent while recovering (fewer than `stop_after` good scrapes),
            // because the alert is still open and must not expire meanwhile.
            Signal::Fire { started } => {
                if started {
                    tracing::warn!(
                        "device {} ({}) is unreachable: {last_failure}",
                        device.name,
                        device.address
                    );
                }
                self.fire(device, &last_failure).await
            }
            Signal::Stop => {
                tracing::info!(
                    "device {} ({}) is reachable again",
                    device.name,
                    device.address
                );
                self.stop(device.id).await
            }
        };

        // A failed fire is repeated by the next scrape; a failed stop is covered
        // by the keep-alive running out.
        if let Err(e) = sent {
            tracing::error!("could not send watchdog alert for {}: {e}", device.id);
        }
    }

    /// Forgets devices that are no longer scraped (deleted, or scraping turned
    /// off). Their open alerts are no longer fired and expire on their own.
    pub(crate) fn retain(&mut self, scraped: &[Device]) {
        self.devices.retain(|id, _| {
            let keep = scraped.iter().any(|d| d.id == *id);
            if !keep {
                // The metrics recorder can't drop a series; NaN never matches `== 0`.
                gauge!("device_reachable", "device" => id.to_string()).set(f64::NAN);
            }
            keep
        });
    }

    async fn fire(&self, device: &Device, reason: &str) -> Result<(), String> {
        let Some(base) = &self.config.data_storage_service else {
            return Ok(());
        };
        let alert = CreateAlertDto {
            severity: Severity::Error,
            identifier: UNREACHABLE_IDENTIFIER.to_string(),
            value: Some(reason.to_string()),
            note: Some(format!(
                "Device '{}' at {} is not responding",
                device.name, device.address
            )),
            datasource_id: device.id.to_string(),
            kind: AlertKind::Concurrent,
            keep_alive_secs: Some(self.config.keep_alive_secs),
        };
        self.post(&format!("{base}{}", endpoints::ALERT), &alert)
            .await
    }

    async fn stop(&self, device_id: Uuid) -> Result<(), String> {
        let Some(base) = &self.config.data_storage_service else {
            return Ok(());
        };
        let stop = StopAlertDto {
            identifier: UNREACHABLE_IDENTIFIER.to_string(),
            datasource_id: device_id.to_string(),
            note: Some(String::from("Device is reachable again")),
        };
        self.post(
            &format!("{base}{}{}", endpoints::ALERT, endpoints::STOP),
            &stop,
        )
        .await
    }

    async fn post<T: serde::Serialize>(&self, url: &str, body: &T) -> Result<(), String> {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("{url} returned {}", response.status()))
        }
    }
}
