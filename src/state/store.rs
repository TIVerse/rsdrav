use super::Signal;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global state store for managing signals across the app
///
/// Basically a type-safe key-value store where values are Signals.
/// Lets components share state without passing it around manually.
pub struct Store {
    inner: Arc<StoreInner>,
}

struct StoreInner {
    // Map from type-erased key to type-erased Signal
    // A bit gnarly but works well enough
    signals: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(StoreInner {
                signals: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Get or create a signal with the given key
    ///
    /// If the signal doesn't exist, it's created with `default_val`.
    /// If it exists but has a different type, this will panic (type mismatch).
    pub fn get_or_create<T>(&self, key: &str, default_val: T) -> Signal<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let mut signals = self.inner.signals.write().unwrap();

        if let Some(existing) = signals.get(key) {
            // Try to downcast to Signal<T>
            if let Some(sig) = existing.downcast_ref::<Signal<T>>() {
                return sig.clone();
            } else {
                panic!("Store key '{}' exists but has wrong type", key);
            }
        }

        // Doesn't exist, create it
        let sig = Signal::new(default_val);
        signals.insert(key.to_string(), Arc::new(sig.clone()));
        sig
    }

    /// Get an existing signal, or None if it doesn't exist
    pub fn get<T>(&self, key: &str) -> Option<Signal<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let signals = self.inner.signals.read().unwrap();
        signals
            .get(key)
            .and_then(|sig| sig.downcast_ref::<Signal<T>>().cloned())
    }

    /// Set a signal value (creates if doesn't exist)
    pub fn set<T>(&self, key: &str, value: T)
    where
        T: Clone + Send + Sync + 'static,
    {
        let sig = self.get_or_create(key, value.clone());
        sig.set(value);
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.inner.signals.read().unwrap().contains_key(key)
    }

    /// Remove a signal from the store
    pub fn remove(&self, key: &str) -> bool {
        self.inner.signals.write().unwrap().remove(key).is_some()
    }

    /// Clear all signals
    pub fn clear(&self) {
        self.inner.signals.write().unwrap().clear();
    }
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_basic() {
        let store = Store::new();

        let sig = store.get_or_create("count", 0);
        assert_eq!(sig.get(), 0);

        sig.set(42);

        // Getting again should return same signal
        let sig2 = store.get_or_create("count", 999);
        assert_eq!(sig2.get(), 42); // not 999!
    }

    #[test]
    fn test_store_set() {
        let store = Store::new();

        store.set("name", "Alice".to_string());

        let sig: Signal<String> = store.get("name").unwrap();
        assert_eq!(sig.get(), "Alice");
    }

    #[test]
    fn test_store_get_nonexistent() {
        let store = Store::new();
        let sig: Option<Signal<i32>> = store.get("nope");
        assert!(sig.is_none());
    }

    #[test]
    #[should_panic(expected = "wrong type")]
    fn test_store_type_mismatch() {
        let store = Store::new();

        store.set("value", 42_i32);

        // This should panic - trying to get as wrong type
        let _: Signal<String> = store.get_or_create("value", "nope".to_string());
    }

    #[test]
    fn test_store_contains_remove() {
        let store = Store::new();

        assert!(!store.contains("test"));

        store.set("test", 123);
        assert!(store.contains("test"));

        store.remove("test");
        assert!(!store.contains("test"));
    }

    #[test]
    fn test_store_clear() {
        let store = Store::new();

        store.set("a", 1);
        store.set("b", 2);
        store.set("c", 3);

        assert!(store.contains("a"));
        assert!(store.contains("b"));

        store.clear();

        assert!(!store.contains("a"));
        assert!(!store.contains("b"));
        assert!(!store.contains("c"));
    }
}
