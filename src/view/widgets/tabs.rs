//! Tabs widget for multi-view navigation
//!
//! Displays multiple views with tab navigation.

use crate::event::{Event, EventResult, KeyCode};
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};

/// Tabs widget for switching between multiple views
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let selected = Signal::new(0);
/// let overview = Text::new("Overview content");
/// let details = Text::new("Details content");
/// let settings = Text::new("Settings content");
///
/// let tabs = Tabs::new(selected)
///     .tab("Overview", overview)
///     .tab("Details", details)
///     .tab("Settings", settings);
/// ```
pub struct Tabs {
    tabs: Vec<Tab>,
    selected: Signal<usize>,
    style: TabStyle,
}

struct Tab {
    title: String,
    content: Box<dyn Component>,
}

#[derive(Clone)]
struct TabStyle {
    active: Style,
    inactive: Style,
    separator: Style,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            active: Style::default()
                .bg(Color::BLUE)
                .fg(Color::WHITE)
                .add_modifier(Modifier::BOLD),
            inactive: Style::default().fg(Color::GRAY),
            separator: Style::default().fg(Color::rgb(60, 60, 60)),
        }
    }
}

impl Tabs {
    /// Create a new tabs widget
    pub fn new(selected: Signal<usize>) -> Self {
        Self {
            tabs: Vec::new(),
            selected,
            style: TabStyle::default(),
        }
    }

    /// Add a tab with title and content
    pub fn tab(mut self, title: impl Into<String>, content: impl Component + 'static) -> Self {
        self.tabs.push(Tab {
            title: title.into(),
            content: Box::new(content),
        });
        self
    }

    /// Select next tab
    fn select_next(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        let current = self.selected.get();
        let next = (current + 1) % self.tabs.len();
        self.selected.set(next);
    }

    /// Select previous tab
    fn select_prev(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        let current = self.selected.get();
        let prev = if current == 0 {
            self.tabs.len() - 1
        } else {
            current - 1
        };
        self.selected.set(prev);
    }

    /// Render tab bar
    fn render_tab_bar(&self, selected: usize) -> ViewNode {
        let mut parts = Vec::new();

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_selected = i == selected;

            let tab_text = format!(" {} ", tab.title);

            let style = if is_selected {
                self.style.active
            } else {
                self.style.inactive
            };

            parts.push(ViewNode::text_styled(tab_text, style));

            // Add separator between tabs
            if i < self.tabs.len() - 1 {
                parts.push(ViewNode::text_styled("│", self.style.separator));
            }
        }

        ViewNode::container(parts)
    }
}

impl Component for Tabs {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        if self.tabs.is_empty() {
            return ViewNode::text_styled("(no tabs)", Style::default().fg(Color::GRAY));
        }

        let selected = self.selected.get().min(self.tabs.len() - 1);

        let mut children = Vec::new();

        // Render tab bar
        children.push(self.render_tab_bar(selected));

        // Separator line
        children.push(ViewNode::text_styled("─".repeat(60), self.style.separator));

        // Render selected tab content
        if let Some(tab) = self.tabs.get(selected) {
            children.push(tab.content.render(ctx));
        }

        // Help text
        children.push(ViewNode::text_styled(
            "  [Tab/→] Next  [Shift+Tab/←] Previous",
            Style::default().fg(Color::GRAY),
        ));

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Pass event to active tab first
        let selected = self.selected.get();
        if let Some(tab) = self.tabs.get_mut(selected) {
            let result = tab.content.handle_event(event, ctx);
            if result != EventResult::Ignored {
                return result;
            }
        }

        // Handle tab navigation
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Tab => {
                    if key.modifiers.contains(crate::event::KeyModifiers::SHIFT) {
                        self.select_prev();
                    } else {
                        self.select_next();
                    }
                    EventResult::Handled
                }
                KeyCode::Right => {
                    self.select_next();
                    EventResult::Handled
                }
                KeyCode::Left => {
                    self.select_prev();
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
    fn test_tabs_creation() {
        let selected = Signal::new(0);
        let tabs = Tabs::new(selected);
        assert_eq!(tabs.tabs.len(), 0);
    }

    #[test]
    fn test_add_tabs() {
        let selected = Signal::new(0);
        let tabs = Tabs::new(selected)
            .tab("Tab 1", Text::new("Content 1"))
            .tab("Tab 2", Text::new("Content 2"));

        assert_eq!(tabs.tabs.len(), 2);
    }

    #[test]
    fn test_tab_navigation() {
        let selected = Signal::new(0);
        let mut tabs = Tabs::new(selected.clone())
            .tab("Tab 1", Text::new("Content 1"))
            .tab("Tab 2", Text::new("Content 2"))
            .tab("Tab 3", Text::new("Content 3"));

        assert_eq!(selected.get(), 0);

        tabs.select_next();
        assert_eq!(selected.get(), 1);

        tabs.select_next();
        assert_eq!(selected.get(), 2);

        // Wrap around
        tabs.select_next();
        assert_eq!(selected.get(), 0);
    }

    #[test]
    fn test_tab_prev_navigation() {
        let selected = Signal::new(1);
        let mut tabs = Tabs::new(selected.clone())
            .tab("Tab 1", Text::new("Content 1"))
            .tab("Tab 2", Text::new("Content 2"))
            .tab("Tab 3", Text::new("Content 3"));

        tabs.select_prev();
        assert_eq!(selected.get(), 0);

        // Wrap around
        tabs.select_prev();
        assert_eq!(selected.get(), 2);
    }
}
