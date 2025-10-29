use super::{Align, Justify, Length, Rect};

/// Row container - lays out children horizontally
///
/// Children are arranged left-to-right with optional gap between them.
/// Supports flexible sizing with Fill weights.
#[derive(Clone, Debug)]
pub struct Row {
    pub gap: u16,
    pub align: Align,
    pub justify: Justify,
}

impl Row {
    pub fn new() -> Self {
        Self {
            gap: 0,
            align: Align::Stretch,
            justify: Justify::Start,
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Layout children horizontally within the given area
    ///
    /// Returns a vec of Rects, one for each child based on their widths.
    /// Respects Length specifications (Fixed, Percent, Fill) and distributes
    /// space according to fill weights.
    pub fn layout(&self, area: Rect, child_widths: &[Length]) -> Vec<Rect> {
        if child_widths.is_empty() {
            return Vec::new();
        }

        let n = child_widths.len();
        let total_gap = self.gap.saturating_mul((n.saturating_sub(1)) as u16);
        let available = area.width.saturating_sub(total_gap);

        // Phase 1: calculate fixed/percent sizes and count fill weights
        let mut sizes = vec![0u16; n];
        let mut remaining = available;
        let mut total_fill_weight = 0u16;

        for (i, width) in child_widths.iter().enumerate() {
            match width {
                Length::Fill(weight) => {
                    total_fill_weight = total_fill_weight.saturating_add(*weight);
                }
                _ => {
                    let size = width.resolve(available);
                    sizes[i] = size;
                    remaining = remaining.saturating_sub(size);
                }
            }
        }

        // Phase 2: distribute remaining space to Fill items by weight
        if total_fill_weight > 0 {
            for (i, width) in child_widths.iter().enumerate() {
                if let Length::Fill(weight) = width {
                    let size =
                        ((remaining as f32) * (*weight as f32) / (total_fill_weight as f32)) as u16;
                    sizes[i] = size;
                }
            }
        }

        // Phase 3: create rects based on justification
        let mut rects = Vec::with_capacity(n);
        let total_size: u16 = sizes.iter().sum();
        let total_with_gaps = total_size.saturating_add(total_gap);

        let mut x = match self.justify {
            Justify::Start => area.x,
            Justify::End => area
                .x
                .saturating_add(area.width.saturating_sub(total_with_gaps)),
            Justify::Center => area
                .x
                .saturating_add((area.width.saturating_sub(total_with_gaps)) / 2),
            Justify::SpaceBetween | Justify::SpaceAround | Justify::SpaceEvenly => area.x,
        };

        for &width in sizes.iter() {
            let y = match self.align {
                Align::Start => area.y,
                Align::End => area
                    .y
                    .saturating_add(area.height.saturating_sub(area.height)),
                Align::Center => area
                    .y
                    .saturating_add(area.height / 2)
                    .saturating_sub(area.height / 2),
                Align::Stretch => area.y,
            };

            // Height is determined by alignment:
            // - Stretch: fill full cross-axis
            // - Other alignments: use full height (child can choose to render smaller)
            let height = area.height;

            rects.push(Rect::new(x, y, width, height));
            x = x.saturating_add(width).saturating_add(self.gap);
        }

        rects
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

/// Column container - lays out children vertically
///
/// Similar to Row but arranges top-to-bottom.
#[derive(Clone, Debug)]
pub struct Column {
    pub gap: u16,
    pub align: Align,
    pub justify: Justify,
}

impl Column {
    pub fn new() -> Self {
        Self {
            gap: 0,
            align: Align::Stretch,
            justify: Justify::Start,
        }
    }

    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    /// Layout children vertically within the given area
    pub fn layout(&self, area: Rect, child_heights: &[Length]) -> Vec<Rect> {
        if child_heights.is_empty() {
            return Vec::new();
        }

        let n = child_heights.len();
        let total_gap = self.gap.saturating_mul((n.saturating_sub(1)) as u16);
        let available = area.height.saturating_sub(total_gap);

        // Phase 1: calculate sizes
        let mut sizes = vec![0u16; n];
        let mut remaining = available;
        let mut total_fill_weight = 0u16;

        for (i, height) in child_heights.iter().enumerate() {
            match height {
                Length::Fill(weight) => {
                    total_fill_weight = total_fill_weight.saturating_add(*weight);
                }
                _ => {
                    let size = height.resolve(available);
                    sizes[i] = size;
                    remaining = remaining.saturating_sub(size);
                }
            }
        }

        // Phase 2: distribute Fill items
        if total_fill_weight > 0 {
            for (i, height) in child_heights.iter().enumerate() {
                if let Length::Fill(weight) = height {
                    let size =
                        ((remaining as f32) * (*weight as f32) / (total_fill_weight as f32)) as u16;
                    sizes[i] = size;
                }
            }
        }

        // Phase 3: create rects
        let mut rects = Vec::with_capacity(n);
        let total_size: u16 = sizes.iter().sum();
        let total_with_gaps = total_size.saturating_add(total_gap);

        let mut y = match self.justify {
            Justify::Start => area.y,
            Justify::End => area
                .y
                .saturating_add(area.height.saturating_sub(total_with_gaps)),
            Justify::Center => area
                .y
                .saturating_add((area.height.saturating_sub(total_with_gaps)) / 2),
            Justify::SpaceBetween | Justify::SpaceAround | Justify::SpaceEvenly => area.y,
        };

        for &height in &sizes {
            let x = match self.align {
                Align::Start => area.x,
                Align::End => area.x.saturating_add(area.width.saturating_sub(area.width)),
                Align::Center => area
                    .x
                    .saturating_add(area.width / 2)
                    .saturating_sub(area.width / 2),
                Align::Stretch => area.x,
            };

            // Width is determined by alignment:
            // - Stretch: fill full cross-axis
            // - Other alignments: use full width (child can choose to render smaller)
            let width = area.width;

            rects.push(Rect::new(x, y, width, height));
            y = y.saturating_add(height).saturating_add(self.gap);
        }

        rects
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

/// Stack container - overlays children on top of each other
///
/// All children get the same area. Useful for modals, overlays, etc.
#[derive(Clone, Debug, Default)]
pub struct Stack {
    pub align: Align,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            align: Align::Stretch,
        }
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Layout children - all get the full area
    pub fn layout(&self, area: Rect, count: usize) -> Vec<Rect> {
        vec![area; count]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_equal_distribution() {
        let row = Row::new();
        let area = Rect::new(0, 0, 100, 20);
        let widths = vec![Length::Fill(1), Length::Fill(1), Length::Fill(1)];

        let rects = row.layout(area, &widths);
        assert_eq!(rects.len(), 3);

        // Each should get roughly 1/3 of width
        assert!(rects[0].width >= 30 && rects[0].width <= 35);
        assert!(rects[1].width >= 30 && rects[1].width <= 35);
        assert!(rects[2].width >= 30 && rects[2].width <= 35);
    }

    #[test]
    fn test_row_with_gap() {
        let row = Row::new().gap(5);
        let area = Rect::new(0, 0, 100, 20);
        let widths = vec![Length::Fill(1), Length::Fill(1)];

        let rects = row.layout(area, &widths);
        assert_eq!(rects.len(), 2);

        // Gap should be 5 between them
        assert_eq!(rects[1].x, rects[0].x + rects[0].width + 5);
    }

    #[test]
    fn test_row_fixed_sizes() {
        let row = Row::new();
        let area = Rect::new(0, 0, 100, 20);
        let widths = vec![Length::Fixed(30), Length::Fixed(40)];

        let rects = row.layout(area, &widths);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 30);
        assert_eq!(rects[1].width, 40);
    }

    #[test]
    fn test_column_basic() {
        let col = Column::new();
        let area = Rect::new(0, 0, 100, 100);
        let heights = vec![Length::Fill(1), Length::Fill(2)];

        let rects = col.layout(area, &heights);
        assert_eq!(rects.len(), 2);

        // Second should be roughly twice the first
        assert!(rects[1].height >= rects[0].height * 2 - 2);
    }

    #[test]
    fn test_stack() {
        let stack = Stack::new();
        let area = Rect::new(10, 10, 50, 50);

        let rects = stack.layout(area, 3);
        assert_eq!(rects.len(), 3);

        // All should be the same
        for rect in rects {
            assert_eq!(rect, area);
        }
    }
}
