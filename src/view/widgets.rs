//! Built-in widgets
//!
//! Widgets are pre-built components for common UI patterns.

mod input;
mod list;
mod modal;
mod progress;
mod scrollable;
mod table;
mod tabs;

pub use input::Input;
pub use list::List;
pub use modal::Modal;
pub use progress::ProgressBar;
pub use scrollable::Scrollable;
pub use table::{Column as TableColumn, SortOrder, Table};
pub use tabs::Tabs;

use super::{Component, EventContext, MountContext, RenderContext, UpdateContext, ViewNode};
use crate::event::{Event, EventResult, KeyCode, MouseButton, MouseEventKind};
use crate::layout::Rect;
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use std::cell::Cell;
use std::sync::Arc;

/// Simple text display widget
///
/// Can show static or reactive text content.
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// // Static text
/// let text = Text::new("Hello, world!");
///
/// // Reactive text
/// let count = Signal::new(0);
/// let text = Text::bind(move || format!("Count: {}", count.get()));
/// ```
pub struct Text {
    content: TextContent,
    style: Style,
}

enum TextContent {
    Static(String),
    Dynamic(Arc<dyn Fn() -> String + Send + Sync>),
}

impl Text {
    /// Create static text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            content: TextContent::Static(text.into()),
            style: Style::default(),
        }
    }

    /// Create text that updates from a signal
    pub fn bind(f: impl Fn() -> String + Send + Sync + 'static) -> Self {
        Self {
            content: TextContent::Dynamic(Arc::new(f)),
            style: Style::default(),
        }
    }

    /// Set the text style
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self
    }

    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self
    }

    /// Add text modifier (bold, italic, etc.)
    pub fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.style = self.style.add_modifier(modifier);
        self
    }

    fn get_content(&self) -> String {
        match &self.content {
            TextContent::Static(s) => s.clone(),
            TextContent::Dynamic(f) => f(),
        }
    }
}

impl Component for Text {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        ViewNode::text_styled(self.get_content(), self.style)
    }
}

/// Interactive button widget
///
/// Triggers a callback when clicked or activated.
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let count = Signal::new(0);
/// let btn = Button::new("Click me!", move || {
///     count.update(|v| *v += 1);
/// });
/// ```
pub struct Button {
    label: String,
    on_click: Arc<dyn Fn() + Send + Sync>,
    style: ButtonStyle,
    state: ButtonState,
    /// Track the last rendered position for hit-testing (using Cell for interior mutability)
    last_rect: Cell<Option<Rect>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Normal,
    Hover,
    Active,
}

#[derive(Clone)]
struct ButtonStyle {
    normal: Style,
    hover: Style,
    active: Style,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            normal: Style::default().fg(Color::WHITE).bg(Color::BLUE),
            hover: Style::default()
                .fg(Color::WHITE)
                .bg(Color::rgb(100, 150, 255))
                .add_modifier(Modifier::BOLD),
            active: Style::default()
                .fg(Color::BLACK)
                .bg(Color::CYAN)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl Button {
    /// Create a button with a label and click handler
    pub fn new(label: impl Into<String>, on_click: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            label: label.into(),
            on_click: Arc::new(on_click),
            style: ButtonStyle::default(),
            state: ButtonState::Normal,
            last_rect: Cell::new(None),
        }
    }

    /// Set custom style for normal state
    pub fn style_normal(mut self, style: Style) -> Self {
        self.style.normal = style;
        self
    }

    /// Set custom style for hover state
    pub fn style_hover(mut self, style: Style) -> Self {
        self.style.hover = style;
        self
    }

    /// Set custom style for active state
    pub fn style_active(mut self, style: Style) -> Self {
        self.style.active = style;
        self
    }

    fn get_style(&self) -> Style {
        match self.state {
            ButtonState::Normal => self.style.normal,
            ButtonState::Hover => self.style.hover,
            ButtonState::Active => self.style.active,
        }
    }

    fn activate(&self) {
        (self.on_click)();
    }
}

impl Component for Button {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Store the rendering area for hit-testing
        self.last_rect.set(Some(ctx.area));

        // Render button with [ label ] format
        let content = format!("[ {} ]", self.label);
        ViewNode::text_styled(content, self.get_style())
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Key(key) => {
                // Enter or Space activates the button when focused
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        self.state = ButtonState::Active;
                        self.activate();
                        return EventResult::Consumed;
                    }
                    _ => {}
                }
            }

            Event::Mouse(mouse) => {
                // Check if mouse is over button using stored rect
                let is_over = if let Some(rect) = self.last_rect.get() {
                    rect.contains(mouse.x, mouse.y)
                } else {
                    false
                };

                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if is_over {
                            self.state = ButtonState::Active;
                            return EventResult::Handled;
                        }
                    }
                    MouseEventKind::Up(MouseButton::Left) => {
                        if self.state == ButtonState::Active {
                            if is_over {
                                self.activate();
                            }
                            self.state = ButtonState::Normal;
                            return EventResult::Consumed;
                        }
                    }
                    MouseEventKind::Moved => {
                        // Set hover state based on position
                        if is_over && self.state == ButtonState::Normal {
                            self.state = ButtonState::Hover;
                        } else if !is_over && self.state == ButtonState::Hover {
                            self.state = ButtonState::Normal;
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        EventResult::Ignored
    }
}

/// Container that renders children in a vertical column
///
/// This is a simple widget version of the Column layout.
pub struct VStack {
    children: Vec<Box<dyn Component>>,
    gap: u16,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            gap: 0,
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn push(mut self, child: impl Component + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl Component for VStack {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        let children: Vec<ViewNode> = self.children.iter().map(|c| c.render(ctx)).collect();

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Pass event to all children until one handles it
        for child in &mut self.children {
            match child.handle_event(event, ctx) {
                EventResult::Consumed => return EventResult::Consumed,
                EventResult::Handled => return EventResult::Handled,
                EventResult::Ignored => continue,
            }
        }
        EventResult::Ignored
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Container that renders children in a horizontal row
pub struct HStack {
    children: Vec<Box<dyn Component>>,
    gap: u16,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            gap: 0,
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn push(mut self, child: impl Component + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl Component for HStack {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // For now, render children side-by-side in a simple way
        // TODO: proper horizontal layout with the Layout system
        let children: Vec<ViewNode> = self.children.iter().map(|c| c.render(ctx)).collect();

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        for child in &mut self.children {
            match child.handle_event(event, ctx) {
                EventResult::Consumed => return EventResult::Consumed,
                EventResult::Handled => return EventResult::Handled,
                EventResult::Ignored => continue,
            }
        }
        EventResult::Ignored
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Panel widget - a bordered container with optional title
///
/// Draws a box border around its content with an optional title in the border.
pub struct Panel {
    title: Option<String>,
    child: Option<Box<dyn Component>>,
    border_style: Style,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            title: None,
            child: None,
            border_style: Style::default().fg(Color::GRAY),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn child(mut self, child: impl Component + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }
}

impl Component for Panel {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Render border and title
        // For now, simple approach - just add title text and child
        let mut children = Vec::new();

        // Title line with borders
        if let Some(ref title) = self.title {
            let border_line = format!("┌─ {} ─┐", title);
            children.push(ViewNode::text_styled(border_line, self.border_style));
        } else {
            children.push(ViewNode::text_styled("┌─────┐", self.border_style));
        }

        // Child content
        if let Some(ref child) = self.child {
            children.push(child.render(ctx));
        }

        // Bottom border
        children.push(ViewNode::text_styled("└─────┘", self.border_style));

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        if let Some(ref mut child) = self.child {
            child.handle_event(event, ctx)
        } else {
            EventResult::Ignored
        }
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Rect;
    use crate::render::Buffer;
    use crate::state::{Signal, Store};

    #[test]
    fn test_static_text() {
        let text = Text::new("Hello");
        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);

        let ctx = RenderContext::new(&mut buffer, area, &store);
        let node = text.render(&ctx);

        match node {
            ViewNode::Text { content, .. } => {
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_reactive_text() {
        let signal = Signal::new(42);
        let text = Text::bind({
            let s = signal.clone();
            move || format!("Value: {}", s.get())
        });

        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);

        let ctx = RenderContext::new(&mut buffer, area, &store);
        let node = text.render(&ctx);

        match node {
            ViewNode::Text { content, .. } => {
                assert_eq!(content, "Value: 42");
            }
            _ => panic!("Expected text node"),
        }

        // Update signal
        signal.set(99);
        let node = text.render(&ctx);

        match node {
            ViewNode::Text { content, .. } => {
                assert_eq!(content, "Value: 99");
            }
            _ => panic!("Expected text node"),
        }
    }

    #[test]
    fn test_button_creation() {
        let clicked = Signal::new(false);
        let btn = Button::new("Test", {
            let c = clicked.clone();
            move || c.set(true)
        });

        assert_eq!(btn.label, "Test");
        assert_eq!(btn.state, ButtonState::Normal);
    }

    #[test]
    fn test_vstack_with_children() {
        let stack = VStack::new()
            .push(Text::new("Line 1"))
            .push(Text::new("Line 2"))
            .push(Text::new("Line 3"));

        assert_eq!(stack.children.len(), 3);
    }

    #[test]
    fn test_hstack_creation() {
        let stack = HStack::new()
            .gap(2)
            .push(Text::new("Left"))
            .push(Text::new("Right"));

        assert_eq!(stack.children.len(), 2);
        assert_eq!(stack.gap, 2);
    }

    #[test]
    fn test_panel_with_title() {
        let panel = Panel::new().title("Test Panel").child(Text::new("Content"));

        assert!(panel.title.is_some());
        assert_eq!(panel.title.unwrap(), "Test Panel");
    }

    #[test]
    fn test_panel_render() {
        use crate::layout::Rect;
        use crate::render::Buffer;
        use crate::state::Store;

        let panel = Panel::new().title("Info").child(Text::new("Test content"));

        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);
        let ctx = RenderContext::new(&mut buffer, area, &store);

        let node = panel.render(&ctx);

        // Should produce a container with border and content
        match node {
            ViewNode::Container { children, .. } => {
                assert!(children.len() >= 2); // At least border and content
            }
            _ => panic!("Expected container node"),
        }
    }
}
