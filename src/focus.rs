//! Focus management system
//!
//! Tracks which component has keyboard focus and enables Tab navigation.

use std::collections::HashMap;

/// Unique identifier for a focusable component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub usize);

impl ComponentId {
    /// Create a new component ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Focus manager tracks which component has keyboard focus
///
/// Components register themselves with an ID and order.
/// Tab/Shift+Tab cycles through focusable components.
///
/// ## Example
/// ```no_run
/// use rsdrav::focus::{FocusManager, ComponentId};
///
/// let mut focus = FocusManager::new();
///
/// // Register components in tab order
/// focus.register(ComponentId::new(1), 0, true);
/// focus.register(ComponentId::new(2), 1, true);
///
/// // Navigate with tab
/// focus.focus_next(); // Focus component 1
/// focus.focus_next(); // Focus component 2
/// focus.focus_prev(); // Focus component 1
/// ```
pub struct FocusManager {
    /// Registered components in tab order
    components: Vec<FocusableComponent>,
    /// Currently focused component
    current: Option<ComponentId>,
    /// Next ID to assign
    next_id: usize,
}

#[derive(Debug, Clone)]
struct FocusableComponent {
    id: ComponentId,
    order: usize,
    focusable: bool,
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            current: None,
            next_id: 1,
        }
    }

    /// Generate a new unique component ID
    pub fn new_id(&mut self) -> ComponentId {
        let id = ComponentId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Register a focusable component
    ///
    /// - `id`: Unique component identifier
    /// - `order`: Tab order (lower numbers focused first)
    /// - `focusable`: Whether component can receive focus
    pub fn register(&mut self, id: ComponentId, order: usize, focusable: bool) {
        // Remove existing registration if present
        self.components.retain(|c| c.id != id);

        // Add new registration
        self.components.push(FocusableComponent {
            id,
            order,
            focusable,
        });

        // Keep sorted by order
        self.components.sort_by_key(|c| c.order);

        // If no focus and component is focusable, focus it
        if self.current.is_none() && focusable {
            self.current = Some(id);
        }
    }

    /// Unregister a component (when it's removed from UI)
    pub fn unregister(&mut self, id: ComponentId) {
        self.components.retain(|c| c.id != id);

        // Clear focus if this component had it
        if self.current == Some(id) {
            self.current = None;
            // Try to focus next available
            if !self.components.is_empty() {
                self.focus_next();
            }
        }
    }

    /// Get the currently focused component ID
    pub fn current(&self) -> Option<ComponentId> {
        self.current
    }

    /// Check if a specific component has focus
    pub fn is_focused(&self, id: ComponentId) -> bool {
        self.current == Some(id)
    }

    /// Set focus to a specific component
    pub fn focus(&mut self, id: ComponentId) -> bool {
        // Check if component exists and is focusable
        if let Some(comp) = self.components.iter().find(|c| c.id == id) {
            if comp.focusable {
                self.current = Some(id);
                return true;
            }
        }
        false
    }

    /// Focus the next component (Tab)
    pub fn focus_next(&mut self) -> bool {
        if self.components.is_empty() {
            return false;
        }

        // Find current position
        let current_idx = if let Some(current_id) = self.current {
            self.components.iter().position(|c| c.id == current_id)
        } else {
            None
        };

        // Start from next position (or beginning if none)
        let start_idx = current_idx.map(|i| i + 1).unwrap_or(0);

        // Search forward, wrapping around
        for offset in 0..self.components.len() {
            let idx = (start_idx + offset) % self.components.len();
            let comp = &self.components[idx];

            if comp.focusable {
                self.current = Some(comp.id);
                return true;
            }
        }

        false
    }

    /// Focus the previous component (Shift+Tab)
    pub fn focus_prev(&mut self) -> bool {
        if self.components.is_empty() {
            return false;
        }

        // Find current position
        let current_idx = if let Some(current_id) = self.current {
            self.components.iter().position(|c| c.id == current_id)
        } else {
            None
        };

        // Start from previous position (or end if none)
        let start_idx = current_idx
            .map(|i| {
                if i == 0 {
                    self.components.len() - 1
                } else {
                    i - 1
                }
            })
            .unwrap_or(self.components.len() - 1);

        // Search backward, wrapping around
        for offset in 0..self.components.len() {
            let idx = if start_idx >= offset {
                start_idx - offset
            } else {
                self.components.len() - (offset - start_idx)
            };

            let comp = &self.components[idx];

            if comp.focusable {
                self.current = Some(comp.id);
                return true;
            }
        }

        false
    }

    /// Clear all focus (no component focused)
    pub fn clear(&mut self) {
        self.current = None;
    }

    /// Clear all registered components
    pub fn clear_all(&mut self) {
        self.components.clear();
        self.current = None;
    }

    /// Get number of registered components
    pub fn count(&self) -> usize {
        self.components.len()
    }

    /// Get number of focusable components
    pub fn focusable_count(&self) -> usize {
        self.components.iter().filter(|c| c.focusable).count()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_creation() {
        let mgr = FocusManager::new();
        assert_eq!(mgr.current(), None);
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_register_component() {
        let mut mgr = FocusManager::new();
        let id = ComponentId::new(1);

        mgr.register(id, 0, true);
        assert_eq!(mgr.count(), 1);
        assert_eq!(mgr.current(), Some(id)); // Auto-focus first component
    }

    #[test]
    fn test_focus_next() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);

        mgr.register(id1, 0, true);
        mgr.register(id2, 1, true);
        mgr.register(id3, 2, true);

        assert_eq!(mgr.current(), Some(id1));

        mgr.focus_next();
        assert_eq!(mgr.current(), Some(id2));

        mgr.focus_next();
        assert_eq!(mgr.current(), Some(id3));

        // Wrap around
        mgr.focus_next();
        assert_eq!(mgr.current(), Some(id1));
    }

    #[test]
    fn test_focus_prev() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);

        mgr.register(id1, 0, true);
        mgr.register(id2, 1, true);
        mgr.register(id3, 2, true);

        assert_eq!(mgr.current(), Some(id1));

        // Go backwards from first - wraps to last
        mgr.focus_prev();
        assert_eq!(mgr.current(), Some(id3));

        mgr.focus_prev();
        assert_eq!(mgr.current(), Some(id2));

        mgr.focus_prev();
        assert_eq!(mgr.current(), Some(id1));
    }

    #[test]
    fn test_skip_non_focusable() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);

        mgr.register(id1, 0, true);
        mgr.register(id2, 1, false); // Not focusable
        mgr.register(id3, 2, true);

        assert_eq!(mgr.current(), Some(id1));

        mgr.focus_next();
        assert_eq!(mgr.current(), Some(id3)); // Skipped id2

        mgr.focus_prev();
        assert_eq!(mgr.current(), Some(id1)); // Skipped id2
    }

    #[test]
    fn test_explicit_focus() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);

        mgr.register(id1, 0, true);
        mgr.register(id2, 1, true);

        assert!(mgr.focus(id2));
        assert_eq!(mgr.current(), Some(id2));
    }

    #[test]
    fn test_unregister() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);

        mgr.register(id1, 0, true);
        mgr.register(id2, 1, true);

        mgr.focus(id1);
        mgr.unregister(id1);

        assert_eq!(mgr.count(), 1);
        // Should auto-focus next available
        assert_eq!(mgr.current(), Some(id2));
    }

    #[test]
    fn test_is_focused() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);

        mgr.register(id1, 0, true);
        mgr.register(id2, 1, true);

        assert!(mgr.is_focused(id1));
        assert!(!mgr.is_focused(id2));

        mgr.focus_next();
        assert!(!mgr.is_focused(id1));
        assert!(mgr.is_focused(id2));
    }

    #[test]
    fn test_tab_order_respected() {
        let mut mgr = FocusManager::new();
        let id1 = ComponentId::new(1);
        let id2 = ComponentId::new(2);
        let id3 = ComponentId::new(3);

        // Register out of order
        mgr.register(id3, 2, true);
        mgr.register(id1, 0, true);
        mgr.register(id2, 1, true);

        // id3 was registered first so it gets auto-focus
        // But explicit focus should work in order
        mgr.focus(id1);
        assert_eq!(mgr.current(), Some(id1));

        mgr.focus_next();
        assert_eq!(mgr.current(), Some(id2));
        mgr.focus_next();
        assert_eq!(mgr.current(), Some(id3));
    }

    #[test]
    fn test_clear() {
        let mut mgr = FocusManager::new();
        let id = ComponentId::new(1);

        mgr.register(id, 0, true);
        assert_eq!(mgr.current(), Some(id));

        mgr.clear();
        assert_eq!(mgr.current(), None);
        assert_eq!(mgr.count(), 1); // Components still registered
    }

    #[test]
    fn test_new_id() {
        let mut mgr = FocusManager::new();

        let id1 = mgr.new_id();
        let id2 = mgr.new_id();
        let id3 = mgr.new_id();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}
