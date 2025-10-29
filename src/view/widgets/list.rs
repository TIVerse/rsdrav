//! List widget for displaying scrollable collections
//!
//! A vertical list of items with selection, scrolling, and keyboard navigation.

use crate::event::{Event, EventResult, KeyCode};
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};
use std::sync::Arc;

/// Scrollable list widget with selection
///
/// Displays a collection of items with keyboard navigation and visual selection.
/// Items can be any type that implements the render callback.
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let items = Signal::new(vec!["Apple", "Banana", "Cherry"]);
/// let selected = Signal::new(Some(0));
///
/// let list = List::new(items, selected)
///     .render_item(|item, selected| {
///         let style = if selected {
///             Style::default().bg(Color::BLUE)
///         } else {
///             Style::default()
///         };
///         ViewNode::text_styled(item.to_string(), style)
///     });
/// ```
#[allow(clippy::type_complexity)]
pub struct List<T> {
    items: Signal<Vec<T>>,
    selected: Signal<Option<usize>>,
    scroll_offset: usize,
    visible_height: usize,
    render_item: Arc<dyn Fn(&T, bool) -> ViewNode + Send + Sync>,
    style: ListStyle,
}

#[derive(Clone)]
struct ListStyle {
    normal: Style,
    selected: Style,
    focused_selected: Style,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            normal: Style::default(),
            selected: Style::default().bg(Color::rgb(60, 60, 80)),
            focused_selected: Style::default()
                .bg(Color::BLUE)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> List<T> {
    /// Create a new list
    ///
    /// - `items`: Signal containing the list items
    /// - `selected`: Signal containing the selected index (None = no selection)
    pub fn new(items: Signal<Vec<T>>, selected: Signal<Option<usize>>) -> Self {
        Self {
            items,
            selected,
            scroll_offset: 0,
            visible_height: 10, // Default, will be updated based on available space
            render_item: Arc::new(|_item, _selected| {
                // Default renderer - just use Debug
                ViewNode::text(format!("{:?}", std::any::type_name::<T>()))
            }),
            style: ListStyle::default(),
        }
    }

    /// Set custom item renderer
    ///
    /// The callback receives the item and whether it's currently selected.
    pub fn render_item<F>(mut self, f: F) -> Self
    where
        F: Fn(&T, bool) -> ViewNode + Send + Sync + 'static,
    {
        self.render_item = Arc::new(f);
        self
    }

    /// Set visible height (number of items shown at once)
    pub fn visible_height(mut self, height: usize) -> Self {
        self.visible_height = height;
        self
    }

    /// Select next item (Down arrow)
    fn select_next(&mut self) {
        let items = self.items.get();
        if items.is_empty() {
            return;
        }

        let current = self.selected.get();
        let next = match current {
            None => Some(0),
            Some(idx) => {
                if idx + 1 < items.len() {
                    Some(idx + 1)
                } else {
                    Some(idx) // Stay at last item
                }
            }
        };

        self.selected.set(next);
        self.ensure_visible(next.unwrap());
    }

    /// Select previous item (Up arrow)
    fn select_prev(&mut self) {
        let items = self.items.get();
        if items.is_empty() {
            return;
        }

        let current = self.selected.get();
        let prev = match current {
            None => Some(items.len() - 1),
            Some(idx) => {
                if idx > 0 {
                    Some(idx - 1)
                } else {
                    Some(0) // Stay at first item
                }
            }
        };

        self.selected.set(prev);
        self.ensure_visible(prev.unwrap());
    }

    /// Jump to first item (Home)
    fn select_first(&mut self) {
        let items = self.items.get();
        if !items.is_empty() {
            self.selected.set(Some(0));
            self.scroll_offset = 0;
        }
    }

    /// Jump to last item (End)
    fn select_last(&mut self) {
        let items = self.items.get();
        if !items.is_empty() {
            let last = items.len() - 1;
            self.selected.set(Some(last));
            self.ensure_visible(last);
        }
    }

    /// Page down
    fn page_down(&mut self) {
        let items = self.items.get();
        if items.is_empty() {
            return;
        }

        let current = self.selected.get().unwrap_or(0);
        let next = (current + self.visible_height).min(items.len() - 1);
        self.selected.set(Some(next));
        self.ensure_visible(next);
    }

    /// Page up
    fn page_up(&mut self) {
        let items = self.items.get();
        if items.is_empty() {
            return;
        }

        let current = self.selected.get().unwrap_or(0);
        let prev = current.saturating_sub(self.visible_height);
        self.selected.set(Some(prev));
        self.ensure_visible(prev);
    }

    /// Ensure selected item is visible (adjust scroll offset)
    fn ensure_visible(&mut self, index: usize) {
        // Scroll down if selected is below visible area
        if index >= self.scroll_offset + self.visible_height {
            self.scroll_offset = index - self.visible_height + 1;
        }
        // Scroll up if selected is above visible area
        else if index < self.scroll_offset {
            self.scroll_offset = index;
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Component for List<T> {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        let items = self.items.get();
        let selected_idx = self.selected.get();

        if items.is_empty() {
            return ViewNode::text_styled("(empty list)", Style::default().fg(Color::GRAY));
        }

        // Render visible items only (scrolling viewport)
        let end = (self.scroll_offset + self.visible_height).min(items.len());
        let visible_items = &items[self.scroll_offset..end];

        let mut children = Vec::new();

        for (offset, item) in visible_items.iter().enumerate() {
            let absolute_idx = self.scroll_offset + offset;
            let is_selected = selected_idx == Some(absolute_idx);

            // Render item with custom renderer
            let mut item_node = (self.render_item)(item, is_selected);

            // Apply selection styling if selected
            if is_selected {
                // Wrap in styled container
                match item_node {
                    ViewNode::Text { content, style } => {
                        item_node = ViewNode::Text {
                            content: format!("> {}", content),
                            style: style.bg(self.style.focused_selected.bg.unwrap_or(Color::BLUE)),
                        };
                    }
                    _ => {
                        // For other node types, just add indicator
                        children.push(ViewNode::text_styled("> ", self.style.focused_selected));
                    }
                }
            } else {
                // Add spacing for non-selected items
                if let ViewNode::Text { content, style } = item_node {
                    item_node = ViewNode::Text {
                        content: format!("  {}", content),
                        style,
                    };
                }
            }

            children.push(item_node);
        }

        // Add scroll indicator if needed
        let total_items = items.len();
        if total_items > self.visible_height {
            let scroll_info = format!(
                "  [↕ {}-{} of {}]",
                self.scroll_offset + 1,
                end,
                total_items
            );
            children.push(ViewNode::text_styled(
                scroll_info,
                Style::default().fg(Color::GRAY),
            ));
        }

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.select_prev();
                    EventResult::Handled
                }
                KeyCode::Down => {
                    self.select_next();
                    EventResult::Handled
                }
                KeyCode::Home => {
                    self.select_first();
                    EventResult::Handled
                }
                KeyCode::End => {
                    self.select_last();
                    EventResult::Handled
                }
                KeyCode::PageUp => {
                    self.page_up();
                    EventResult::Handled
                }
                KeyCode::PageDown => {
                    self.page_down();
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
    use crate::layout::Rect;
    use crate::render::Buffer;
    use crate::state::Store;

    #[test]
    fn test_list_creation() {
        let items = Signal::new(vec![1, 2, 3]);
        let selected = Signal::new(Some(0));
        let list = List::new(items, selected);

        assert_eq!(list.scroll_offset, 0);
        assert_eq!(list.visible_height, 10);
    }

    #[test]
    fn test_select_next() {
        let items = Signal::new(vec!["a", "b", "c"]);
        let selected = Signal::new(Some(0));
        let mut list = List::new(items.clone(), selected.clone());

        list.select_next();
        assert_eq!(selected.get(), Some(1));

        list.select_next();
        assert_eq!(selected.get(), Some(2));

        // Should stay at last
        list.select_next();
        assert_eq!(selected.get(), Some(2));
    }

    #[test]
    fn test_select_prev() {
        let items = Signal::new(vec!["a", "b", "c"]);
        let selected = Signal::new(Some(2));
        let mut list = List::new(items, selected.clone());

        list.select_prev();
        assert_eq!(selected.get(), Some(1));

        list.select_prev();
        assert_eq!(selected.get(), Some(0));

        // Should stay at first
        list.select_prev();
        assert_eq!(selected.get(), Some(0));
    }

    #[test]
    fn test_page_navigation() {
        let items = Signal::new((0..20).collect::<Vec<_>>());
        let selected = Signal::new(Some(0));
        let mut list = List::new(items, selected.clone()).visible_height(5);

        list.page_down();
        assert_eq!(selected.get(), Some(5));

        list.page_down();
        assert_eq!(selected.get(), Some(10));

        list.page_up();
        assert_eq!(selected.get(), Some(5));
    }

    #[test]
    fn test_scrolling() {
        let items = Signal::new((0..20).collect::<Vec<_>>());
        let selected = Signal::new(Some(0));
        let mut list = List::new(items, selected.clone()).visible_height(5);

        // Navigate down - should scroll
        for _ in 0..10 {
            list.select_next();
        }

        assert_eq!(selected.get(), Some(10));
        // Scroll offset should follow
        assert!(list.scroll_offset >= 6); // Keep selected in view
    }

    #[test]
    fn test_home_end() {
        let items = Signal::new(vec![1, 2, 3, 4, 5]);
        let selected = Signal::new(Some(2));
        let mut list = List::new(items, selected.clone());

        list.select_last();
        assert_eq!(selected.get(), Some(4));

        list.select_first();
        assert_eq!(selected.get(), Some(0));
    }

    #[test]
    fn test_empty_list() {
        let items = Signal::new(Vec::<String>::new());
        let selected = Signal::new(None);
        let mut list = List::new(items, selected.clone());

        list.select_next(); // Should not panic
        assert_eq!(selected.get(), None);
    }

    #[test]
    fn test_custom_renderer() {
        let items = Signal::new(vec!["apple", "banana"]);
        let selected = Signal::new(Some(0));

        let list = List::new(items, selected).render_item(|item, is_sel| {
            let text = if is_sel {
                format!(">>> {}", item)
            } else {
                item.to_string()
            };
            ViewNode::text(text)
        });

        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);
        let ctx = RenderContext::new(&mut buffer, area, &store);

        let node = list.render(&ctx);

        // Should use custom renderer
        match node {
            ViewNode::Container { .. } => {} // Expected
            _ => panic!("Expected container node"),
        }
    }
}
