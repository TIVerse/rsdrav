//! Layout system for positioning and sizing widgets
//!
//! Two-pass layout:
//! 1. Measure pass: widgets report their preferred/min/max sizes
//! 2. Layout pass: parent allocates space based on constraints
//!
//! Core types:
//! - `Rect`: rectangular area (x, y, width, height)
//! - `Length`: size specification (Fixed, Percent, Fill, Min, Max)
//! - `Align`/`Justify`: alignment modes

mod containers;
mod flex;

pub use containers::{Column, Row, Stack};
pub use flex::{Flex, FlexItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a rect at origin with given size
    pub const fn from_size(width: u16, height: u16) -> Self {
        Self::new(0, 0, width, height)
    }

    /// Area of the rect (width * height)
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Check if point is inside rect
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Shrink rect by margin on all sides
    pub fn inner(&self, margin: u16) -> Self {
        let m2 = margin.saturating_mul(2);
        Self {
            x: self.x.saturating_add(margin),
            y: self.y.saturating_add(margin),
            width: self.width.saturating_sub(m2),
            height: self.height.saturating_sub(m2),
        }
    }

    /// Shrink with individual margins (top, right, bottom, left)
    pub fn inner_margins(&self, top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            x: self.x.saturating_add(left),
            y: self.y.saturating_add(top),
            width: self.width.saturating_sub(left.saturating_add(right)),
            height: self.height.saturating_sub(top.saturating_add(bottom)),
        }
    }

    /// Split horizontally at position (returns left, right)
    pub fn split_h(&self, at: u16) -> (Self, Self) {
        let left = Self {
            x: self.x,
            y: self.y,
            width: at.min(self.width),
            height: self.height,
        };
        let right = Self {
            x: self.x.saturating_add(at),
            y: self.y,
            width: self.width.saturating_sub(at),
            height: self.height,
        };
        (left, right)
    }

    /// Split vertically at position (returns top, bottom)
    pub fn split_v(&self, at: u16) -> (Self, Self) {
        let top = Self {
            x: self.x,
            y: self.y,
            width: self.width,
            height: at.min(self.height),
        };
        let bottom = Self {
            x: self.x,
            y: self.y.saturating_add(at),
            width: self.width,
            height: self.height.saturating_sub(at),
        };
        (top, bottom)
    }

    /// Intersect with another rect
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x1 < x2 && y1 < y2 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// Union with another rect (smallest rect containing both)
    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);

        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }
}

/// Size specification for layout
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    /// Fixed absolute size
    Fixed(u16),

    /// Percentage of parent (0.0 to 1.0)
    Percent(f32),

    /// Fill remaining space with weight
    /// If multiple Fill items, space is distributed by weight
    Fill(u16),

    /// At least this size
    Min(u16),

    /// At most this size
    Max(u16),
}

impl Length {
    /// Resolve length to actual pixels given available space
    pub fn resolve(&self, available: u16) -> u16 {
        match self {
            Length::Fixed(n) => *n,
            Length::Percent(p) => ((available as f32) * p).round() as u16,
            Length::Fill(_) => available, // caller handles Fill specially
            Length::Min(n) => (*n).min(available),
            Length::Max(n) => (*n).min(available),
        }
    }
}

/// Alignment along cross axis
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Align {
    Start, // top or left
    Center,
    End,     // bottom or right
    Stretch, // fill entire cross axis
}

impl Default for Align {
    fn default() -> Self {
        Self::Stretch
    }
}

/// Justification along main axis
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Justify {
    Start,        // pack at start
    Center,       // center items
    End,          // pack at end
    SpaceBetween, // even spacing between items
    SpaceAround,  // even spacing around items
    SpaceEvenly,  // truly even spacing
}

/// Direction for flex layouts
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlexDirection {
    Row,
    Column,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 20, 20);

        assert!(rect.contains(10, 10));
        assert!(rect.contains(29, 29));
        assert!(!rect.contains(30, 30));
        assert!(!rect.contains(9, 10));
    }

    #[test]
    fn test_rect_inner() {
        let rect = Rect::new(0, 0, 20, 20);
        let inner = rect.inner(5);

        assert_eq!(inner.x, 5);
        assert_eq!(inner.y, 5);
        assert_eq!(inner.width, 10);
        assert_eq!(inner.height, 10);
    }

    #[test]
    fn test_rect_split() {
        let rect = Rect::new(0, 0, 20, 20);

        let (left, right) = rect.split_h(10);
        assert_eq!(left.width, 10);
        assert_eq!(right.width, 10);
        assert_eq!(right.x, 10);

        let (top, bottom) = rect.split_v(5);
        assert_eq!(top.height, 5);
        assert_eq!(bottom.height, 15);
        assert_eq!(bottom.y, 5);
    }

    #[test]
    fn test_rect_intersect() {
        let r1 = Rect::new(0, 0, 10, 10);
        let r2 = Rect::new(5, 5, 10, 10);

        let inter = r1.intersect(&r2).unwrap();
        assert_eq!(inter, Rect::new(5, 5, 5, 5));

        let r3 = Rect::new(20, 20, 10, 10);
        assert!(r1.intersect(&r3).is_none());
    }

    #[test]
    fn test_rect_union() {
        let r1 = Rect::new(0, 0, 10, 10);
        let r2 = Rect::new(5, 5, 10, 10);

        let union = r1.union(&r2);
        assert_eq!(union, Rect::new(0, 0, 15, 15));
    }

    #[test]
    fn test_length_resolve() {
        assert_eq!(Length::Fixed(100).resolve(200), 100);
        assert_eq!(Length::Percent(0.5).resolve(200), 100);
        assert_eq!(Length::Min(50).resolve(200), 50);
        assert_eq!(Length::Max(50).resolve(200), 50);
        assert_eq!(Length::Max(300).resolve(200), 200);
    }
}
