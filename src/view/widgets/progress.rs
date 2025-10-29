//! Progress bar widget for showing task completion
//!
//! Displays progress as a filled bar with optional label and percentage.

use crate::event::{Event, EventResult};
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};

/// Progress bar widget
///
/// Shows completion progress as a horizontal bar with optional label.
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
///
/// let progress = Signal::new(0.75); // 75%
///
/// let bar = ProgressBar::new(progress)
///     .label("Loading...")
///     .width(40)
///     .show_percentage(true);
/// ```
pub struct ProgressBar {
    progress: Signal<f32>, // 0.0 to 1.0
    label: Option<String>,
    width: usize,
    show_percentage: bool,
    style: ProgressStyle,
}

#[derive(Clone)]
struct ProgressStyle {
    filled: Style,
    empty: Style,
    label: Style,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            filled: Style::default().bg(Color::GREEN).fg(Color::BLACK),
            empty: Style::default().bg(Color::rgb(40, 40, 40)).fg(Color::GRAY),
            label: Style::default().fg(Color::WHITE),
        }
    }
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(progress: Signal<f32>) -> Self {
        Self {
            progress,
            label: None,
            width: 30,
            show_percentage: true,
            style: ProgressStyle::default(),
        }
    }

    /// Set label text
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set bar width in characters
    pub fn width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Show/hide percentage text
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set filled bar color
    pub fn filled_color(mut self, color: Color) -> Self {
        self.style.filled = self.style.filled.bg(color);
        self
    }

    /// Render the progress bar
    fn render_bar(&self, progress: f32) -> String {
        let filled_width = ((self.width as f32) * progress.clamp(0.0, 1.0)) as usize;
        let empty_width = self.width.saturating_sub(filled_width);

        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(empty_width);

        format!("{}{}", filled, empty)
    }
}

impl Component for ProgressBar {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        let progress = self.progress.get().clamp(0.0, 1.0);
        let bar = self.render_bar(progress);

        let mut parts = Vec::new();

        // Add label if present
        if let Some(ref label) = self.label {
            parts.push(ViewNode::text_styled(
                format!("{} ", label),
                self.style.label,
            ));
        }

        // Add bar
        parts.push(ViewNode::text_styled(
            bar.clone(),
            if progress > 0.0 {
                self.style.filled
            } else {
                self.style.empty
            },
        ));

        // Add percentage if enabled
        if self.show_percentage {
            let pct = (progress * 100.0) as u32;
            parts.push(ViewNode::text_styled(
                format!(" {}%", pct),
                self.style.label,
            ));
        }

        // Combine horizontally
        let combined = parts
            .into_iter()
            .map(|node| match node {
                ViewNode::Text { content, .. } => content.to_string(),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("");

        ViewNode::text_styled(combined, self.style.label)
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation() {
        let progress = Signal::new(0.5);
        let bar = ProgressBar::new(progress);
        assert_eq!(bar.width, 30);
        assert!(bar.show_percentage);
    }

    #[test]
    fn test_progress_bar_render() {
        let progress = Signal::new(0.5);
        let bar = ProgressBar::new(progress).width(10);

        let rendered = bar.render_bar(0.5);
        assert_eq!(rendered.chars().count(), 10);
    }

    #[test]
    fn test_progress_clamping() {
        let progress = Signal::new(1.5); // Over 100%
        let bar = ProgressBar::new(progress).width(10);

        let rendered = bar.render_bar(1.5);
        assert_eq!(rendered.chars().filter(|&c| c == '█').count(), 10); // All filled
    }

    #[test]
    fn test_progress_bar_label() {
        let progress = Signal::new(0.75);
        let bar = ProgressBar::new(progress).label("Loading");

        assert_eq!(bar.label, Some("Loading".to_string()));
    }

    #[test]
    fn test_zero_progress() {
        let progress = Signal::new(0.0);
        let bar = ProgressBar::new(progress).width(10);

        let rendered = bar.render_bar(0.0);
        assert_eq!(rendered.chars().filter(|&c| c == '░').count(), 10); // All empty
    }

    #[test]
    fn test_full_progress() {
        let progress = Signal::new(1.0);
        let bar = ProgressBar::new(progress).width(10);

        let rendered = bar.render_bar(1.0);
        assert_eq!(rendered.chars().filter(|&c| c == '█').count(), 10); // All filled
    }
}
