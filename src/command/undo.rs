/// Trait for undoable actions
///
/// Commands implement this trait to support undo/redo functionality.
/// Each action stores the information needed to reverse itself.
pub trait UndoableAction: Send + Sync {
    /// Get the command name for display purposes
    fn command_name(&self) -> &str;

    /// Execute the undo operation (reverse the action)
    /// Returns true if successful
    fn undo(&mut self) -> bool;

    /// Execute the redo operation (reapply the action)
    /// Returns true if successful
    fn redo(&mut self) -> bool;

    /// Clone this action into a new Box
    fn clone_box(&self) -> Box<dyn UndoableAction>;
}

/// Type-erased wrapper for undoable actions
pub struct UndoAction {
    action: Box<dyn UndoableAction>,
}

impl UndoAction {
    /// Create a new undo action from any type implementing UndoableAction
    pub fn new(action: impl UndoableAction + 'static) -> Self {
        Self {
            action: Box::new(action),
        }
    }

    /// Get the command name
    pub fn command_name(&self) -> &str {
        self.action.command_name()
    }

    /// Execute undo
    pub fn undo(&mut self) -> bool {
        self.action.undo()
    }

    /// Execute redo
    pub fn redo(&mut self) -> bool {
        self.action.redo()
    }
}

impl Clone for UndoAction {
    fn clone(&self) -> Self {
        Self {
            action: self.action.clone_box(),
        }
    }
}

/// Stack for undo/redo operations
///
/// Maintains history of undoable actions with a max size.
/// When undo is called, pops from undo stack and pushes to redo stack.
pub struct UndoStack {
    undo: Vec<UndoAction>,
    redo: Vec<UndoAction>,
    max_size: usize,
}

impl UndoStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_size,
        }
    }

    /// Push an action onto the undo stack
    ///
    /// Clears the redo stack since we're on a new timeline.
    pub fn push(&mut self, action: UndoAction) {
        // Clear redo stack - we're on a new branch now
        self.redo.clear();

        // Add to undo stack
        self.undo.push(action);

        // Enforce max size
        if self.undo.len() > self.max_size {
            self.undo.remove(0);
        }
    }

    /// Pop an action from the undo stack and execute its undo operation
    ///
    /// Returns the action if successful, None if undo stack is empty.
    pub fn undo(&mut self) -> Option<UndoAction> {
        let mut action = self.undo.pop()?;

        // Execute the undo operation
        if action.undo() {
            // Clone and move to redo stack
            self.redo.push(action.clone());
            Some(action)
        } else {
            // Undo failed, put it back
            self.undo.push(action);
            None
        }
    }

    /// Pop an action from the redo stack and execute its redo operation
    ///
    /// Returns the action if successful, None if redo stack is empty.
    pub fn redo(&mut self) -> Option<UndoAction> {
        let mut action = self.redo.pop()?;

        // Execute the redo operation
        if action.redo() {
            // Clone and move to undo stack
            self.undo.push(action.clone());
            Some(action)
        } else {
            // Redo failed, put it back
            self.redo.push(action);
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Get size of undo stack
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// Get size of redo stack
    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(100) // default to 100 undo levels
    }
}

/// Example implementation of UndoableAction for a simple value change
#[derive(Clone)]
pub struct ValueChangeAction<T: Clone + Send + Sync> {
    name: String,
    old_value: T,
    new_value: T,
    current_value: T,
}

impl<T: Clone + Send + Sync + 'static> ValueChangeAction<T> {
    pub fn new(name: impl Into<String>, old: T, new: T) -> Self {
        Self {
            name: name.into(),
            old_value: old.clone(),
            new_value: new.clone(),
            current_value: new,
        }
    }

    /// Get the current value after undo/redo operations
    #[allow(dead_code)]
    pub fn current(&self) -> &T {
        &self.current_value
    }
}

impl<T: Clone + Send + Sync + 'static> UndoableAction for ValueChangeAction<T> {
    fn command_name(&self) -> &str {
        &self.name
    }

    fn undo(&mut self) -> bool {
        self.current_value = self.old_value.clone();
        true
    }

    fn redo(&mut self) -> bool {
        self.current_value = self.new_value.clone();
        true
    }

    fn clone_box(&self) -> Box<dyn UndoableAction> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_stack_push() {
        let mut stack = UndoStack::new(10);

        let action = ValueChangeAction::new("test", 10, 42);
        stack.push(UndoAction::new(action));
        assert_eq!(stack.undo_len(), 1);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_undo_stack_max_size() {
        let mut stack = UndoStack::new(3);

        stack.push(UndoAction::new(ValueChangeAction::new("cmd1", 0, 1)));
        stack.push(UndoAction::new(ValueChangeAction::new("cmd2", 0, 2)));
        stack.push(UndoAction::new(ValueChangeAction::new("cmd3", 0, 3)));
        stack.push(UndoAction::new(ValueChangeAction::new("cmd4", 0, 4)));

        // Should only keep last 3
        assert_eq!(stack.undo_len(), 3);
    }

    #[test]
    fn test_undo_redo() {
        let mut stack = UndoStack::new(10);

        let action = ValueChangeAction::new("change", 10, 20);
        stack.push(UndoAction::new(action));

        // Undo should reverse the action
        let undone = stack.undo();
        assert!(undone.is_some());
        assert_eq!(stack.undo_len(), 0);
        assert_eq!(stack.redo_len(), 1);

        // Redo should reapply the action
        let redone = stack.redo();
        assert!(redone.is_some());
        assert_eq!(stack.undo_len(), 1);
        assert_eq!(stack.redo_len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut stack = UndoStack::new(10);

        let action = ValueChangeAction::new("test", 0, 42);
        stack.push(UndoAction::new(action));
        assert_eq!(stack.undo_len(), 1);

        stack.clear();
        assert_eq!(stack.undo_len(), 0);
        assert_eq!(stack.redo_len(), 0);
    }
}
