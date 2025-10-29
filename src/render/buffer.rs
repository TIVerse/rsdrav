use crate::theme::Style;

/// Single terminal cell with character and styling
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            style: Style::default(),
        }
    }

    pub fn with_style(ch: char, style: Style) -> Self {
        Self { ch, style }
    }
}

/// Virtual terminal buffer - represents a 2D grid of cells
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    cells: Vec<Cell>, // flat array: cells[y * width + x]
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![Cell::default(); size],
        }
    }

    /// Get cell at position (returns None if out of bounds)
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = self.index(x, y);
        self.cells.get(idx)
    }

    /// Get mutable cell reference
    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = self.index(x, y);
        self.cells.get_mut(idx)
    }

    /// Set cell (silently ignores out-of-bounds)
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.cells[idx] = cell;
        }
    }

    /// Get entire line as slice
    pub fn line(&self, y: u16) -> &[Cell] {
        if y >= self.height {
            return &[];
        }
        let start = (y as usize) * (self.width as usize);
        let end = start + (self.width as usize);
        &self.cells[start..end]
    }

    /// Clear buffer to blank cells
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Resize buffer (clears content)
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = (width as usize) * (height as usize);
        self.cells = vec![Cell::default(); size];
    }

    #[inline]
    fn index(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }
}

// Implement Clone for Buffer
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            cells: self.cells.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buf = Buffer::new(80, 24);
        assert_eq!(buf.width, 80);
        assert_eq!(buf.height, 24);
        assert_eq!(buf.cells.len(), 80 * 24);
    }

    #[test]
    fn test_get_set() {
        let mut buf = Buffer::new(10, 10);
        let cell = Cell::new('A');
        buf.set(5, 5, cell.clone());
        assert_eq!(buf.get(5, 5), Some(&cell));
    }

    #[test]
    fn test_out_of_bounds() {
        let buf = Buffer::new(10, 10);
        assert_eq!(buf.get(10, 5), None);
        assert_eq!(buf.get(5, 10), None);
    }

    #[test]
    fn test_line_access() {
        let mut buf = Buffer::new(5, 3);
        buf.set(0, 1, Cell::new('H'));
        buf.set(1, 1, Cell::new('i'));

        let line = buf.line(1);
        assert_eq!(line.len(), 5);
        assert_eq!(line[0].ch, 'H');
        assert_eq!(line[1].ch, 'i');
    }

    #[test]
    fn test_clear() {
        let mut buf = Buffer::new(5, 5);
        buf.set(2, 2, Cell::new('X'));
        buf.clear();
        assert_eq!(buf.get(2, 2).unwrap().ch, '\0');
    }

    #[test]
    fn test_resize() {
        let mut buf = Buffer::new(10, 10);
        buf.set(5, 5, Cell::new('A'));

        buf.resize(20, 20);
        assert_eq!(buf.width, 20);
        assert_eq!(buf.height, 20);
        // Content cleared after resize
        assert_eq!(buf.get(5, 5).unwrap().ch, '\0');
    }
}
