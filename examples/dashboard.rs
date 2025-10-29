//! Dashboard Example
//!
//! Demonstrates:
//! - Panel widget with borders and titles
//! - HStack and VStack composition
//! - Multiple reactive signals
//! - Styled text
//!
//! Controls:
//! - Press 'r' to refresh stats
//! - Press 'q' to quit

use rsdrav::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> rsdrav::Result<()> {
    App::new()?.root(Dashboard::new()).run()
}

/// Dashboard with multiple panels showing stats
struct Dashboard {
    cpu_usage: Signal<f32>,
    memory_usage: Signal<f32>,
    uptime: Signal<u64>,
    status: Signal<String>,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            cpu_usage: Signal::new(42.5),
            memory_usage: Signal::new(68.2),
            uptime: Signal::new(3600),
            status: Signal::new("Running".to_string()),
        }
    }

    fn refresh_stats(&self) {
        // Simulate updating stats
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cpu_usage.set((now % 100) as f32);
        self.memory_usage.set(((now * 7) % 100) as f32);
        self.uptime.update(|v| *v += 1);
    }
}

impl Component for Dashboard {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        // Header
        let header = Text::new("=== System Dashboard ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        // CPU Panel
        let cpu_panel = Panel::new()
            .title("CPU")
            .border_style(Style::default().fg(Color::CYAN))
            .child(
                VStack::new()
                    .push(
                        Text::bind({
                            let c = self.cpu_usage.clone();
                            move || format!("Usage: {:.1}%", c.get())
                        })
                        .fg(Color::GREEN),
                    )
                    .push(Text::new("Cores: 8").fg(Color::GRAY)),
            );

        // Memory Panel
        let memory_panel = Panel::new()
            .title("Memory")
            .border_style(Style::default().fg(Color::MAGENTA))
            .child(
                VStack::new()
                    .push(
                        Text::bind({
                            let m = self.memory_usage.clone();
                            move || format!("Usage: {:.1}%", m.get())
                        })
                        .fg(Color::GREEN),
                    )
                    .push(Text::new("Total: 16 GB").fg(Color::GRAY)),
            );

        // Status Panel
        let status_panel = Panel::new()
            .title("Status")
            .border_style(Style::default().fg(Color::GREEN))
            .child(
                VStack::new()
                    .push(
                        Text::bind({
                            let s = self.status.clone();
                            move || format!("State: {}", s.get())
                        })
                        .fg(Color::GREEN),
                    )
                    .push(
                        Text::bind({
                            let u = self.uptime.clone();
                            move || format!("Uptime: {}s", u.get())
                        })
                        .fg(Color::GRAY),
                    ),
            );

        // Instructions
        let instructions = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Controls:").fg(Color::YELLOW))
            .push(Text::new("  r - Refresh stats").fg(Color::GRAY))
            .push(Text::new("  q - Quit").fg(Color::GRAY));

        // Compose layout
        VStack::new()
            .gap(1)
            .push(header)
            .push(Text::new(""))
            .push(cpu_panel)
            .push(Text::new(""))
            .push(memory_panel)
            .push(Text::new(""))
            .push(status_panel)
            .push(instructions)
            .render(_ctx)
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.refresh_stats();
                    return EventResult::Handled;
                }
                _ => {}
            },
            _ => {}
        }

        EventResult::Ignored
    }

    fn mount(&mut self, _ctx: &mut MountContext) {
        // Initialize with current time
        self.refresh_stats();
    }
}
