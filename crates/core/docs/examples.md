# Examples

## Complete Application Setup

```rust
use lighty_core::{AppState, download_file, extract_archive, get_os, trace_info};

#[tokio::main]
async fn main()  {
    // Initialize tracing
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Initialize AppState FIRST
    AppState::init("MyLauncher")?;

    trace_info!("Application initialized");
    trace_info!("Running on: {:?}", get_os());

    // Get launcher directory
    trace_info!("Data dir: {:?}", AppState::data_dir());

    Ok(())
}
```

## Download and Extract Workflow

```rust
use lighty_core::{download_file, extract_archive, trace_info, trace_error};
use std::path::PathBuf;

async fn download_and_extract(
    url: &str,
    archive_path: &str,
    extract_to: &str,
    expected_sha1: Option<&str>,
)  {
    // Download
    trace_info!("Downloading from: {}", url);
    download_file(url, archive_path, expected_sha1).await?;
    trace_info!("Download complete");

    // Extract
    trace_info!("Extracting to: {}", extract_to);
    extract_archive(archive_path, extract_to).await?;
    trace_info!("Extraction complete");

    // Clean up archive
    tokio::fs::remove_file(archive_path).await?;
    trace_info!("Cleaned up archive");

    Ok(())
}
```

## Platform-Specific Logic

```rust
use lighty_core::{get_os, get_architecture, OS, Architecture};

fn get_native_library() -> &'static str {
    match (get_os(), get_architecture()) {
        (OS::Windows, Architecture::X86_64) => "natives-windows-x64",
        (OS::Windows, Architecture::X86) => "natives-windows-x86",
        (OS::MacOS, Architecture::Aarch64) => "natives-macos-arm64",
        (OS::MacOS, Architecture::X86_64) => "natives-macos-x64",
        (OS::Linux, Architecture::X86_64) => "natives-linux-x64",
        _ => panic!("Unsupported platform"),
    }
}

fn main() {
    let lib = get_native_library();
    println!("Using native library: {}", lib);
}
```

## Concurrent Downloads

```rust
use lighty_core::{download_file, trace_info};
use futures::future::join_all;

async fn download_multiple_files(
    files: Vec<(&str, &str, Option<&str>)> // (url, path, sha1)
)  {
    trace_info!("Starting {} downloads", files.len());

    let downloads: Vec<_> = files
        .into_iter()
        .map(|(url, path, sha1)| download_file(url, path, sha1))
        .collect();

    let results = join_all(downloads).await;

    let mut errors = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_) => trace_info!("Download {} completed", i + 1),
            Err(e) => errors.push((i, e)),
        }
    }

    if !errors.is_empty() {
        eprintln!("{} downloads failed", errors.len());
        return Err("Some downloads failed".into());
    }

    trace_info!("All downloads completed");
    Ok(())
}
```

## Error Handling Patterns

```rust
use lighty_core::{download_file, DownloadError, trace_error};

async fn robust_download(url: &str, path: &str)  {
    match download_file(url, path, None).await {
        Ok(_) => {
            println!("Download successful");
            Ok(())
        }
        Err(DownloadError::NetworkError(e)) => {
            trace_error!("Network error: {}", e);
            Err("Check your internet connection".into())
        }
        Err(DownloadError::IOError(e)) => {
            trace_error!("File system error: {}", e);
            Err("Check disk space and permissions".into())
        }
        Err(DownloadError::VerificationFailed { expected, actual }) => {
            trace_error!("Hash mismatch: expected {}, got {}", expected, actual);
            Err("File corrupted, please retry".into())
        }
    }
}
```

## Testing AppState

```rust
#[cfg(test)]
mod tests {
    use lighty_core::AppState;

    #[test]
    fn test_app_state_init() {
        let result = AppState::init("TestLauncher");
        assert!(result.is_ok());

        let name = AppState::name();
        assert_eq!(name, "TestLauncher");

        let version = AppState::app_version();
        assert!(!version.is_empty());
    }
}
```

## Integration Example

Complete launcher initialization:

```rust
use lighty_core::{AppState, get_os, trace_info};

async fn initialize_launcher()  {
    // 1. Initialize AppState
    AppState::init("MyLauncher")?;

    // 2. Get directories
    let data_dir = AppState::data_dir();

    // 3. Create required directories
    tokio::fs::create_dir_all(data_dir.join("instances")).await?;
    tokio::fs::create_dir_all(data_dir.join("assets")).await?;
    tokio::fs::create_dir_all(data_dir.join("libraries")).await?;
    tokio::fs::create_dir_all(data_dir.join("java")).await?;

    trace_info!("Launcher initialized on {:?}", get_os());
    trace_info!("Data directory: {:?}", data_dir);

    Ok(())
}
```

## See Also

- [Application State](./app_state.md)
- [Download System](./download.md)
- [Archive Extraction](./extract.md)
