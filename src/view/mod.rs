//! # View and Component System
//!
//! This module provides the component model for building reactive UIs.
//!
//! Key concepts:
//! - `Component`: Stateful UI element with lifecycle (mount/unmount/update)
//! - `View`: Trait for things that can be converted to renderable nodes
//! - `ViewNode`: Tree structure representing the UI hierarchy
//!
//! ## Example
//! ```no_run
//! use rsdrav::prelude::*;
//!
//! struct Counter {
//!     count: Signal<i32>,
//! }
//!
//! impl Component for Counter {
//!     fn render(&self, ctx: &RenderContext) -> ViewNode {
//!         // Return view tree
//!         ViewNode::text(format!("Count: {}", self.count.get()))
//!     }
//! }
//! ```

use crate::error::Result;
use crate::event::{Event, EventResult};
use crate::layout::Rect;
use crate::render::{Buffer, Cell};
use crate::state::Store;
use crate::theme::Style;

pub mod component;
pub mod widgets;

pub use component::*;
pub use widgets::*;

/// Context provided during rendering
pub struct RenderContext<'a> {
    pub buffer: &'a mut Buffer,
    pub area: Rect,
    pub style: Style,
    pub store: &'a Store,
}

impl<'a> RenderContext<'a> {
    pub fn new(buffer: &'a mut Buffer, area: Rect, store: &'a Store) -> Self {
        Self {
            buffer,
            area,
            style: Style::default(),
            store,
        }
    }

    /// Helper to write a string at position with current style
    pub fn write_str(&mut self, x: u16, y: u16, s: &str) {
        let style = self.style;
        for (i, ch) in s.chars().enumerate() {
            let cell = Cell::with_style(ch, style);
            self.buffer.set(x + i as u16, y, cell);
        }
    }
}

/// Context for component mounting
pub struct MountContext<'a> {
    pub store: &'a mut Store,
}

/// Context for component updates
pub struct UpdateContext<'a> {
    pub store: &'a Store,
}

/// Context for event handling
pub struct EventContext<'a> {
    pub store: &'a mut Store,
    /// The area where the component was last rendered (for hit-testing)
    pub area: Rect,
}

/// View node - the basic building block of the UI tree
///
/// This represents a renderable element. Components produce ViewNodes
/// which get laid out and rendered to the buffer.
#[derive(Debug, Clone)]
pub enum ViewNode {
    /// Text content
    Text { content: String, style: Style },

    /// Container with children and layout
    Container {
        children: Vec<ViewNode>,
        area: Rect,
        style: Style,
        /// Layout direction for children (defaults to vertical)
        direction: ContainerDirection,
    },

    /// Empty/spacer node
    Empty,
}

/// Direction for container layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerDirection {
    Vertical,
    Horizontal,
    Stacked, // All children get same area
}

impl ViewNode {
    /// Create a text node
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            style: Style::default(),
        }
    }

    /// Create a text node with style
    pub fn text_styled(content: impl Into<String>, style: Style) -> Self {
        Self::Text {
            content: content.into(),
            style,
        }
    }

    /// Create a container node
    pub fn container(children: Vec<ViewNode>) -> Self {
        Self::Container {
            children,
            area: Rect::new(0, 0, 0, 0),
            style: Style::default(),
            direction: ContainerDirection::Vertical,
        }
    }

    /// Create a container with specific direction
    pub fn container_with_direction(
        children: Vec<ViewNode>,
        direction: ContainerDirection,
    ) -> Self {
        Self::Container {
            children,
            area: Rect::new(0, 0, 0, 0),
            style: Style::default(),
            direction,
        }
    }

    /// Create an empty node
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Render this view node to the buffer
    ///
    /// This is called during the render phase after layout has been computed.
    pub fn render(&self, ctx: &mut RenderContext) {
        match self {
            ViewNode::Text { content, style } => {
                // Render text at the top-left of the area
                let x = ctx.area.x;
                let y = ctx.area.y;

                // Save old style, apply new one
                let old_style = ctx.style;
                ctx.style = *style;

                ctx.write_str(x, y, content);

                // Restore style
                ctx.style = old_style;
            }

            ViewNode::Container {
                children,
                direction,
                ..
            } => {
                use crate::layout::{Column, Length, Row};

                if children.is_empty() {
                    return;
                }

                // Calculate layout based on direction
                let child_rects = match direction {
                    ContainerDirection::Vertical => {
                        let col = Column::new();
                        // Distribute space equally among children
                        let heights = vec![Length::Fill(1); children.len()];
                        col.layout(ctx.area, &heights)
                    }
                    ContainerDirection::Horizontal => {
                        let row = Row::new();
                        // Distribute space equally among children
                        let widths = vec![Length::Fill(1); children.len()];
                        row.layout(ctx.area, &widths)
                    }
                    ContainerDirection::Stacked => {
                        // All children get the full area
                        vec![ctx.area; children.len()]
                    }
                };

                // Render each child in its calculated rect
                for (child, &child_area) in children.iter().zip(child_rects.iter()) {
                    let mut child_ctx = RenderContext {
                        buffer: ctx.buffer,
                        area: child_area,
                        style: ctx.style,
                        store: ctx.store,
                    };

                    child.render(&mut child_ctx);
                }
            }

            ViewNode::Empty => {
                // Nothing to render
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Buffer;
    use crate::state::Store;

    #[test]
    fn test_text_node_creation() {
        let node = ViewNode::text("Hello");
        match node {
            ViewNode::Text { content, .. } => {
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_container_with_children() {
        let children = vec![ViewNode::text("Line 1"), ViewNode::text("Line 2")];
        let container = ViewNode::container(children);

        match container {
            ViewNode::Container {
                children,
                direction,
                ..
            } => {
                assert_eq!(children.len(), 2);
                assert_eq!(direction, ContainerDirection::Vertical);
            }
            _ => panic!("Expected Container node"),
        }
    }

    #[test]
    fn test_render_text_node() {
        let mut buffer = Buffer::new(40, 10);
        let store = Store::new();
        let area = Rect::new(0, 0, 40, 10);

        let mut ctx = RenderContext::new(&mut buffer, area, &store);

        let node = ViewNode::text("Test");
        node.render(&mut ctx);

        // Check that text was written
        assert_eq!(buffer.get(0, 0).unwrap().ch, 'T');
        assert_eq!(buffer.get(1, 0).unwrap().ch, 'e');
        assert_eq!(buffer.get(2, 0).unwrap().ch, 's');
        assert_eq!(buffer.get(3, 0).unwrap().ch, 't');
    }
}
