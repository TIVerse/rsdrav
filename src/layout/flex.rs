//! Flexible box layout (Flexbox-inspired)
//!
//! Provides CSS Flexbox-like layout for terminal UIs.

use super::{FlexDirection, Length, Rect};

/// Flex container for flexible layout
#[derive(Debug, Clone)]
pub struct Flex {
    direction: FlexDirection,
    items: Vec<FlexItem>,
}

/// Individual flex item with sizing constraints
#[derive(Debug, Clone)]
pub struct FlexItem {
    /// Flex grow factor (how much it grows relative to others)
    pub grow: f32,
    /// Flex shrink factor (how much it shrinks relative to others)
    pub shrink: f32,
    /// Base size (defaults to Fill(0))
    pub basis: Length,
    /// Minimum size constraint
    pub min: Option<u16>,
    /// Maximum size constraint
    pub max: Option<u16>,
}

impl FlexItem {
    /// Create a new flex item with default settings
    pub fn new() -> Self {
        Self {
            grow: 0.0,
            shrink: 1.0,
            basis: Length::Fill(0), // Default to flexible size
            min: None,
            max: None,
        }
    }

    /// Set grow factor
    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }

    /// Set shrink factor
    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = shrink;
        self
    }

    /// Set basis size
    pub fn basis(mut self, basis: Length) -> Self {
        self.basis = basis;
        self
    }

    /// Convenience: Set basis to fixed size
    pub fn fixed(mut self, size: u16) -> Self {
        self.basis = Length::Fixed(size);
        self
    }

    /// Set minimum size
    pub fn min(mut self, min: u16) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum size
    pub fn max(mut self, max: u16) -> Self {
        self.max = Some(max);
        self
    }
}

impl Default for FlexItem {
    fn default() -> Self {
        Self::new()
    }
}

impl Flex {
    /// Create a new flex container
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            items: Vec::new(),
        }
    }

    /// Add a flex item
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: FlexItem) -> Self {
        self.items.push(item);
        self
    }

    /// Calculate layout for all flex items
    pub fn calculate(&self, container: Rect) -> Vec<Rect> {
        if self.items.is_empty() {
            return Vec::new();
        }

        let (main_size, cross_size) = match self.direction {
            FlexDirection::Row => (container.width, container.height),
            FlexDirection::Column => (container.height, container.width),
        };

        // Calculate base sizes
        let mut sizes = self.calculate_base_sizes(main_size);

        // Distribute remaining space or shrink
        let total_size: u16 = sizes.iter().sum();
        if total_size < main_size {
            self.grow_items(&mut sizes, main_size);
        } else if total_size > main_size {
            self.shrink_items(&mut sizes, main_size);
        }

        // Convert sizes to rectangles
        self.sizes_to_rects(sizes, container, cross_size)
    }

    /// Calculate initial base sizes for all items
    fn calculate_base_sizes(&self, main_size: u16) -> Vec<u16> {
        self.items
            .iter()
            .map(|item| {
                let base = match item.basis {
                    Length::Fixed(n) => n,
                    Length::Percent(p) => ((main_size as f32) * p) as u16,
                    Length::Fill(_) => 0, // Will be calculated during grow
                    Length::Min(n) => n,
                    Length::Max(n) => n.min(main_size),
                };

                // Apply constraints
                let mut size = base;
                if let Some(min) = item.min {
                    size = size.max(min);
                }
                if let Some(max) = item.max {
                    size = size.min(max);
                }
                size
            })
            .collect()
    }

    /// Grow items to fill remaining space
    fn grow_items(&self, sizes: &mut [u16], main_size: u16) {
        let total: u16 = sizes.iter().sum();
        let remaining = main_size.saturating_sub(total);

        if remaining == 0 {
            return;
        }

        let total_grow: f32 = self.items.iter().map(|item| item.grow).sum();
        if total_grow <= 0.0 {
            return;
        }

        for (i, item) in self.items.iter().enumerate() {
            if item.grow > 0.0 {
                let grow_amount = ((remaining as f32) * item.grow / total_grow) as u16;
                let mut new_size = sizes[i] + grow_amount;

                // Respect max constraint
                if let Some(max) = item.max {
                    new_size = new_size.min(max);
                }

                sizes[i] = new_size;
            }
        }
    }

    /// Shrink items to fit available space
    fn shrink_items(&self, sizes: &mut [u16], main_size: u16) {
        let total: u16 = sizes.iter().sum();
        let overflow = total.saturating_sub(main_size);

        if overflow == 0 {
            return;
        }

        let total_shrink: f32 = self.items.iter().map(|item| item.shrink).sum();
        if total_shrink <= 0.0 {
            return;
        }

        for (i, item) in self.items.iter().enumerate() {
            if item.shrink > 0.0 && sizes[i] > 0 {
                let shrink_amount = ((overflow as f32) * item.shrink / total_shrink) as u16;
                let mut new_size = sizes[i].saturating_sub(shrink_amount);

                // Respect min constraint
                if let Some(min) = item.min {
                    new_size = new_size.max(min);
                }

                sizes[i] = new_size;
            }
        }
    }

    /// Convert calculated sizes to rectangles
    fn sizes_to_rects(&self, sizes: Vec<u16>, container: Rect, cross_size: u16) -> Vec<Rect> {
        let mut rects = Vec::new();
        let mut offset = 0;

        for size in sizes {
            let rect = match self.direction {
                FlexDirection::Row => Rect::new(
                    container.x + offset,
                    container.y,
                    size,
                    cross_size.min(container.height),
                ),
                FlexDirection::Column => Rect::new(
                    container.x,
                    container.y + offset,
                    cross_size.min(container.width),
                    size,
                ),
            };

            rects.push(rect);
            offset += size;
        }

        rects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_row_equal() {
        let flex = Flex::new(FlexDirection::Row)
            .add(FlexItem::new().grow(1.0))
            .add(FlexItem::new().grow(1.0))
            .add(FlexItem::new().grow(1.0));

        let container = Rect::new(0, 0, 90, 10);
        let rects = flex.calculate(container);

        assert_eq!(rects.len(), 3);
        assert_eq!(rects[0].width, 30);
        assert_eq!(rects[1].width, 30);
        assert_eq!(rects[2].width, 30);
    }

    #[test]
    fn test_flex_column_unequal() {
        let flex = Flex::new(FlexDirection::Column)
            .add(FlexItem::new().grow(1.0))
            .add(FlexItem::new().grow(2.0));

        let container = Rect::new(0, 0, 80, 30);
        let rects = flex.calculate(container);

        assert_eq!(rects.len(), 2);
        assert!(rects[1].height > rects[0].height);
    }

    #[test]
    fn test_flex_with_min_max() {
        let flex = Flex::new(FlexDirection::Row)
            .add(FlexItem::new().grow(1.0).min(20).max(40))
            .add(FlexItem::new().grow(1.0));

        let container = Rect::new(0, 0, 100, 10);
        let rects = flex.calculate(container);

        assert!(rects[0].width <= 40);
        assert!(rects[0].width >= 20);
    }

    #[test]
    fn test_flex_shrink() {
        let flex = Flex::new(FlexDirection::Row)
            .add(FlexItem::new().basis(Length::Fixed(60)))
            .add(FlexItem::new().basis(Length::Fixed(60)));

        let container = Rect::new(0, 0, 80, 10);
        let rects = flex.calculate(container);

        // Should shrink to fit
        let total_width: u16 = rects.iter().map(|r| r.width).sum();
        assert!(total_width <= 80);
    }
}
