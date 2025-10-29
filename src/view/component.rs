//! Component trait and lifecycle management
//!
//! Components are stateful, reusable UI elements with lifecycle hooks.

use super::{EventContext, MountContext, RenderContext, UpdateContext, ViewNode};
use crate::error::Result;
use crate::event::{Event, EventResult};

/// Core component trait - the heart of the reactive UI system
///
/// Components manage their own state and can respond to:
/// - Mounting (initialization)
/// - Events (user input)
/// - Updates (state changes)
/// - Unmounting (cleanup)
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// struct MyComponent {
///     value: Signal<i32>,
/// }
///
/// impl Component for MyComponent {
///     fn render(&self, ctx: &RenderContext) -> ViewNode {
///         ViewNode::text(format!("Value: {}", self.value.get()))
///     }
/// }
/// ```
pub trait Component: Send {
    /// Render the component to a view tree
    ///
    /// This is called every frame if the component is dirty.
    /// Should be fast - do computation in `update()` instead.
    fn render(&self, ctx: &RenderContext) -> ViewNode;

    /// Called when the component is first added to the UI
    ///
    /// Use this to:
    /// - Set up signal subscriptions
    /// - Register event handlers
    /// - Initialize resources
    fn mount(&mut self, _ctx: &mut MountContext) {
        // Default: no setup needed
    }

    /// Called when the component is removed from the UI
    ///
    /// Use this to:
    /// - Clean up resources
    /// - Cancel subscriptions (though RAII handles most of this)
    fn unmount(&mut self, _ctx: &mut MountContext) {
        // Default: no cleanup needed
    }

    /// Called when reactive state changes
    ///
    /// Return `true` if this component needs to re-render,
    /// `false` to skip rendering this frame.
    fn update(&mut self, _ctx: &mut UpdateContext) -> bool {
        // Default: always re-render on updates
        true
    }

    /// Handle an event (keyboard, mouse, etc.)
    ///
    /// Return:
    /// - `EventResult::Handled` - event was processed
    /// - `EventResult::Ignored` - pass to next handler
    /// - `EventResult::Consumed` - event processed, stop propagation
    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }
}

/// A boxed component for dynamic dispatch
pub type BoxedComponent = Box<dyn Component>;

/// Simple wrapper to make any Component easily cloneable via Arc
/// (useful for sharing components in signals)
use std::sync::Arc;

pub struct SharedComponent<C: Component> {
    inner: Arc<std::sync::Mutex<C>>,
}

impl<C: Component> SharedComponent<C> {
    pub fn new(component: C) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(component)),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, C> {
        self.inner.lock().unwrap()
    }
}

impl<C: Component> Clone for SharedComponent<C> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Helper trait for converting things into components
pub trait IntoComponent {
    type Component: Component;

    fn into_component(self) -> Self::Component;
}

// Blanket impl: any Component converts to itself
impl<C: Component> IntoComponent for C {
    type Component = C;

    fn into_component(self) -> Self::Component {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Signal;

    struct TestComponent {
        value: Signal<i32>,
        mounted: bool,
    }

    impl TestComponent {
        fn new(value: i32) -> Self {
            Self {
                value: Signal::new(value),
                mounted: false,
            }
        }
    }

    impl Component for TestComponent {
        fn render(&self, _ctx: &RenderContext) -> ViewNode {
            ViewNode::text(format!("Value: {}", self.value.get()))
        }

        fn mount(&mut self, _ctx: &mut MountContext) {
            self.mounted = true;
        }

        fn unmount(&mut self, _ctx: &mut MountContext) {
            self.mounted = false;
        }
    }

    #[test]
    fn test_component_lifecycle() {
        use crate::state::Store;

        let mut comp = TestComponent::new(42);
        let mut store = Store::new();
        let mut ctx = MountContext { store: &mut store };

        assert!(!comp.mounted);

        comp.mount(&mut ctx);
        assert!(comp.mounted);

        comp.unmount(&mut ctx);
        assert!(!comp.mounted);
    }

    #[test]
    fn test_component_render() {
        use crate::layout::Rect;
        use crate::render::Buffer;
        use crate::state::Store;

        let comp = TestComponent::new(99);
        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);

        let ctx = RenderContext::new(&mut buffer, area, &store);
        let node = comp.render(&ctx);

        // Should produce a text node
        match node {
            ViewNode::Text { content, .. } => {
                assert!(content.contains("99"));
            }
            _ => panic!("Expected text node"),
        }
    }
}
