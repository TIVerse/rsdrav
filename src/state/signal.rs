use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};

type SubscriberId = u64;

/// Reactive value that notifies subscribers when it changes
///
/// This is the core building block of reactivity. When the value changes,
/// all subscribed closures get called automatically. Pretty neat!
pub struct Signal<T> {
    inner: Arc<SignalInner<T>>,
}

struct SignalInner<T> {
    value: RwLock<T>,
    // Use Weak refs so subscribers can drop without explicit cleanup
    subscribers: Mutex<Vec<(SubscriberId, Weak<dyn Fn(&T) + Send + Sync>)>>,
    version: AtomicU64, // for tracking changes in Derived
    next_sub_id: AtomicU64,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(SignalInner {
                value: RwLock::new(initial),
                subscribers: Mutex::new(Vec::new()),
                version: AtomicU64::new(0),
                next_sub_id: AtomicU64::new(0),
            }),
        }
    }

    /// Get current value (clones it out)
    pub fn get(&self) -> T {
        // Lock might be held briefly, shouldn't be a problem
        self.inner.value.read().unwrap().clone()
    }

    /// Set new value and notify subscribers
    pub fn set(&self, new_val: T) {
        {
            let mut guard = self.inner.value.write().unwrap();
            *guard = new_val.clone();
        }

        // Bump version for Derived tracking
        self.inner.version.fetch_add(1, Ordering::SeqCst);

        // Notify all subscribers
        self.notify(&new_val);
    }

    /// Update value in-place with closure
    /// Handy for things like: count.update(|val| *val += 1)
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let new_val = {
            let mut guard = self.inner.value.write().unwrap();
            f(&mut *guard);
            guard.clone()
        };

        self.inner.version.fetch_add(1, Ordering::SeqCst);
        self.notify(&new_val);
    }

    /// Get current version (for Derived dependency tracking)
    pub fn version(&self) -> u64 {
        self.inner.version.load(Ordering::SeqCst)
    }

    fn notify(&self, val: &T) {
        let mut subs = self.inner.subscribers.lock().unwrap();

        // Clean up dead subscribers while notifying
        // This keeps the subscriber list from growing forever
        subs.retain(|(_, weak)| {
            if let Some(callback) = weak.upgrade() {
                callback(val);
                true // keep
            } else {
                false // drop dead ref
            }
        });
    }

    /// Subscribe to changes
    /// Returns a Subscription handle - keep it alive to stay subscribed
    pub fn subscribe(&self, callback: impl Fn(&T) + Send + Sync + 'static) -> Subscription<T> {
        let cb = Arc::new(callback);
        let weak = Arc::downgrade(&cb) as Weak<dyn Fn(&T) + Send + Sync>;
        let id = self.inner.next_sub_id.fetch_add(1, Ordering::SeqCst);

        self.inner.subscribers.lock().unwrap().push((id, weak));

        Subscription {
            _callback: cb, // keep strong ref alive
        }
    }
}

impl<T: Clone + Send + Sync> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Subscription handle - automatically unsubscribes on drop
///
/// Just hold onto this while you want to receive notifications,
/// then drop it when you're done. No manual cleanup needed!
pub struct Subscription<T> {
    _callback: Arc<dyn Fn(&T) + Send + Sync>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn test_signal_basics() {
        let sig = Signal::new(42);
        assert_eq!(sig.get(), 42);

        sig.set(100);
        assert_eq!(sig.get(), 100);
    }

    #[test]
    fn test_signal_update() {
        let sig = Signal::new(0);
        sig.update(|val| *val += 10);
        assert_eq!(sig.get(), 10);

        sig.update(|val| *val *= 2);
        assert_eq!(sig.get(), 20);
    }

    #[test]
    fn test_signal_subscription() {
        let sig = Signal::new(0);
        let notified = Arc::new(AtomicBool::new(false));

        let n = notified.clone();
        let _sub = sig.subscribe(move |val| {
            if *val == 42 {
                n.store(true, Ordering::SeqCst);
            }
        });

        sig.set(42);
        assert!(notified.load(Ordering::SeqCst));
    }

    #[test]
    fn test_subscription_cleanup() {
        let sig = Signal::new(0);

        {
            let _sub = sig.subscribe(|_| {});
            // sub drops here
        }

        // Should not panic or leak
        sig.set(1);
        sig.set(2);
    }

    #[test]
    fn test_multiple_subscribers() {
        let sig = Signal::new(0);
        let count1 = Arc::new(AtomicU64::new(0));
        let count2 = Arc::new(AtomicU64::new(0));

        let c1 = count1.clone();
        let _sub1 = sig.subscribe(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let c2 = count2.clone();
        let _sub2 = sig.subscribe(move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        sig.set(1);
        sig.set(2);

        assert_eq!(count1.load(Ordering::SeqCst), 2);
        assert_eq!(count2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_signal_version() {
        let sig = Signal::new(0);
        let v1 = sig.version();

        sig.set(1);
        let v2 = sig.version();
        assert!(v2 > v1);

        sig.update(|val| *val += 1);
        let v3 = sig.version();
        assert!(v3 > v2);
    }
}
