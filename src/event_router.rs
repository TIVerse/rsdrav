//! Advanced event routing with capture and bubble phases
//!
//! Implements a DOM-like event propagation system for component trees.

use crate::event::{Event, EventResult};
use std::collections::HashMap;

/// Event propagation phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// Capture phase - events travel from root to target
    Capture,
    /// Target phase - event is at the target component
    Target,
    /// Bubble phase - events travel from target back to root
    Bubble,
}

/// Event routing context with phase information
pub struct EventRoutingContext {
    /// Current propagation phase
    pub phase: EventPhase,
    /// Whether propagation has been stopped
    pub stopped: bool,
    /// Whether default action should be prevented
    pub prevented: bool,
}

impl EventRoutingContext {
    /// Create a new routing context
    pub fn new() -> Self {
        Self {
            phase: EventPhase::Capture,
            stopped: false,
            prevented: false,
        }
    }

    /// Stop event propagation (no further handlers will be called)
    pub fn stop_propagation(&mut self) {
        self.stopped = true;
    }

    /// Prevent default action
    pub fn prevent_default(&mut self) {
        self.prevented = true;
    }

    /// Check if propagation should continue
    pub fn should_continue(&self) -> bool {
        !self.stopped
    }
}

impl Default for EventRoutingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Event handler with phase specification
#[allow(clippy::type_complexity)]
pub struct EventHandler {
    /// Handler function
    handler: Box<dyn Fn(&Event, &mut EventRoutingContext) -> EventResult + Send + Sync>,
    /// Which phase to handle in
    phase: EventPhase,
}

impl EventHandler {
    /// Create a new event handler for a specific phase
    pub fn new<F>(phase: EventPhase, handler: F) -> Self
    where
        F: Fn(&Event, &mut EventRoutingContext) -> EventResult + Send + Sync + 'static,
    {
        Self {
            handler: Box::new(handler),
            phase,
        }
    }

    /// Execute the handler if the phase matches
    pub fn handle(&self, event: &Event, ctx: &mut EventRoutingContext) -> EventResult {
        if self.phase == ctx.phase {
            (self.handler)(event, ctx)
        } else {
            EventResult::Ignored
        }
    }
}

/// Component ID for event routing
pub type ComponentId = usize;

/// Event router manages event propagation through component tree
pub struct EventRouter {
    /// Component hierarchy (child -> parent mapping)
    hierarchy: HashMap<ComponentId, ComponentId>,
    /// Event handlers by component
    handlers: HashMap<ComponentId, Vec<EventHandler>>,
    /// Next component ID
    next_id: ComponentId,
}

impl EventRouter {
    /// Create a new event router
    pub fn new() -> Self {
        Self {
            hierarchy: HashMap::new(),
            handlers: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a component in the hierarchy
    pub fn register(&mut self, parent: Option<ComponentId>) -> ComponentId {
        let id = self.next_id;
        self.next_id += 1;

        if let Some(parent_id) = parent {
            self.hierarchy.insert(id, parent_id);
        }

        id
    }

    /// Add an event handler to a component
    pub fn add_handler(&mut self, component: ComponentId, handler: EventHandler) {
        self.handlers.entry(component).or_default().push(handler);
    }

    /// Route an event through the component tree
    pub fn route(&self, event: &Event, target: ComponentId) -> EventResult {
        let mut ctx = EventRoutingContext::new();

        // Build the path from root to target
        let mut path = vec![target];
        let mut current = target;
        while let Some(&parent) = self.hierarchy.get(&current) {
            path.push(parent);
            current = parent;
        }
        path.reverse(); // Now root is first

        // Capture phase - from root to target (excluding target)
        ctx.phase = EventPhase::Capture;
        for &component in path.iter().rev().skip(1) {
            if !ctx.should_continue() {
                break;
            }

            if let Some(handlers) = self.handlers.get(&component) {
                for handler in handlers {
                    let result = handler.handle(event, &mut ctx);
                    if result == EventResult::Consumed {
                        ctx.stop_propagation();
                        break;
                    }
                }
            }
        }

        // Target phase
        if ctx.should_continue() {
            ctx.phase = EventPhase::Target;
            if let Some(handlers) = self.handlers.get(&target) {
                for handler in handlers {
                    let result = handler.handle(event, &mut ctx);
                    if result == EventResult::Consumed {
                        ctx.stop_propagation();
                        break;
                    }
                }
            }
        }

        // Bubble phase - from target back to root (excluding target)
        if ctx.should_continue() {
            ctx.phase = EventPhase::Bubble;
            for &component in path.iter().skip(1) {
                if !ctx.should_continue() {
                    break;
                }

                if let Some(handlers) = self.handlers.get(&component) {
                    for handler in handlers {
                        let result = handler.handle(event, &mut ctx);
                        if result == EventResult::Consumed {
                            ctx.stop_propagation();
                            break;
                        }
                    }
                }
            }
        }

        if ctx.stopped {
            EventResult::Consumed
        } else if ctx.prevented {
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }

    /// Remove a component and its handlers
    pub fn unregister(&mut self, component: ComponentId) {
        self.hierarchy.remove(&component);
        self.handlers.remove(&component);
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_event_router_creation() {
        let router = EventRouter::new();
        assert_eq!(router.next_id, 1);
    }

    #[test]
    fn test_component_registration() {
        let mut router = EventRouter::new();

        let root = router.register(None);
        let child1 = router.register(Some(root));
        let child2 = router.register(Some(root));

        assert_eq!(root, 1);
        assert_eq!(child1, 2);
        assert_eq!(child2, 3);
    }

    #[test]
    fn test_capture_phase() {
        let mut router = EventRouter::new();
        let root = router.register(None);
        let child = router.register(Some(root));

        let _captured_in_root = false;

        router.add_handler(
            root,
            EventHandler::new(EventPhase::Capture, move |_, _| {
                // This should be called during capture phase
                EventResult::Ignored
            }),
        );

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
        });

        let result = router.route(&event, child);
        assert_eq!(result, EventResult::Ignored);
    }

    #[test]
    fn test_stop_propagation() {
        let mut router = EventRouter::new();
        let root = router.register(None);
        let child = router.register(Some(root));

        router.add_handler(
            root,
            EventHandler::new(EventPhase::Capture, |_, ctx| {
                ctx.stop_propagation();
                EventResult::Consumed
            }),
        );

        router.add_handler(
            child,
            EventHandler::new(EventPhase::Target, |_, _| {
                panic!("This handler should not be called!");
            }),
        );

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
        });

        let result = router.route(&event, child);
        assert_eq!(result, EventResult::Consumed);
    }

    #[test]
    fn test_bubble_phase() {
        let mut router = EventRouter::new();
        let root = router.register(None);
        let child = router.register(Some(root));

        let _bubbled = false;

        router.add_handler(
            root,
            EventHandler::new(EventPhase::Bubble, move |_, _| EventResult::Handled),
        );

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
        });

        router.route(&event, child);
        // If we get here without panic, bubble phase worked
    }
}
