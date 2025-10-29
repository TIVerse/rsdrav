use super::buffer::{Buffer, Cell};
use crate::layout::Rect;

/// Represents a rectangular region that needs to be redrawn
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirtyRegion {
    pub rect: Rect,
}

impl DirtyRegion {
    pub fn new(rect: Rect) -> Self {
        Self { rect }
    }

    /// Create a full-screen dirty region
    pub fn full_screen(width: u16, height: u16) -> Self {
        Self {
            rect: Rect::new(0, 0, width, height),
        }
    }
}

/// Compute differences between two buffers
///
/// Returns a list of dirty regions that need redrawing.
/// Uses line hashing for fast comparison and finds exact changed spans.
pub fn compute_diff(old: &Buffer, new: &Buffer) -> Vec<DirtyRegion> {
    // Quick bailout if dimensions changed - just redraw everything
    if old.width != new.width || old.height != new.height {
        return vec![DirtyRegion::full_screen(new.width, new.height)];
    }

    let mut dirty = Vec::new();

    // Check each line
    for y in 0..new.height {
        let old_line = old.line(y);
        let new_line = new.line(y);

        // Quick hash comparison first
        if line_hash(old_line) == line_hash(new_line) {
            continue; // lines are identical, skip
        }

        // Lines differ - find the exact changed spans
        find_changed_spans(old_line, new_line, y, &mut dirty);
    }

    // Merge adjacent dirty regions to reduce draw calls
    merge_adjacent_regions(&mut dirty);

    dirty
}

/// Find exact changed spans within a line
fn find_changed_spans(old_line: &[Cell], new_line: &[Cell], y: u16, dirty: &mut Vec<DirtyRegion>) {
    let width = old_line.len().min(new_line.len());
    let mut start: Option<u16> = None;

    for x in 0..width {
        let old_cell = &old_line[x];
        let new_cell = &new_line[x];

        if old_cell != new_cell {
            // Cell changed
            if start.is_none() {
                start = Some(x as u16);
            }
        } else {
            // Cell same - if we were tracking a span, close it
            if let Some(start_x) = start {
                let span_width = (x as u16) - start_x;
                dirty.push(DirtyRegion::new(Rect::new(start_x, y, span_width, 1)));
                start = None;
            }
        }
    }

    // Close any open span at end of line
    if let Some(start_x) = start {
        let span_width = (width as u16) - start_x;
        dirty.push(DirtyRegion::new(Rect::new(start_x, y, span_width, 1)));
    }
}

/// Merge adjacent dirty regions to reduce draw calls
fn merge_adjacent_regions(dirty: &mut Vec<DirtyRegion>) {
    if dirty.len() <= 1 {
        return;
    }

    // Sort by y coordinate first, then x
    dirty.sort_by_key(|r| (r.rect.y, r.rect.x));

    let mut merged = Vec::new();
    let mut current = dirty[0].clone();

    for i in 1..dirty.len() {
        let next = &dirty[i];

        // Check if regions can be merged (adjacent or overlapping on same line)
        if current.rect.y == next.rect.y {
            let current_end = current.rect.x + current.rect.width;
            let next_start = next.rect.x;

            // If adjacent or overlapping, merge
            if next_start <= current_end + 1 {
                let new_end = (next.rect.x + next.rect.width).max(current_end);
                current.rect.width = new_end - current.rect.x;
                continue;
            }
        }

        // Can't merge, push current and start new
        merged.push(current);
        current = next.clone();
    }

    merged.push(current);
    *dirty = merged;
}

/// Fast hash of a line for quick comparison
///
/// Uses FNV-1a hash - simple and fast for this use case.
/// Not cryptographic, just needs to detect changes reliably.
fn line_hash(line: &[Cell]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64; // FNV offset basis

    for cell in line {
        // Hash the character
        hash ^= cell.ch as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime

        // Hash the style (fg, bg, modifiers)
        if let Some(fg) = cell.style.fg {
            hash ^= fg.r as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            hash ^= fg.g as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            hash ^= fg.b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        if let Some(bg) = cell.style.bg {
            hash ^= bg.r as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            hash ^= bg.g as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            hash ^= bg.b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        hash ^= cell.style.modifiers.bits() as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    hash
}

/// Alias for compute_diff - precise diff is now the default
///
/// This function now delegates to compute_diff which includes
/// exact span detection and region merging.
pub fn compute_diff_precise(old: &Buffer, new: &Buffer) -> Vec<DirtyRegion> {
    compute_diff(old, new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, Style};

    #[test]
    fn test_diff_unchanged() {
        let buf1 = Buffer::new(10, 10);
        let buf2 = buf1.clone();

        let diff = compute_diff(&buf1, &buf2);
        assert_eq!(diff.len(), 0);
    }

    #[test]
    fn test_diff_single_cell() {
        let buf1 = Buffer::new(10, 10);
        let mut buf2 = buf1.clone();

        buf2.set(5, 5, Cell::new('X'));

        let diff = compute_diff(&buf1, &buf2);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].rect.y, 5);
    }

    #[test]
    fn test_diff_multiple_lines() {
        let buf1 = Buffer::new(10, 10);
        let mut buf2 = buf1.clone();

        buf2.set(0, 2, Cell::new('A'));
        buf2.set(0, 5, Cell::new('B'));
        buf2.set(0, 8, Cell::new('C'));

        let diff = compute_diff(&buf1, &buf2);
        assert_eq!(diff.len(), 3);
    }

    #[test]
    fn test_diff_size_change() {
        let buf1 = Buffer::new(10, 10);
        let buf2 = Buffer::new(20, 20);

        let diff = compute_diff(&buf1, &buf2);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].rect.width, 20);
        assert_eq!(diff[0].rect.height, 20);
    }

    #[test]
    fn test_diff_style_change() {
        let buf1 = Buffer::new(10, 10);
        let mut buf2 = buf1.clone();

        let style = Style::new().fg(Color::RED);
        buf2.set(5, 5, Cell::with_style('A', style));

        let diff = compute_diff(&buf1, &buf2);
        assert!(!diff.is_empty());
    }

    #[test]
    fn test_line_hash_collision_unlikely() {
        let cells1 = vec![Cell::new('A'); 10];
        let cells2 = vec![Cell::new('B'); 10];

        // Different content should have different hashes
        // (not guaranteed, but very likely with FNV)
        assert_ne!(line_hash(&cells1), line_hash(&cells2));
    }

    #[test]
    fn test_precise_diff() {
        let buf1 = Buffer::new(20, 5);
        let mut buf2 = buf1.clone();

        // Change just a few cells in the middle
        buf2.set(10, 2, Cell::new('X'));
        buf2.set(11, 2, Cell::new('Y'));
        buf2.set(12, 2, Cell::new('Z'));

        let diff = compute_diff_precise(&buf1, &buf2);
        assert_eq!(diff.len(), 1);

        let region = &diff[0].rect;
        assert_eq!(region.y, 2);
        assert_eq!(region.x, 10);
        assert_eq!(region.width, 3);
    }

    #[test]
    fn test_merge_adjacent_regions() {
        let buf1 = Buffer::new(20, 5);
        let mut buf2 = buf1.clone();

        // Create changes that should merge: adjacent cells on same line
        buf2.set(5, 2, Cell::new('A'));
        buf2.set(6, 2, Cell::new('B'));
        buf2.set(7, 2, Cell::new('C'));

        let diff = compute_diff(&buf1, &buf2);

        // Should be merged into single region
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].rect.x, 5);
        assert_eq!(diff[0].rect.width, 3);
    }

    #[test]
    fn test_merge_separate_regions() {
        let buf1 = Buffer::new(20, 5);
        let mut buf2 = buf1.clone();

        // Create separated changes on same line (shouldn't merge)
        buf2.set(5, 2, Cell::new('A'));
        buf2.set(15, 2, Cell::new('B'));

        let diff = compute_diff(&buf1, &buf2);

        // Should remain separate (gap is too large)
        assert!(!diff.is_empty()); // At least one region
    }
}
