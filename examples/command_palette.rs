//! Command Palette Example
//!
//! Demonstrates:
//! - Input widget for command entry
//! - List widget for filtered commands
//! - Command execution
//! - Fuzzy matching (simple)
//! - Modal overlay
//!
//! Controls:
//! - Ctrl+P - Open command palette
//! - Type to filter commands
//! - ↑/↓ - Navigate commands
//! - Enter - Execute command
//! - Esc - Close palette
//! - q - Quit (when palette closed)

use rsdrav::prelude::*;

fn main() -> rsdrav::Result<()> {
    App::new()?.root(CommandPaletteDemo::new()).run()
}

#[derive(Clone, Debug)]
struct Command {
    name: String,
    description: String,
    category: String,
}

struct CommandPaletteDemo {
    palette_visible: Signal<bool>,
    search_query: Signal<String>,
    all_commands: Vec<Command>,
    filtered_commands: Signal<Vec<Command>>,
    selected: Signal<Option<usize>>,
    last_executed: Signal<Option<String>>,
    status_message: Signal<String>,
}

impl CommandPaletteDemo {
    fn new() -> Self {
        let all_commands = Self::create_commands();

        Self {
            palette_visible: Signal::new(false),
            search_query: Signal::new(String::new()),
            all_commands: all_commands.clone(),
            filtered_commands: Signal::new(all_commands),
            selected: Signal::new(Some(0)),
            last_executed: Signal::new(None),
            status_message: Signal::new("Press Ctrl+P to open command palette".into()),
        }
    }

    fn create_commands() -> Vec<Command> {
        vec![
            Command {
                name: "File: New".into(),
                description: "Create a new file".into(),
                category: "File".into(),
            },
            Command {
                name: "File: Open".into(),
                description: "Open an existing file".into(),
                category: "File".into(),
            },
            Command {
                name: "File: Save".into(),
                description: "Save current file".into(),
                category: "File".into(),
            },
            Command {
                name: "Edit: Copy".into(),
                description: "Copy selection to clipboard".into(),
                category: "Edit".into(),
            },
            Command {
                name: "Edit: Paste".into(),
                description: "Paste from clipboard".into(),
                category: "Edit".into(),
            },
            Command {
                name: "View: Toggle Theme".into(),
                description: "Switch between light and dark theme".into(),
                category: "View".into(),
            },
            Command {
                name: "View: Zoom In".into(),
                description: "Increase font size".into(),
                category: "View".into(),
            },
            Command {
                name: "View: Zoom Out".into(),
                description: "Decrease font size".into(),
                category: "View".into(),
            },
            Command {
                name: "Terminal: New".into(),
                description: "Open a new terminal".into(),
                category: "Terminal".into(),
            },
            Command {
                name: "Terminal: Split".into(),
                description: "Split terminal pane".into(),
                category: "Terminal".into(),
            },
            Command {
                name: "Help: Documentation".into(),
                description: "Open documentation".into(),
                category: "Help".into(),
            },
            Command {
                name: "Help: Shortcuts".into(),
                description: "View keyboard shortcuts".into(),
                category: "Help".into(),
            },
        ]
    }

    fn open_palette(&mut self) {
        self.palette_visible.set(true);
        self.search_query.set(String::new());
        self.filter_commands();
        self.selected.set(Some(0));
    }

    fn close_palette(&mut self) {
        self.palette_visible.set(false);
        self.search_query.set(String::new());
    }

    fn filter_commands(&mut self) {
        let query = self.search_query.get().to_lowercase();

        if query.is_empty() {
            self.filtered_commands.set(self.all_commands.clone());
        } else {
            // Simple substring matching
            let filtered: Vec<Command> = self
                .all_commands
                .iter()
                .filter(|cmd| {
                    cmd.name.to_lowercase().contains(&query)
                        || cmd.description.to_lowercase().contains(&query)
                        || cmd.category.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();

            self.filtered_commands.set(filtered);
        }

        // Reset selection
        self.selected.set(Some(0));
    }

    fn execute_selected(&mut self) {
        let commands = self.filtered_commands.get();
        if let Some(idx) = self.selected.get() {
            if let Some(cmd) = commands.get(idx) {
                self.last_executed.set(Some(cmd.name.clone()));
                self.status_message.set(format!("Executed: {}", cmd.name));
                self.close_palette();
            }
        }
    }
}

impl Component for CommandPaletteDemo {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Main app view
        let header = Text::new("=== Command Palette Demo ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        let status = Text::bind({
            let s = self.status_message.clone();
            move || s.get()
        })
        .fg(Color::CYAN);

        let last_cmd = if let Some(ref cmd) = self.last_executed.get() {
            VStack::new()
                .push(Text::new("Last Executed Command:").fg(Color::YELLOW))
                .push(Text::new(cmd.clone()).fg(Color::GREEN))
        } else {
            VStack::new().push(Text::new("No command executed yet").fg(Color::GRAY))
        };

        let help = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Controls:").fg(Color::YELLOW))
            .push(Text::new("  Ctrl+P - Open command palette").fg(Color::GREEN))
            .push(Text::new("  q - Quit").fg(Color::GRAY));

        let main_content = VStack::new()
            .gap(1)
            .push(header)
            .push(status)
            .push(Text::new(""))
            .push(last_cmd)
            .push(help);

        // Command palette overlay (modal)
        if self.palette_visible.get() {
            let search_input = Input::new(self.search_query.clone())
                .placeholder("Type to search commands...")
                .focused(true);

            let command_list = List::new(self.filtered_commands.clone(), self.selected.clone())
                .visible_height(8)
                .render_item(|cmd: &Command, is_selected| {
                    let icon = match cmd.category.as_str() {
                        "File" => "📄",
                        "Edit" => "✏️",
                        "View" => "👁️",
                        "Terminal" => "💻",
                        "Help" => "❓",
                        _ => "⚙️",
                    };

                    let style = if is_selected {
                        Style::default().bg(Color::BLUE).fg(Color::WHITE)
                    } else {
                        Style::default().fg(Color::WHITE)
                    };

                    let text = format!("{} {} - {}", icon, cmd.name, cmd.description);
                    ViewNode::text_styled(text, style)
                });

            let palette_content = VStack::new()
                .push(
                    Text::new("Command Palette")
                        .fg(Color::YELLOW)
                        .add_modifier(Modifier::BOLD),
                )
                .push(Text::new(""))
                .push(search_input)
                .push(Text::new(""))
                .push(command_list)
                .push(Text::new(""))
                .push(Text::new("↑/↓ Navigate  Enter Execute  Esc Close").fg(Color::GRAY));

            let palette = Panel::new()
                .title("🔍 Commands")
                .border_style(Style::default().fg(Color::CYAN))
                .child(palette_content);

            // Combine main content with palette overlay
            VStack::new()
                .push(main_content)
                .push(Text::new(""))
                .push(palette)
                .render(ctx)
        } else {
            main_content.render(ctx)
        }
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Global shortcuts
        if let Event::Key(key) = event {
            // Ctrl+P to open palette
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') && !self.palette_visible.get() {
                self.open_palette();
                return EventResult::Consumed;
            }

            // Quit when palette is closed
            if !self.palette_visible.get() && key.code == KeyCode::Char('q') {
                // Let default handler quit
                return EventResult::Ignored;
            }
        }

        // Handle palette events
        if self.palette_visible.get() {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Esc => {
                        self.close_palette();
                        return EventResult::Consumed;
                    }
                    KeyCode::Enter => {
                        self.execute_selected();
                        return EventResult::Consumed;
                    }
                    KeyCode::Char(_) | KeyCode::Backspace => {
                        // Let input handle it first
                        let mut input = Input::new(self.search_query.clone()).focused(true);
                        let result = input.handle_event(event, ctx);
                        if result == EventResult::Handled {
                            self.filter_commands();
                            return result;
                        }
                    }
                    KeyCode::Up | KeyCode::Down => {
                        // Let list handle navigation
                        let mut list =
                            List::new(self.filtered_commands.clone(), self.selected.clone());
                        return list.handle_event(event, ctx);
                    }
                    _ => {}
                }
            }
        }

        EventResult::Ignored
    }
}
