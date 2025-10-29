//! Reactive state management
//!
//! Core primitives for building reactive UIs:
//! - `Signal<T>`: Mutable reactive value with auto-notification
//! - `Derived<T>`: Computed value from signals (cached)
//! - `Store`: Global state container for sharing signals

mod derived;
mod signal;
mod store;

pub use derived::Derived;
pub use signal::{Signal, Subscription};
pub use store::Store;
