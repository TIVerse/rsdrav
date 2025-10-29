use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Computed signal derived from other signals
///
/// Lazily recomputes when accessed after dependencies change.
/// Caches the result so repeated gets are cheap.
///
/// Note: Currently requires manual invalidation when deps change.
/// Auto-tracking would be nice but adds complexity... maybe later?
pub struct Derived<T> {
    inner: Arc<DerivedInner<T>>,
}

struct DerivedInner<T> {
    compute: Box<dyn Fn() -> T + Send + Sync>,
    cached: RwLock<Option<(T, u64)>>, // (value, dep_version)
    deps_version: AtomicU64,
}

impl<T: Clone + Send + Sync + 'static> Derived<T> {
    pub fn new(compute: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(DerivedInner {
                compute: Box::new(compute),
                cached: RwLock::new(None),
                deps_version: AtomicU64::new(0),
            }),
        }
    }

    /// Get computed value (uses cache if dependencies unchanged)
    pub fn get(&self) -> T {
        let current_ver = self.inner.deps_version.load(Ordering::SeqCst);

        // Check cache first
        {
            let cached = self.inner.cached.read().unwrap();
            if let Some((ref val, ver)) = *cached {
                if ver == current_ver {
                    return val.clone();
                }
            }
        }

        // Cache miss or stale - recompute
        let new_val = (self.inner.compute)();
        *self.inner.cached.write().unwrap() = Some((new_val.clone(), current_ver));
        new_val
    }

    /// Mark dependencies as changed (call this when dependent signals change)
    ///
    /// TODO: would be great to auto-track this somehow...
    /// Maybe subscribe to all accessed signals during compute?
    /// Could work but seems tricky. Good enough for now.
    pub fn invalidate(&self) {
        self.inner.deps_version.fetch_add(1, Ordering::SeqCst);
    }
}

impl<T: Clone + Send + Sync> Clone for Derived<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Signal;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn test_derived_basic() {
        let a = Signal::new(2);
        let b = Signal::new(3);

        let sum = {
            let a = a.clone();
            let b = b.clone();
            Derived::new(move || a.get() + b.get())
        };

        assert_eq!(sum.get(), 5);

        a.set(10);
        sum.invalidate();
        assert_eq!(sum.get(), 13);

        b.set(7);
        sum.invalidate();
        assert_eq!(sum.get(), 17);
    }

    #[test]
    fn test_derived_caching() {
        let compute_count = Arc::new(AtomicU64::new(0));

        let cc = compute_count.clone();
        let derived = Derived::new(move || {
            cc.fetch_add(1, Ordering::SeqCst);
            42
        });

        // First get should compute
        assert_eq!(derived.get(), 42);
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        // Subsequent gets should use cache
        assert_eq!(derived.get(), 42);
        assert_eq!(derived.get(), 42);
        assert_eq!(compute_count.load(Ordering::SeqCst), 1);

        // After invalidation, should recompute
        derived.invalidate();
        assert_eq!(derived.get(), 42);
        assert_eq!(compute_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_derived_chain() {
        let x = Signal::new(5);

        let doubled = {
            let x = x.clone();
            Derived::new(move || x.get() * 2)
        };

        let squared = {
            let doubled = doubled.clone();
            Derived::new(move || {
                let val = doubled.get();
                val * val
            })
        };

        assert_eq!(squared.get(), 100); // (5 * 2)^2 = 100

        x.set(3);
        doubled.invalidate();
        squared.invalidate();
        assert_eq!(squared.get(), 36); // (3 * 2)^2 = 36
    }
}
