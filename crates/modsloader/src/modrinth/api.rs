// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shared Modrinth API endpoint, user-agent, and URL helpers.

pub const BASE_URL: &str = "https://api.modrinth.com/v2";

pub const USER_AGENT: &str = concat!(
    "Lighty-Launcher/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Lighty-Launcher/LightyLauncherLib)"
);

pub const PROVIDER: &str = "modrinth";

/// Minimal RFC3986 URL-encoder; avoids pulling a urlencoding dep.
pub fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}
