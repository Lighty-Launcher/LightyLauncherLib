#[macro_export]
macro_rules! join_and_mkdir {
    ($path:expr, $join:expr) => {{
        let path = $path.join($join);
        $crate::mkdir!(&path);
        path
    }};
}

#[macro_export]
macro_rules! join_and_mkdir_vec {
    ($path:expr, $joins:expr) => {{
        let mut path = $path.to_path_buf();
        for join in $joins {
            path = path.join(join);
            $crate::mkdir!(&path);
        }
        path
    }};
}

/// Fire-and-forget `create_dir_all` — logs errors via `trace_error!` and
/// swallows them. Prefer [`try_mkdir!`] when callers need to short-circuit.
#[macro_export]
macro_rules! mkdir {
    ($path:expr) => {
        if !$path.exists() {
            if let Err(e) = tokio::fs::create_dir_all(&$path).await {
                $crate::trace_error!("Failed to create directory {:?}: {}", $path, e);
            }
        }
    };
}

/// Async `create_dir_all` that propagates the `io::Error` for fail-fast pipelines.
#[macro_export]
macro_rules! try_mkdir {
    ($path:expr) => {{
        let __path = &$path;
        if __path.exists() {
            Ok::<(), std::io::Error>(())
        } else {
            tokio::fs::create_dir_all(__path).await
        }
    }};
}

#[macro_export]
macro_rules! mkdir_blocking {
    ($path:expr) => {
        if !$path.exists() {
            let path = $path.to_path_buf();
            tokio::task::spawn_blocking(move || {
                std::fs::create_dir_all(&path)
            }).await.ok();
        }
    };
}


#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! time_it {
    ($label:expr, $expr:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        let elapsed = start.elapsed();
        ::tracing::debug!(label = $label, elapsed = ?elapsed, "Operation completed");
        result
    }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! time_it {
    ($label:expr, $expr:expr) => {{
        $expr
    }};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace_debug {
    ($($arg:tt)*) => {
        ::tracing::debug!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace_debug {
    ($($arg:tt)*) => {{}};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace_info {
    ($($arg:tt)*) => {
        ::tracing::info!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace_info {
    ($($arg:tt)*) => {{}};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace_warn {
    ($($arg:tt)*) => {
        ::tracing::warn!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace_warn {
    ($($arg:tt)*) => {{}};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! trace_error {
    ($($arg:tt)*) => {
        ::tracing::error!($($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace_error {
    ($($arg:tt)*) => {{}};
}
