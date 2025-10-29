//! Async support for background tasks and async event handlers
//!
//! Requires the `tokio` feature flag.

#[cfg(feature = "tokio")]
use std::future::Future;
#[cfg(feature = "tokio")]
use tokio::runtime::{Handle, Runtime};

use crate::error::Result;

/// Async runtime wrapper for running background tasks
#[cfg(feature = "tokio")]
pub struct AsyncRuntime {
    runtime: Runtime,
}

#[cfg(feature = "tokio")]
impl AsyncRuntime {
    /// Create a new async runtime
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(Self { runtime })
    }

    /// Spawn a background task
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Block on a future (for integration with sync code)
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Get a handle to the runtime
    pub fn handle(&self) -> Handle {
        self.runtime.handle().clone()
    }
}

#[cfg(feature = "tokio")]
impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create async runtime")
    }
}

/// Async task handle for spawning background work
#[cfg(feature = "tokio")]
pub struct AsyncTask<T> {
    handle: tokio::task::JoinHandle<T>,
}

#[cfg(feature = "tokio")]
impl<T> AsyncTask<T> {
    /// Create from a join handle
    pub fn new(handle: tokio::task::JoinHandle<T>) -> Self {
        Self { handle }
    }

    /// Check if the task is finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    /// Abort the task
    pub fn abort(&self) {
        self.handle.abort();
    }
}

/// Helper for spawning async work from sync context
#[cfg(feature = "tokio")]
pub fn spawn_task<F>(future: F) -> AsyncTask<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    AsyncTask::new(tokio::spawn(future))
}

/// Helper for running async work with timeout
#[cfg(feature = "tokio")]
pub async fn with_timeout<F>(
    duration: std::time::Duration,
    future: F,
) -> std::result::Result<F::Output, tokio::time::error::Elapsed>
where
    F: Future,
{
    tokio::time::timeout(duration, future).await
}

// Stub implementations when tokio feature is disabled
#[cfg(not(feature = "tokio"))]
pub struct AsyncRuntime;

#[cfg(not(feature = "tokio"))]
impl AsyncRuntime {
    pub fn new() -> Result<Self> {
        Err(crate::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Async support requires 'tokio' feature",
        )))
    }
}

#[cfg(test)]
#[cfg(feature = "tokio")]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_runtime_creation() {
        let runtime = AsyncRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_spawn_task() {
        let runtime = AsyncRuntime::new().unwrap();

        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        let handle = runtime.spawn(async move {
            completed_clone.store(true, Ordering::SeqCst);
            42
        });

        let result = runtime.block_on(handle).unwrap();
        assert_eq!(result, 42);
        assert!(completed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_timeout() {
        let runtime = AsyncRuntime::new().unwrap();

        let result = runtime.block_on(async {
            with_timeout(std::time::Duration::from_millis(100), async {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await
            })
            .await
        });

        assert!(result.is_ok());

        let result = runtime.block_on(async {
            with_timeout(std::time::Duration::from_millis(10), async {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await
            })
            .await
        });

        assert!(result.is_err());
    }
}
