use super::diff::{compute_diff, DirtyRegion};
use super::{Backend, Buffer};
use crate::error::Result;
use crate::theme::Modifier;
use std::io::Write;

/// Renderer that efficiently writes buffer changes to a backend
///
/// Uses diff algorithm to minimize terminal writes. Only updates
/// regions that actually changed between frames.
pub struct Renderer {
    // Track if we've done first render (forces full redraw)
    first_render: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self { first_render: true }
    }

    /// Render buffer to backend using diff from previous buffer
    ///
    /// If prev_buffer is None, does a full redraw.
    pub fn render(
        &mut self,
        backend: &mut dyn Backend,
        prev_buffer: Option<&Buffer>,
        buffer: &Buffer,
    ) -> Result<()> {
        let dirty_regions = if self.first_render || prev_buffer.is_none() {
            // First render or no previous buffer - redraw everything
            self.first_render = false;
            vec![DirtyRegion::full_screen(buffer.width, buffer.height)]
        } else {
            compute_diff(prev_buffer.unwrap(), buffer)
        };

        // Nothing to update?
        if dirty_regions.is_empty() {
            return Ok(());
        }

        // Render each dirty region
        for region in dirty_regions {
            self.render_region(backend, buffer, &region)?;
        }

        backend.flush()?;
        Ok(())
    }

    /// Render a specific region of the buffer
    fn render_region(
        &self,
        backend: &mut dyn Backend,
        buffer: &Buffer,
        region: &DirtyRegion,
    ) -> Result<()> {
        let rect = region.rect;

        // Render each line in the region
        for y in rect.y..(rect.y + rect.height).min(buffer.height) {
            backend.cursor_goto(rect.x, y)?;

            // Build output for this line
            let mut output = Vec::new();
            let mut current_style = None;

            for x in rect.x..(rect.x + rect.width).min(buffer.width) {
                if let Some(cell) = buffer.get(x, y) {
                    // Apply style if it changed
                    if current_style.as_ref() != Some(&cell.style) {
                        write_style_codes(&mut output, &cell.style)?;
                        current_style = Some(cell.style);
                    }

                    // Write the character
                    write!(output, "{}", cell.ch)?;
                }
            }

            // Reset style at end of line
            if current_style.is_some() {
                write_reset_codes(&mut output)?;
            }

            backend.write(&output)?;
        }

        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Write ANSI escape codes for style
fn write_style_codes(output: &mut Vec<u8>, style: &crate::theme::Style) -> Result<()> {
    // Reset first to clear previous style
    write!(output, "\x1b[0m")?;

    // Foreground color
    if let Some(fg) = style.fg {
        write!(output, "\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b)?;
    }

    // Background color
    if let Some(bg) = style.bg {
        write!(output, "\x1b[48;2;{};{};{}m", bg.r, bg.g, bg.b)?;
    }

    // Modifiers
    if style.modifiers.contains(Modifier::BOLD) {
        write!(output, "\x1b[1m")?;
    }
    if style.modifiers.contains(Modifier::DIM) {
        write!(output, "\x1b[2m")?;
    }
    if style.modifiers.contains(Modifier::ITALIC) {
        write!(output, "\x1b[3m")?;
    }
    if style.modifiers.contains(Modifier::UNDERLINE) {
        write!(output, "\x1b[4m")?;
    }
    if style.modifiers.contains(Modifier::BLINK) {
        write!(output, "\x1b[5m")?;
    }
    if style.modifiers.contains(Modifier::REVERSE) {
        write!(output, "\x1b[7m")?;
    }
    if style.modifiers.contains(Modifier::HIDDEN) {
        write!(output, "\x1b[8m")?;
    }
    if style.modifiers.contains(Modifier::STRIKETHROUGH) {
        write!(output, "\x1b[9m")?;
    }

    Ok(())
}

/// Write ANSI reset codes
fn write_reset_codes(output: &mut Vec<u8>) -> Result<()> {
    write!(output, "\x1b[0m")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Color, Style};

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();
        assert!(renderer.first_render);
    }

    #[test]
    fn test_style_codes() {
        let mut output = Vec::new();
        let style = Style::new().fg(Color::RED).bg(Color::BLUE);

        write_style_codes(&mut output, &style).unwrap();

        // Should contain ANSI escape sequences
        let s = String::from_utf8_lossy(&output);
        assert!(s.contains("\x1b["));
    }
}
