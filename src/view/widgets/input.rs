//! Input widget for text entry
//!
//! A text input field with cursor positioning, validation, and styling.

use crate::event::{Event, EventResult, KeyCode, KeyModifiers};
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};

/// Text input widget with cursor and validation
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let username = Signal::new(String::new());
/// let input = Input::new(username)
///     .placeholder("Enter username")
///     .max_length(20);
/// ```
pub struct Input {
    value: Signal<String>,
    cursor_pos: usize,
    placeholder: Option<String>,
    password_mode: bool,
    max_length: Option<usize>,
    focused: bool,
    style: InputStyle,
}

#[derive(Clone)]
struct InputStyle {
    normal: Style,
    focused: Style,
    placeholder: Style,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            normal: Style::default().fg(Color::WHITE).bg(Color::rgb(40, 40, 40)),
            focused: Style::default()
                .fg(Color::WHITE)
                .bg(Color::rgb(60, 60, 80))
                .add_modifier(Modifier::BOLD),
            placeholder: Style::default().fg(Color::GRAY).bg(Color::rgb(40, 40, 40)),
        }
    }
}

impl Input {
    /// Create a new input bound to a signal
    pub fn new(value: Signal<String>) -> Self {
        Self {
            value,
            cursor_pos: 0,
            placeholder: None,
            password_mode: false,
            max_length: None,
            focused: false,
            style: InputStyle::default(),
        }
    }

    /// Set placeholder text shown when empty
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Enable password mode (shows * instead of characters)
    pub fn password(mut self) -> Self {
        self.password_mode = true;
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    /// Set focused state
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Get the display text (with password masking if needed)
    fn display_text(&self) -> String {
        let text = self.value.get();
        if text.is_empty() {
            return String::new();
        }

        if self.password_mode {
            "*".repeat(text.chars().count())
        } else {
            text
        }
    }

    /// Insert a character at the cursor position
    fn insert_char(&mut self, c: char) {
        let current = self.value.get();

        // Check max length
        if let Some(max) = self.max_length {
            if current.chars().count() >= max {
                return;
            }
        }

        // Insert character at cursor position
        let mut chars: Vec<char> = current.chars().collect();
        chars.insert(self.cursor_pos, c);
        let new_value: String = chars.into_iter().collect();

        self.value.set(new_value);
        self.cursor_pos += 1;
    }

    /// Delete character before cursor (backspace)
    fn delete_before_cursor(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        let current = self.value.get();
        let mut chars: Vec<char> = current.chars().collect();

        if self.cursor_pos <= chars.len() {
            chars.remove(self.cursor_pos - 1);
            let new_value: String = chars.into_iter().collect();
            self.value.set(new_value);
            self.cursor_pos -= 1;
        }
    }

    /// Delete character at cursor (delete key)
    fn delete_at_cursor(&mut self) {
        let current = self.value.get();
        let mut chars: Vec<char> = current.chars().collect();

        if self.cursor_pos < chars.len() {
            chars.remove(self.cursor_pos);
            let new_value: String = chars.into_iter().collect();
            self.value.set(new_value);
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self) {
        let len = self.value.get().chars().count();
        if self.cursor_pos < len {
            self.cursor_pos += 1;
        }
    }

    /// Move cursor to start
    fn move_cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Move cursor to end
    fn move_cursor_end(&mut self) {
        self.cursor_pos = self.value.get().chars().count();
    }

    /// Clear all text
    fn clear(&mut self) {
        self.value.set(String::new());
        self.cursor_pos = 0;
    }
}

impl Component for Input {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        let display = self.display_text();
        let style = if self.focused {
            self.style.focused
        } else {
            self.style.normal
        };

        // If empty, show placeholder
        if display.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                return ViewNode::text_styled(
                    format!("{} ", placeholder), // Extra space for cursor
                    self.style.placeholder,
                );
            }
        }

        // Render text with cursor
        if self.focused {
            // Insert cursor at position
            let chars: Vec<char> = display.chars().collect();
            let before: String = chars.iter().take(self.cursor_pos).collect();
            let after: String = chars.iter().skip(self.cursor_pos).collect();

            // Use | as cursor
            let with_cursor = format!("{}|{}", before, after);
            ViewNode::text_styled(with_cursor, style)
        } else {
            ViewNode::text_styled(format!("{} ", display), style)
        }
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        // Only handle events when focused
        if !self.focused {
            return EventResult::Ignored;
        }

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    // Don't handle Ctrl combinations as regular chars
                    if !key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.insert_char(c);
                        return EventResult::Handled;
                    }
                }
                KeyCode::Backspace => {
                    self.delete_before_cursor();
                    return EventResult::Handled;
                }
                KeyCode::Delete => {
                    self.delete_at_cursor();
                    return EventResult::Handled;
                }
                KeyCode::Left => {
                    self.move_cursor_left();
                    return EventResult::Handled;
                }
                KeyCode::Right => {
                    self.move_cursor_right();
                    return EventResult::Handled;
                }
                KeyCode::Home => {
                    self.move_cursor_home();
                    return EventResult::Handled;
                }
                KeyCode::End => {
                    self.move_cursor_end();
                    return EventResult::Handled;
                }
                _ => {}
            }

            // Handle Ctrl combinations
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('u') => {
                        // Ctrl+U: Clear line (common in terminals)
                        self.clear();
                        return EventResult::Handled;
                    }
                    KeyCode::Char('a') => {
                        // Ctrl+A: Move to start
                        self.move_cursor_home();
                        return EventResult::Handled;
                    }
                    KeyCode::Char('e') => {
                        // Ctrl+E: Move to end
                        self.move_cursor_end();
                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }
        }

        EventResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Rect;
    use crate::render::Buffer;
    use crate::state::Store;

    #[test]
    fn test_input_creation() {
        let value = Signal::new(String::new());
        let input = Input::new(value);
        assert_eq!(input.cursor_pos, 0);
        assert!(!input.password_mode);
    }

    #[test]
    fn test_insert_char() {
        let value = Signal::new(String::new());
        let mut input = Input::new(value.clone()).focused(true);

        input.insert_char('h');
        assert_eq!(value.get(), "h");
        assert_eq!(input.cursor_pos, 1);

        input.insert_char('i');
        assert_eq!(value.get(), "hi");
        assert_eq!(input.cursor_pos, 2);
    }

    #[test]
    fn test_delete_before_cursor() {
        let value = Signal::new("hello".to_string());
        let mut input = Input::new(value.clone()).focused(true);
        input.cursor_pos = 5;

        input.delete_before_cursor();
        assert_eq!(value.get(), "hell");
        assert_eq!(input.cursor_pos, 4);
    }

    #[test]
    fn test_delete_at_cursor() {
        let value = Signal::new("hello".to_string());
        let mut input = Input::new(value.clone()).focused(true);
        input.cursor_pos = 0;

        input.delete_at_cursor();
        assert_eq!(value.get(), "ello");
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_cursor_movement() {
        let value = Signal::new("hello".to_string());
        let mut input = Input::new(value.clone()).focused(true);
        input.cursor_pos = 2;

        input.move_cursor_left();
        assert_eq!(input.cursor_pos, 1);

        input.move_cursor_right();
        assert_eq!(input.cursor_pos, 2);

        input.move_cursor_home();
        assert_eq!(input.cursor_pos, 0);

        input.move_cursor_end();
        assert_eq!(input.cursor_pos, 5);
    }

    #[test]
    fn test_password_mode() {
        let value = Signal::new("secret".to_string());
        let input = Input::new(value).password();
        assert_eq!(input.display_text(), "******");
    }

    #[test]
    fn test_max_length() {
        let value = Signal::new(String::new());
        let mut input = Input::new(value.clone()).max_length(3).focused(true);

        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        input.insert_char('d'); // Should be ignored

        assert_eq!(value.get(), "abc");
    }

    #[test]
    fn test_placeholder() {
        let value = Signal::new(String::new());
        let input = Input::new(value).placeholder("Enter text");
        assert_eq!(input.placeholder, Some("Enter text".to_string()));
    }

    #[test]
    fn test_clear() {
        let value = Signal::new("hello".to_string());
        let mut input = Input::new(value.clone()).focused(true);
        input.cursor_pos = 5;

        input.clear();
        assert_eq!(value.get(), "");
        assert_eq!(input.cursor_pos, 0);
    }

    #[test]
    fn test_insert_at_middle() {
        let value = Signal::new("hllo".to_string());
        let mut input = Input::new(value.clone()).focused(true);
        input.cursor_pos = 1;

        input.insert_char('e');
        assert_eq!(value.get(), "hello");
        assert_eq!(input.cursor_pos, 2);
    }

    #[test]
    fn test_render_with_cursor() {
        let value = Signal::new("test".to_string());
        let mut input = Input::new(value).focused(true);
        input.cursor_pos = 2;

        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);
        let ctx = RenderContext::new(&mut buffer, area, &store);

        let node = input.render(&ctx);

        // Should have cursor at position 2: "te|st"
        match node {
            ViewNode::Text { content, .. } => {
                assert!(content.contains('|'));
                assert_eq!(content, "te|st");
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_not_focused_ignores_events() {
        use crate::layout::Rect;

        let value = Signal::new(String::new());
        let mut input = Input::new(value.clone()).focused(false);

        let mut ctx = EventContext {
            store: &mut Store::new(),
            area: Rect::new(0, 0, 40, 10),
        };

        let event = Event::Key(crate::event::KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::empty(),
        ));

        let result = input.handle_event(&event, &mut ctx);
        assert_eq!(result, EventResult::Ignored);
        assert_eq!(value.get(), ""); // No change
    }
}
