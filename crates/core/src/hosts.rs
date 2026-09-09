use std::env;
use std::path::Path;
use std::time::Duration;
use once_cell::sync::Lazy;
use reqwest::Client;
use thiserror::Error;

/// Shared HTTP client tuned for parallel asset/library downloads.
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(100)
        .pool_idle_timeout(Some(Duration::from_secs(90)))

        .http2_initial_stream_window_size(Some(2 * 1024 * 1024))
        .http2_initial_connection_window_size(Some(4 * 1024 * 1024))
        .http2_adaptive_window(true)
        .http2_max_frame_size(Some(16 * 1024))

        .tcp_keepalive(Some(Duration::from_secs(60)))
        .tcp_nodelay(true)

        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(5))

        .zstd(true)
        .gzip(true)
        .brotli(true)

        .build()
        .expect("Failed to build HTTP client with default configuration - this should never fail")
});


#[cfg(target_os = "windows")]
const HOSTS_PATH: &str = "System32\\drivers\\etc\\hosts";

#[cfg(not(target_os = "windows"))]
const HOSTS_PATH: &str = "etc/hosts";

/// Hosts the launcher must be able to reach. Antivirus, parental filters and
/// cracked-launcher installers routinely blackhole these in the hosts file,
/// which breaks login; add a domain here when a launch depends on reaching it.
const HOSTS: [&str; 5] = [
    "mojang.com",
    "minecraft.net",
    "minecraftservices.com",
    "microsoftonline.com",
    "xboxlive.com",
];

/// Errors related to the hosts file check.
#[derive(Debug, Error)]
pub enum HostsError {
    #[error("Failed to read hosts file at {0}")]
    HostsReadError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type HostsResult<T> = std::result::Result<T, HostsError>;

/// Returns the hosts entries intercepting a domain the launcher needs; empty
/// means the file is clean. `extra` adds a launcher's own auth domain.
///
/// Synchronous on purpose: the file is tiny and [`crate::AppState::init`],
/// which calls it, is not async.
pub fn blocked_launcher_domains(extra: &[&str]) -> HostsResult<Vec<String>> {
    let hosts_path = if cfg!(target_os = "windows") {
        let system_drive = env::var("SystemDrive").unwrap_or("C:".to_string());
        format!("{}\\{}", system_drive, HOSTS_PATH)
    } else {
        format!("/{}", HOSTS_PATH)
    };

    if !Path::new(&hosts_path).exists() {
        return Ok(Vec::new());
    }

    let hosts_file = std::fs::read_to_string(&hosts_path)
        .map_err(|_| HostsError::HostsReadError(hosts_path.clone()))?;

    Ok(hosts_file
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .flat_map(|line| line.split_whitespace().skip(1))
        .filter(|host| is_watched_host(host, extra))
        .map(str::to_string)
        .collect())
}

/// An entry matches when it is the domain itself or one of its subdomains —
/// a `contains` test also fired on lookalikes such as mojang.com.evil.tld.
fn is_watched_host(host: &str, extra: &[&str]) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();

    HOSTS.iter().chain(extra).any(|domain| {
        host == *domain
            || host
                .strip_suffix(*domain)
                .is_some_and(|prefix| prefix.ends_with('.'))
    })
}
