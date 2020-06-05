//! Provides client builders for the entire crate.

use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;
use structopt::clap::crate_version;

/// User agent string for all clients.
pub const USER_AGENT: &str = concat!(
    "My IoT / ",
    crate_version!(),
    " (Rust; https://github.com/eigenein/my-iot-rs)"
);

/// Default timeout for all clients.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Returns blocking client builder.
pub fn blocking_builder() -> reqwest::blocking::ClientBuilder {
    reqwest::blocking::Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers())
        .timeout(TIMEOUT)
}

/// Returns async client builder.
pub fn async_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .default_headers(headers())
        .timeout(TIMEOUT)
}

/// Returns default header set for all clients.
fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, HeaderValue::from_static(USER_AGENT));
    headers
}
