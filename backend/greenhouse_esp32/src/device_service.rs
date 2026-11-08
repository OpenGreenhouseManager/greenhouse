//! Alerts to the scripting API, mirroring
//! `greenhouse_core::smart_device_interface::device_service`.
//!
//! The network stack and TLS context are owned by an alert task spawned in
//! [`crate::Platform::start`]; `trigger_alert` hands the request to it and
//! waits for the result, so handlers keep the Linux signature
//! (`trigger_alert(config, alert).await`).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, with_deadline};
use greenhouse_core::data_storage_service_dto::alert_dto::{
    alert::Severity, post_create_alert::CreateAlertDto,
};
use mbedtls_rs::Tls;

use crate::Arc;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::http_client;

/// Upper bound for one alert including DNS, TCP and the TLS handshake.
const ALERT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct AlertCreation {
    pub severity: Severity,
    pub identifier: String,
    pub value: Option<String>,
    pub note: Option<String>,
}

struct AlertJob {
    id: u32,
    url: String,
    token: String,
    body: Vec<u8>,
}

static ALERT_TASK_RUNNING: AtomicBool = AtomicBool::new(false);
// `AtomicU32` without CAS: only written while holding `ONE_AT_A_TIME`.
static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(0);
static ONE_AT_A_TIME: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());
static REQUEST: Signal<CriticalSectionRawMutex, AlertJob> = Signal::new();
static RESPONSE: Signal<CriticalSectionRawMutex, (u32, Result<()>)> = Signal::new();

/// POSTs an alert to the configured scripting API (`{url}/alert`).
///
/// `https://` URLs are verified against the root CAs in `certs/roots.pem`
/// (chain, signature and hostname). Certificate validity dates are **not**
/// checked: the device has no wall clock.
pub async fn trigger_alert<T>(config: Arc<Config<T>>, alert: AlertCreation) -> Result<()>
where
    T: Clone + Default,
{
    let scripting_api = config.scripting_api.as_ref().ok_or_else(|| {
        log::warn!("scripting_api missing; cannot send alert");
        Error::ScriptingApiNotConfigured
    })?;
    if !ALERT_TASK_RUNNING.load(Ordering::Relaxed) {
        return Err(Error::NotInitialized);
    }

    let dto = CreateAlertDto {
        severity: alert.severity,
        identifier: alert.identifier,
        value: alert.value,
        note: alert.note,
        datasource_id: config.datasource_id.clone(),
    };
    let body = serde_json::to_vec(&dto).map_err(|_| Error::SerializationError)?;
    let url = format!("{}/alert", scripting_api.url.trim_end_matches('/'));
    let token = scripting_api.token.clone();

    let deadline = Instant::now() + ALERT_TIMEOUT;
    let _guard = with_deadline(deadline, ONE_AT_A_TIME.lock())
        .await
        .map_err(|_| Error::Timeout)?;
    let id = NEXT_JOB_ID.load(Ordering::Relaxed).wrapping_add(1);
    NEXT_JOB_ID.store(id, Ordering::Relaxed);

    REQUEST.signal(AlertJob {
        id,
        url,
        token,
        body,
    });
    // A late answer to an earlier, timed-out caller carries an older id.
    loop {
        let (job_id, result) = with_deadline(deadline, RESPONSE.wait())
            .await
            .map_err(|_| Error::Timeout)?;
        if job_id == id {
            return result;
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn alert_task(stack: Stack<'static>, tls: Tls<'static>) {
    ALERT_TASK_RUNNING.store(true, Ordering::Relaxed);
    loop {
        let job = REQUEST.wait().await;
        let result = send(stack, &tls, &job).await;
        if let Err(e) = &result {
            log::warn!("alert POST to {} failed: {e:?}", job.url);
        }
        RESPONSE.signal((job.id, result));
    }
}

async fn send(stack: Stack<'static>, tls: &Tls<'static>, job: &AlertJob) -> Result<()> {
    let url = http_client::parse_url(&job.url)?;
    let cookie = format!("auth-token={}", job.token);
    let headers = [
        ("Content-Type", "application/json"),
        ("Cookie", cookie.as_str()),
    ];
    let status = http_client::post(stack, tls.reference(), &url, &headers, &job.body).await?;
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(Error::HttpStatus(status))
    }
}
