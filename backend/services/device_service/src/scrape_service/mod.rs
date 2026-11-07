mod error;
mod watchdog;

use error::{Error, Result};
use greenhouse_core::{
    alert_debounce::DebounceConfig,
    smart_device_dto::{Type, read::ReadResponseDto},
};
use metrics::gauge;
use std::time::{Duration, Instant};
use uuid::Uuid;
use watchdog::{Reachability, Watchdog, WatchdogConfig};

use crate::{AppState, database::device::Device};

pub(crate) fn start_scrape_devices(state: AppState) {
    tokio::spawn(async move {
        let gauge = gauge!(
            "scrape_service_duration",
            &[("device", "scripting_service")]
        );
        let mut watchdog = Watchdog::new(WatchdogConfig {
            data_storage_service: state.config.data_storage_service.clone(),
            debounce: DebounceConfig {
                start_after: state.config.watchdog_start_after,
                stop_after: state.config.watchdog_stop_after,
            },
            keep_alive_secs: state.config.watchdog_keep_alive_secs,
        });
        loop {
            let scrape_interval = Duration::from_secs(5);
            let now = Instant::now();
            if let Err(e) = scrape_devices(state.clone(), &mut watchdog).await {
                sentry::capture_error(&e);
                tracing::error!("Error scraping devices: {:?}", e);
            }
            let elapsed = now.elapsed();
            gauge.set(elapsed.as_secs_f64());

            tracing::debug!("Scrape service took {} seconds", elapsed.as_secs_f64());

            if elapsed < scrape_interval {
                tokio::time::sleep(scrape_interval - elapsed).await;
                continue;
            }
            // sentry Error service is to slow
            sentry::capture_error(&Error::ServiceIsTooSlow);
            tracing::error!(
                "Service is too slow and took {} seconds",
                elapsed.as_secs_f64()
            );
        }
    });
}

async fn scrape_devices(state: AppState, watchdog: &mut Watchdog) -> Result<()> {
    let devices = Device::get_scraping_devices(&state.pool).await?;

    // Scrape concurrently: one slow or dead device must not delay the others.
    let results = futures::future::join_all(devices.iter().map(|device| {
        tracing::debug!("Scraping device: {}", device.address);
        read_device(device.id, device.address.clone())
    }))
    .await;

    for (device, result) in devices.iter().zip(results) {
        let reachability = match result {
            Ok(()) => {
                tracing::debug!("Device scraped successfully");
                Reachability::Reachable
            }
            Err(Error::Unreachable(reason)) => {
                tracing::debug!("Device {} unreachable: {reason}", device.address);
                Reachability::Unreachable(reason)
            }
            // The device answered, so it is reachable; the payload is a different problem.
            Err(e) => {
                tracing::error!("Error scraping device {}: {:?}", device.address, e);
                Reachability::Reachable
            }
        };
        watchdog.observe(device, reachability).await;
    }
    watchdog.retain(&devices);

    Ok(())
}

async fn read_device(id: Uuid, address: String) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{address}/read"))
        .timeout(Duration::from_secs(4))
        .send()
        .await
        .map_err(|e| Error::Unreachable(describe_request_error(&e)))?;

    let response: ReadResponseDto = response
        .error_for_status()
        .map_err(|_| Error::Request)?
        .json()
        .await
        .map_err(|_| Error::Json)?;

    generate_metric(format!("scrape_service_duration_{id}"), &response.data);

    Ok(())
}

fn describe_request_error(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        String::from("timeout")
    } else if e.is_connect() {
        String::from("connection failed")
    } else {
        String::from("request failed")
    }
}

fn generate_metric(name: String, data: &Type) {
    match data {
        Type::Number(data) => {
            let gauge = gauge!(name, &[("type", "number")]);
            gauge.set(*data);
        }
        Type::Boolean(data) => {
            let gauge = gauge!(name, &[("type", "boolean")]);
            let b = if *data { 1.0 } else { 0.0 };
            gauge.set(b);
        }
        Type::Object(data) => {
            for (key, value) in data.iter() {
                let next_name = format!("{name}_{key}");
                generate_metric(next_name, value);
            }
        }
        Type::Measurement(data) => {
            let gauge = gauge!(name, "type" => "measurement", "unit" => data.unit.clone());
            gauge.set(data.value);
        }
        Type::Stream => {
            tracing::debug!("Not implemented: received stream");
        }
        Type::None => {
            tracing::debug!("received none");
        }
    }
}
