//! Outbound HTTP (weather, public IP, calendars). Always time-bounded so
//! offline DNS/TCP cannot stall RPC handlers.

use std::sync::OnceLock;
use std::time::Duration;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

fn parse_ms(var: &str) -> Option<Duration> {
    std::env::var(var)
        .ok()?
        .parse::<u64>()
        .ok()
        .map(|n| Duration::from_millis(n.max(50)))
}

/// Total request timeout (connect + headers + body).
pub fn request_timeout() -> Duration {
    parse_ms("AURA_HTTP_TIMEOUT_MS").unwrap_or(DEFAULT_REQUEST_TIMEOUT)
}

pub fn connect_timeout() -> Duration {
    parse_ms("AURA_HTTP_CONNECT_TIMEOUT_MS")
        .unwrap_or(DEFAULT_CONNECT_TIMEOUT)
        .min(request_timeout())
}

fn build_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .connect_timeout(connect_timeout())
        .pool_max_idle_per_host(4)
        .build()
        .expect("reqwest client")
}

/// Shared client with the default 8s budget. Clone is cheap (Arc).
pub fn client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| build_client(DEFAULT_REQUEST_TIMEOUT))
        .clone()
}

/// Fresh client — used when tests set `AURA_HTTP_TIMEOUT_MS` or a call needs a
/// tighter budget than the process-wide default.
pub fn client_with_timeout(timeout: Duration) -> reqwest::Client {
    build_client(timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_timeout_never_exceeds_request_timeout() {
        assert!(connect_timeout() <= request_timeout());
    }
}
