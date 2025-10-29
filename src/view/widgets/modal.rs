//! Modal/Dialog widget for confirmations and alerts
//!
//! Displays content in a centered overlay box.

use crate::event::{Event, EventResult, KeyCode};
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};

/// Modal dialog widget
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let visible = Signal::new(true);
///
/// let modal = Modal::new(visible)
///     .title("Confirm")
///     .child(Text::new("Are you sure?"))
///     .closable(true);
/// ```
pub struct Modal {
    visible: Signal<bool>,
    title: Option<String>,
    child: Option<Box<dyn Component>>,
    closable: bool,
    style: ModalStyle,
}

#[derive(Clone)]
struct ModalStyle {
    border: Style,
    title: Style,
    background: Style,
}

impl Default for ModalStyle {
    fn default() -> Self {
        Self {
            border: Style::default()
                .fg(Color::CYAN)
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::YELLOW)
                .add_modifier(Modifier::BOLD),
            background: Style::default().bg(Color::rgb(30, 30, 30)),
        }
    }
}

impl Modal {
    /// Create a new modal
    pub fn new(visible: Signal<bool>) -> Self {
        Self {
            visible,
            title: None,
            child: None,
            closable: true,
            style: ModalStyle::default(),
        }
    }

    /// Set modal title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set modal content
    pub fn child(mut self, child: impl Component + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set whether modal can be closed with Esc
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Close the modal
    fn close(&self) {
        self.visible.set(false);
    }
}

impl Component for Modal {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        if !self.visible.get() {
            return ViewNode::text(""); // Hidden
        }

        let mut children = Vec::new();

        // Top border with title
        if let Some(ref title) = self.title {
            let border_line = format!("╔═══ {} ═══╗", title);
            children.push(ViewNode::text_styled(border_line, self.style.border));
        } else {
            children.push(ViewNode::text_styled("╔═════════╗", self.style.border));
        }

        // Content
        if let Some(ref child) = self.child {
            children.push(ViewNode::text_styled("║ ", self.style.border));
            children.push(child.render(ctx));
            children.push(ViewNode::text_styled(" ║", self.style.border));
        }

        // Bottom border
        children.push(ViewNode::text_styled("╚═════════╝", self.style.border));

        // Close instruction
        if self.closable {
            children.push(ViewNode::text_styled(
                "  [ESC to close]",
                Style::default().fg(Color::GRAY),
            ));
        }

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if !self.visible.get() {
            return EventResult::Ignored;
        }

        // Pass event to child first
        if let Some(ref mut child) = self.child {
            let result = child.handle_event(event, ctx);
            if result != EventResult::Ignored {
                return result;
            }
        }

        // Handle close on Esc
        if self.closable {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        self.close();
                        EventResult::Consumed
                    }
                    _ => EventResult::Ignored,
                },
                _ => EventResult::Ignored,
            }
        } else {
            EventResult::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::Text;

    #[test]
    fn test_modal_creation() {
        let visible = Signal::new(true);
        let modal = Modal::new(visible);
        assert!(modal.closable);
    }

    #[test]
    fn test_modal_visibility() {
        let visible = Signal::new(false);
        let _modal = Modal::new(visible.clone());

        assert!(!visible.get());

        // When hidden, should render empty
        // (tested via render method)
    }

    #[test]
    fn test_modal_close() {
        let visible = Signal::new(true);
        let modal = Modal::new(visible.clone()).closable(true);

        modal.close();
        assert!(!visible.get());
    }

    #[test]
    fn test_modal_with_title() {
        let visible = Signal::new(true);
        let modal = Modal::new(visible)
            .title("Test Modal")
            .child(Text::new("Content"));

        assert_eq!(modal.title, Some("Test Modal".to_string()));
    }
}
