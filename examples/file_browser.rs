//! File Browser Example
//!
//! Demonstrates:
//! - List widget with file display
//! - Keyboard navigation (Up/Down/PageUp/PageDown)
//! - Selection highlighting
//! - Reactive state updates
//!
//! Controls:
//! - Up/Down - Navigate files
//! - Page Up/Down - Jump by page
//! - Home/End - Jump to first/last
//! - Enter - "Open" file (just shows selected)
//! - q - Quit

use rsdrav::prelude::*;
use std::fs;
use std::path::PathBuf;

fn main() -> rsdrav::Result<()> {
    App::new()?.root(FileBrowser::new(".")?).run()
}

/// File browser component
struct FileBrowser {
    files: Signal<Vec<FileEntry>>,
    selected: Signal<Option<usize>>,
    current_path: Signal<String>,
    status_message: Signal<String>,
}

#[derive(Clone, Debug)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

impl FileBrowser {
    fn new(path: impl Into<String>) -> rsdrav::Result<Self> {
        let path_str = path.into();
        let files = Self::read_directory(&path_str)?;

        Ok(Self {
            files: Signal::new(files),
            selected: Signal::new(Some(0)),
            current_path: Signal::new(path_str),
            status_message: Signal::new(String::from(
                "Use arrows to navigate, Enter to select, q to quit",
            )),
        })
    }

    fn read_directory(path: &str) -> rsdrav::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        let dir_entries = fs::read_dir(path).map_err(|e| rsdrav::Error::Io(e))?;

        for entry in dir_entries {
            let entry = entry.map_err(|e| rsdrav::Error::Io(e))?;
            let metadata = entry.metadata().map_err(|e| rsdrav::Error::Io(e))?;

            let name = entry.file_name().to_string_lossy().to_string();

            entries.push(FileEntry {
                name,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }

        // Sort: directories first, then files alphabetically
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(entries)
    }

    fn get_selected_file(&self) -> Option<FileEntry> {
        let files = self.files.get();
        let idx = self.selected.get()?;
        files.get(idx).cloned()
    }

    fn handle_select(&mut self) {
        if let Some(file) = self.get_selected_file() {
            let msg = if file.is_dir {
                format!("Directory: {} ({} items)", file.name, "?")
            } else {
                format!("File: {} ({} bytes)", file.name, file.size)
            };
            self.status_message.set(msg);
        }
    }
}

impl Component for FileBrowser {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Title
        let title = Text::new("=== File Browser ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        // Current path display
        let path_display = Text::bind({
            let p = self.current_path.clone();
            move || format!("📁 {}", p.get())
        })
        .fg(Color::CYAN);

        // File list
        let file_list = List::new(self.files.clone(), self.selected.clone())
            .visible_height(15)
            .render_item(|entry, is_selected| {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let size_str = if entry.is_dir {
                    String::from("<DIR>")
                } else {
                    format_size(entry.size)
                };

                let text = format!("{} {:<30} {:>10}", icon, entry.name, size_str);

                let style = if is_selected {
                    Style::default()
                        .bg(Color::BLUE)
                        .fg(Color::WHITE)
                        .add_modifier(Modifier::BOLD)
                } else if entry.is_dir {
                    Style::default().fg(Color::CYAN)
                } else {
                    Style::default().fg(Color::WHITE)
                };

                ViewNode::text_styled(text, style)
            });

        // Status bar
        let status = Text::bind({
            let s = self.status_message.clone();
            move || s.get()
        })
        .fg(Color::GRAY);

        // Instructions
        let instructions = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Controls:").fg(Color::YELLOW))
            .push(Text::new("  ↑/↓     - Navigate").fg(Color::GRAY))
            .push(Text::new("  PgUp/Dn - Page up/down").fg(Color::GRAY))
            .push(Text::new("  Home/End - First/last").fg(Color::GRAY))
            .push(Text::new("  Enter   - Select").fg(Color::GRAY))
            .push(Text::new("  q       - Quit").fg(Color::GRAY));

        // Compose layout
        Panel::new()
            .title("File Browser")
            .border_style(Style::default().fg(Color::GREEN))
            .child(
                VStack::new()
                    .gap(1)
                    .push(path_display)
                    .push(Text::new(""))
                    .push(file_list)
                    .push(Text::new(""))
                    .push(status)
                    .push(instructions),
            )
            .render(ctx)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Let the list handle navigation first
        let mut file_list = List::new(self.files.clone(), self.selected.clone());
        let result = file_list.handle_event(event, ctx);

        if result != EventResult::Ignored {
            return result;
        }

        // Handle our custom actions
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Enter => {
                    self.handle_select();
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}

/// Format file size in human-readable form
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
