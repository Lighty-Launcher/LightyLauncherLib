use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify, RwLock};
use tokio::task::JoinHandle;

/// TTL-keyed async cache with thundering-herd protection.
///
/// `store` holds the actual `(value, expires_at)` entries; `fetch_locks`
/// holds per-key `Arc<Mutex<()>>` so concurrent callers asking for the
/// same key serialize behind a single fetch.
#[derive(Debug)]
pub struct Cache<K, V> {
    store: Arc<RwLock<HashMap<K, (V, Instant)>>>,
    fetch_locks: Arc<RwLock<HashMap<K, Arc<Mutex<()>>>>>,
    cleanup_notify: Arc<Notify>,
    _cleanup_handle: Option<JoinHandle<()>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Creates an empty cache without a background cleanup task.
    ///
    /// Expired entries are evicted lazily on read.
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            fetch_locks: Arc::new(RwLock::new(HashMap::new())),
            cleanup_notify: Arc::new(Notify::new()),
            _cleanup_handle: None,
        }
    }

    /// Creates a cache with a background task that evicts expired entries
    /// and sweeps orphaned fetch-locks on the same cadence.
    pub fn with_smart_cleanup() -> Self {
        let store: Arc<RwLock<HashMap<K, (V, Instant)>>> = Arc::new(RwLock::new(HashMap::new()));
        let fetch_locks: Arc<RwLock<HashMap<K, Arc<Mutex<()>>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let cleanup_notify = Arc::new(Notify::new());

        let store_bg = Arc::clone(&store);
        let fetch_locks_bg = Arc::clone(&fetch_locks);
        let notify_bg = Arc::clone(&cleanup_notify);

        let handle = tokio::spawn(async move {
            const MIN_WAIT: Duration = Duration::from_secs(1);
            const MAX_WAIT: Duration = Duration::from_secs(300);

            loop {
                let wait = {
                    let map = store_bg.read().await;
                    map.values()
                        .map(|(_, expire_at)| *expire_at)
                        .min()
                        .map(|next| {
                            next.saturating_duration_since(Instant::now())
                                .clamp(MIN_WAIT, MAX_WAIT)
                        })
                        .unwrap_or(MAX_WAIT)
                };

                // Edge-triggered Notify: future is constructed before await,
                // so notify_waiters() during the read above is captured by
                // tokio's permit and consumed on the next iteration.
                tokio::select! {
                    _ = tokio::time::sleep(wait) => {}
                    _ = notify_bg.notified() => {}
                }

                let now = Instant::now();
                let expired_keys: Vec<K> = {
                    let map = store_bg.read().await;
                    map.iter()
                        .filter(|(_, (_, exp))| now >= *exp)
                        .map(|(k, _)| k.clone())
                        .collect()
                };

                let removed = if expired_keys.is_empty() {
                    0
                } else {
                    let mut map = store_bg.write().await;
                    let now = Instant::now(); // re-check under write lock
                    let before = map.len();
                    for key in &expired_keys {
                        if let Some((_, exp)) = map.get(key) {
                            if now >= *exp {
                                map.remove(key);
                            }
                        }
                    }
                    before - map.len()
                };

                if removed > 0 {
                    lighty_core::trace_info!(
                        removed = removed,
                        "Cache cleaned expired entries"
                    );
                }

                // Sweep orphaned fetch-locks: strong_count==1 means no caller
                // holds a clone, try_lock succeeding means no waiter is active.
                let swept = {
                    let mut locks = fetch_locks_bg.write().await;
                    let before = locks.len();
                    locks.retain(|_k, lock| {
                        Arc::strong_count(lock) > 1 || lock.try_lock().is_err()
                    });
                    before - locks.len()
                };
                if swept > 0 {
                    lighty_core::trace_debug!(
                        swept = swept,
                        "Cache swept orphan fetch-locks"
                    );
                }
            }
        });

        Self {
            store,
            fetch_locks,
            cleanup_notify,
            _cleanup_handle: Some(handle),
        }
    }

    /// Inserts (or replaces) an entry with the given TTL.
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        {
            let mut store = self.store.write().await;
            let expire_at = Instant::now() + ttl;
            store.insert(key, (value, expire_at));
        }
        // Wake the cleanup task in case the new TTL is shorter than the
        // current sleep. No-op if the cache was built with `new()`.
        self.cleanup_notify.notify_waiters();
    }

    /// Returns the cached value for `key` if present and unexpired.
    pub async fn get_with_ttl(&self, key: &K) -> Option<V> {
        let store = self.store.read().await;

        if let Some((value, expire_at)) = store.get(key) {
            let now = Instant::now();
            if now < *expire_at {
                return Some(value.clone());
            }

            // Drop the read lock before requesting the write lock to
            // avoid deadlocking with any concurrent writer.
            drop(store);

            // Double-checked locking: another task may have refreshed
            // the entry between releasing the read lock and acquiring write.
            let mut store = self.store.write().await;

            match store.get(key) {
                Some((value, expire_at)) => {
                    if Instant::now() < *expire_at {
                        return Some(value.clone());
                    }
                    store.remove(key);
                }
                None => {}
            }
        }

        None
    }

    /// Get or compute with a Result-returning closure, with thundering
    /// herd protection: multiple concurrent calls for the same key only
    /// run the closure once. Errors are propagated and not cached.
    pub async fn get_or_try_insert_with<F, Fut, E>(
        &self,
        key: K,
        ttl: Duration,
        f: F,
    ) -> Result<V, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V, E>>,
    {
        if let Some(v) = self.get_with_ttl(&key).await {
            return Ok(v);
        }

        // Per-key Arc<Mutex<()>>: thundering-herd guard. Does NOT protect
        // the value store; only ensures a single in-flight fetch per key.
        let lock = {
            let mut locks = self.fetch_locks.write().await;
            locks
                .entry(key.clone())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        let _guard = lock.lock().await;

        // Double-check: another task may have populated the cache while
        // we waited on the mutex.
        if let Some(v) = self.get_with_ttl(&key).await {
            return Ok(v);
        }

        match f().await {
            Ok(value) => {
                self.insert_with_ttl(key, value.clone(), ttl).await;
                Ok(value)
            }
            Err(e) => Err(e),
        }
    }

    /// Removes every entry in the cache.
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }

    /// Keeps only the entries for which `keep(&K)` returns `true`.
    pub async fn retain<P>(&self, keep: P)
    where
        P: Fn(&K) -> bool,
    {
        let mut store = self.store.write().await;
        store.retain(|k, _| keep(k));
    }

    /// Returns the number of entries currently stored.
    pub async fn len(&self) -> usize {
        let store = self.store.read().await;
        store.len()
    }

    /// Returns `true` if [`Self::len`] would be zero.
    pub async fn is_empty(&self) -> bool {
        let store = self.store.read().await;
        store.is_empty()
    }

    /// Returns the number of pending fetch-locks currently held in the map.
    #[doc(hidden)]
    pub async fn fetch_locks_len(&self) -> usize {
        let locks = self.fetch_locks.read().await;
        locks.len()
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for Cache<K, V> {
    fn drop(&mut self) {
        if let Some(handle) = self._cleanup_handle.take() {
            handle.abort();
        }
    }
}
