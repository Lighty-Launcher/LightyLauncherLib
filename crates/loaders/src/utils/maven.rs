// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Shared HEAD/sidecar helpers for Maven artifact metadata.

use lighty_core::hosts::HTTP_CLIENT as CLIENT;

/// Fetches the expected SHA1 of a Maven artifact from its `.sha1` sidecar.
///
/// The Forge-family CDNs (Cloudflare in front of JFrog) strip custom
/// checksum headers, so the sidecar is the only authoritative source.
pub async fn fetch_maven_sha1(jar_url: &str) -> Option<String> {
    let sha1_url = format!("{}.sha1", jar_url);

    match CLIENT.get(&sha1_url).send().await {
        Ok(response) if response.status().is_success() => {
            response.text().await.ok().and_then(|text| {
                let sha1 = text.trim().split_whitespace().next()?.to_string();
                (sha1.len() == 40).then_some(sha1)
            })
        }
        _ => None,
    }
}

/// Returns a remote file's size without downloading the body (HEAD request).
pub async fn fetch_file_size(url: &str) -> Option<u64> {
    CLIENT
        .head(url)
        .send()
        .await
        .ok()?
        .headers()
        .get("content-length")?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

/// Fetches `(sha1, size)` in parallel for a single Maven artifact URL.
pub async fn fetch_maven_metadata(url: &str) -> (Option<String>, Option<u64>) {
    tokio::join!(fetch_maven_sha1(url), fetch_file_size(url))
}

/// Probes a list of Maven bases and returns the first one that serves
/// `relative_path` with a non-zero `Content-Length`.
///
/// `bases` must already have a trailing `/`.
pub async fn probe_maven_bases(bases: &[&str], relative_path: &str) -> Option<String> {
    for base in bases {
        let url = format!("{}{}", base, relative_path);
        if let Ok(resp) = CLIENT.head(&url).send().await {
            if resp.status().is_success() {
                // Some CDNs answer 200 with an empty body when the
                // artifact is missing — treat zero-length as not-found.
                let len = resp
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                if len > 0 {
                    return Some(url);
                }
            }
        }
    }
    None
}
