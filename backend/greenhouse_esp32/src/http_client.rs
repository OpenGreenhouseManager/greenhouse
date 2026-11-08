//! Minimal HTTP/1.1 POST client over embassy-net, with optional TLS (MbedTLS).
//! Only what `trigger_alert` needs: one request per connection, status code back.

use alloc::ffi::CString;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::fmt::Write as _;
use core::net::Ipv4Addr;

use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_net::{IpAddress, Stack};
use embassy_time::Duration;
use embedded_io_async::{Read, Write};
use mbedtls_rs::{
    AuthMode, Certificate, ClientSessionConfig, Session, SessionConfig, TlsReference, X509,
};

use crate::error::{Error, Result};

/// Root CAs accepted for `https://` (see `certs/roots.pem` for the list).
const ROOT_CAS: &CStr =
    match CStr::from_bytes_with_nul(concat!(include_str!("../certs/roots.pem"), "\0").as_bytes()) {
        Ok(s) => s,
        Err(_) => panic!("roots.pem must not contain NUL bytes"),
    };

const SOCKET_TIMEOUT: Duration = Duration::from_secs(10);
const TCP_BUF_LEN: usize = 2048;
/// Enough for the status line and headers we skip over.
const RESPONSE_BUF_LEN: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Url<'a> {
    pub https: bool,
    pub host: &'a str,
    pub port: u16,
    /// Path without trailing slash; empty for the root.
    pub path: &'a str,
}

pub(crate) fn parse_url(url: &str) -> Result<Url<'_>> {
    let (https, rest) = if let Some(rest) = url.strip_prefix("https://") {
        (true, rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (false, rest)
    } else {
        return Err(Error::InvalidUrl);
    };
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, port.parse().map_err(|_| Error::InvalidUrl)?),
        None => (authority, if https { 443 } else { 80 }),
    };
    if host.is_empty() {
        return Err(Error::InvalidUrl);
    }
    Ok(Url {
        https,
        host,
        port,
        path: path.trim_end_matches('/'),
    })
}

/// POSTs `body` to `url` and returns the HTTP status code.
pub(crate) async fn post(
    stack: Stack<'_>,
    tls: TlsReference<'_>,
    url: &Url<'_>,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<u16> {
    let address = resolve(stack, url.host).await?;

    let mut rx = vec![0u8; TCP_BUF_LEN];
    let mut tx = vec![0u8; TCP_BUF_LEN];
    let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
    socket.set_timeout(Some(SOCKET_TIMEOUT));
    socket.connect((address, url.port)).await.map_err(|e| {
        log::warn!("connect to {}:{} failed: {e:?}", url.host, url.port);
        Error::Io
    })?;

    let head = request_head(url, headers, body.len());
    let status = if url.https {
        let ca_chain = Certificate::new(X509::PEM(ROOT_CAS)).map_err(|e| Error::Tls(e.code()))?;
        let server_name = CString::new(url.host).map_err(|_| Error::InvalidUrl)?;
        let config = SessionConfig::Client(ClientSessionConfig {
            ca_chain: Some(ca_chain),
            server_name: Some(&server_name),
            auth_mode: AuthMode::Required,
            ..Default::default()
        });
        let mut session = Session::new(tls, &mut socket, &config)?;
        session.connect().await?;
        let status = exchange(&mut session, &head, body).await;
        // Best effort: the status is all we need, the server may already be gone.
        let _ = session.close().await;
        status?
    } else {
        exchange(&mut socket, &head, body).await?
    };
    socket.close();
    Ok(status)
}

async fn resolve(stack: Stack<'_>, host: &str) -> Result<IpAddress> {
    if let Ok(ip) = host.parse::<Ipv4Addr>() {
        return Ok(IpAddress::Ipv4(ip));
    }
    let addresses = stack.dns_query(host, DnsQueryType::A).await.map_err(|e| {
        log::warn!("DNS lookup for {host} failed: {e:?}");
        Error::Dns
    })?;
    addresses.first().copied().ok_or(Error::Dns)
}

fn request_head(url: &Url<'_>, headers: &[(&str, &str)], content_length: usize) -> Vec<u8> {
    let mut head = alloc::string::String::new();
    // Writing to a String cannot fail.
    let _ = write!(
        head,
        "POST {} HTTP/1.1\r\nHost: {}",
        if url.path.is_empty() { "/" } else { url.path },
        url.host,
    );
    // Reverse proxies route by Host, which includes a non-default port.
    if url.port != if url.https { 443 } else { 80 } {
        let _ = write!(head, ":{}", url.port);
    }
    let _ = write!(
        head,
        "\r\nContent-Length: {content_length}\r\nConnection: close\r\n"
    );
    for (name, value) in headers {
        let _ = write!(head, "{name}: {value}\r\n");
    }
    head.push_str("\r\n");
    head.into_bytes()
}

/// Sends the request and reads just enough of the response for the status code.
async fn exchange<S>(stream: &mut S, head: &[u8], body: &[u8]) -> Result<u16>
where
    S: Read + Write,
    S::Error: core::fmt::Debug,
    Error: From<S::Error>,
{
    stream.write_all(head).await?;
    stream.write_all(body).await?;
    stream.flush().await?;

    let mut buf = [0u8; RESPONSE_BUF_LEN];
    let mut len = 0;
    loop {
        if let Some(status) = parse_status_line(&buf[..len])? {
            return Ok(status);
        }
        if len == buf.len() {
            return Err(Error::InvalidResponse);
        }
        let n = stream.read(&mut buf[len..]).await?;
        if n == 0 {
            return Err(Error::InvalidResponse);
        }
        len += n;
    }
}

/// Parses `HTTP/1.x NNN ...\r\n`; `Ok(None)` while the line is incomplete.
fn parse_status_line(buf: &[u8]) -> Result<Option<u16>> {
    let Some(end) = buf.windows(2).position(|w| w == b"\r\n") else {
        return Ok(None);
    };
    let line = core::str::from_utf8(&buf[..end]).map_err(|_| Error::InvalidResponse)?;
    let mut parts = line.split(' ');
    match (parts.next(), parts.next()) {
        (Some(version), Some(code)) if version.starts_with("HTTP/1.") => {
            code.parse().map(Some).map_err(|_| Error::InvalidResponse)
        }
        _ => Err(Error::InvalidResponse),
    }
}

impl From<embassy_net::tcp::Error> for Error {
    fn from(_: embassy_net::tcp::Error) -> Self {
        Error::Io
    }
}
