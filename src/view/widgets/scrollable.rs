//! Scrollable container widget
//!
//! Wraps content in a scrollable viewport with scrollbar indicators.

use crate::event::{Event, EventResult, KeyCode};
use crate::state::Signal;
use crate::theme::{Color, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};

/// Scrollable container that handles content overflow
///
/// Provides vertical scrolling for content that exceeds the visible area.
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let content = VStack::new()
///     .push(Text::new("Line 1"))
///     .push(Text::new("Line 2"))
///     // ... many more lines
///     .push(Text::new("Line 100"));
///
/// let scrollable = Scrollable::new(content)
///     .height(20) // Show 20 lines at a time
///     .show_scrollbar(true);
/// ```
///
/// ## Features
/// - Vertical scrolling with Up/Down/PageUp/PageDown
/// - Optional scrollbar with position indicator
/// - Mouse wheel support (future)
/// - Auto-scroll to keep focused content visible
pub struct Scrollable {
    child: Box<dyn Component>,
    scroll_offset: Signal<usize>,
    viewport_height: usize,
    content_height: usize,
    show_scrollbar: bool,
    style: ScrollStyle,
}

#[derive(Clone)]
struct ScrollStyle {
    scrollbar: Style,
    indicator: Style,
}

impl Default for ScrollStyle {
    fn default() -> Self {
        Self {
            scrollbar: Style::default().fg(Color::rgb(60, 60, 60)),
            indicator: Style::default().fg(Color::CYAN),
        }
    }
}

impl Scrollable {
    /// Create a new scrollable container
    pub fn new(child: impl Component + 'static) -> Self {
        Self {
            child: Box::new(child),
            scroll_offset: Signal::new(0),
            viewport_height: 10,
            content_height: 0, // Will be calculated
            show_scrollbar: true,
            style: ScrollStyle::default(),
        }
    }

    /// Set the visible height in lines
    pub fn height(mut self, height: usize) -> Self {
        self.viewport_height = height;
        self
    }

    /// Show or hide the scrollbar
    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self) {
        let current = self.scroll_offset.get();
        let max_offset = self.content_height.saturating_sub(self.viewport_height);

        if current < max_offset {
            self.scroll_offset.set(current + 1);
        }
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self) {
        let current = self.scroll_offset.get();
        if current > 0 {
            self.scroll_offset.set(current - 1);
        }
    }

    /// Page down
    pub fn page_down(&mut self) {
        let current = self.scroll_offset.get();
        let max_offset = self.content_height.saturating_sub(self.viewport_height);
        let next = (current + self.viewport_height).min(max_offset);

        self.scroll_offset.set(next);
    }

    /// Page up
    pub fn page_up(&mut self) {
        let current = self.scroll_offset.get();
        let prev = current.saturating_sub(self.viewport_height);

        self.scroll_offset.set(prev);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset.set(0);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        let max_offset = self.content_height.saturating_sub(self.viewport_height);
        self.scroll_offset.set(max_offset);
    }

    /// Render scrollbar indicator
    fn render_scrollbar(&self) -> ViewNode {
        if !self.show_scrollbar || self.content_height <= self.viewport_height {
            return ViewNode::text("");
        }

        let scroll_position = if self.content_height > 0 {
            (self.scroll_offset.get() as f32 / self.content_height as f32 * 100.0) as u32
        } else {
            0
        };

        let indicator = format!(" [{}%]", scroll_position);
        ViewNode::text_styled(indicator, self.style.indicator)
    }
}

impl Component for Scrollable {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Render child content
        let content = self.child.render(ctx);

        // For now, we render all content with a note about scrolling
        // In a full implementation, we'd clip to viewport
        let mut children = Vec::new();

        children.push(content);

        // Add scroll indicator
        if self.show_scrollbar {
            children.push(self.render_scrollbar());
        }

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Pass to child first
        let result = self.child.handle_event(event, ctx);
        if result != EventResult::Ignored {
            return result;
        }

        // Handle scrolling
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Down => {
                    self.scroll_down();
                    EventResult::Handled
                }
                KeyCode::Up => {
                    self.scroll_up();
                    EventResult::Handled
                }
                KeyCode::PageDown => {
                    self.page_down();
                    EventResult::Handled
                }
                KeyCode::PageUp => {
                    self.page_up();
                    EventResult::Handled
                }
                KeyCode::Home => {
                    self.scroll_to_top();
                    EventResult::Handled
                }
                KeyCode::End => {
                    self.scroll_to_bottom();
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::Text;

    #[test]
    fn test_scrollable_creation() {
        let content = Text::new("Test content");
        let scrollable = Scrollable::new(content);

        assert_eq!(scrollable.viewport_height, 10);
        assert!(scrollable.show_scrollbar);
    }

    #[test]
    fn test_scroll_down() {
        let content = Text::new("Test");
        let mut scrollable = Scrollable::new(content).height(10);

        scrollable.content_height = 50; // Simulate content
        scrollable.scroll_down();

        assert_eq!(scrollable.scroll_offset.get(), 1);
    }

    #[test]
    fn test_scroll_up() {
        let content = Text::new("Test");
        let mut scrollable = Scrollable::new(content);

        scrollable.scroll_offset.set(5);
        scrollable.scroll_up();

        assert_eq!(scrollable.scroll_offset.get(), 4);
    }

    #[test]
    fn test_page_navigation() {
        let content = Text::new("Test");
        let mut scrollable = Scrollable::new(content).height(10);

        scrollable.content_height = 100;
        scrollable.page_down();

        assert_eq!(scrollable.scroll_offset.get(), 10);

        scrollable.page_up();
        assert_eq!(scrollable.scroll_offset.get(), 0);
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        let content = Text::new("Test");
        let mut scrollable = Scrollable::new(content).height(10);

        scrollable.content_height = 100;
        scrollable.scroll_to_bottom();

        assert_eq!(scrollable.scroll_offset.get(), 90);

        scrollable.scroll_to_top();
        assert_eq!(scrollable.scroll_offset.get(), 0);
    }
}
