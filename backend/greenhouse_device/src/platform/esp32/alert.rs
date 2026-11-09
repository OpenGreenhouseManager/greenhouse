//! Sending alerts on the ESP32.
//!
//! The network stack and TLS context are owned by an alert task spawned in
//! [`crate::Platform::start`]; `send_alert` hands the request to it and waits
//! for the result, so `trigger_alert(config, alert).await` works from any
//! handler, like on the host.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, with_deadline, with_timeout};
use mbedtls_rs::Tls;

use super::http_client;
use crate::error::{Error, Result};

/// Upper bound for one alert including DNS, TCP and the TLS handshake.
const ALERT_TIMEOUT: Duration = Duration::from_secs(30);

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

/// POSTs a serialized alert to `url`, authenticated with the device's token.
///
/// `https://` URLs are verified against the root CAs in `certs/roots.pem`
/// (chain, signature and hostname). Certificate validity dates are **not**
/// checked: the device has no wall clock.
pub(crate) async fn send_alert(url: String, token: String, body: Vec<u8>) -> Result<()> {
    if !ALERT_TASK_RUNNING.load(Ordering::Relaxed) {
        return Err(Error::NotInitialized);
    }

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
        // Bounded here too, so a hung peer cannot keep the task (and every
        // later alert) busy after the caller has given up.
        let result = with_timeout(ALERT_TIMEOUT, send(stack, &tls, &job))
            .await
            .unwrap_or(Err(Error::Timeout));
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
