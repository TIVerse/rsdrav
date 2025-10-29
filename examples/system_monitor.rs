//! System Monitor Example
//!
//! Demonstrates:
//! - ProgressBar for CPU/Memory usage
//! - Table for process list
//! - Panel for grouping sections
//! - Reactive state updates
//! - Real-time-like UI updates
//!
//! Controls:
//! - r - Refresh stats
//! - q - Quit

use rsdrav::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> rsdrav::Result<()> {
    App::new()?.root(SystemMonitor::new()).run()
}

/// System monitor component
struct SystemMonitor {
    cpu_usage: Signal<f32>,
    memory_usage: Signal<f32>,
    disk_usage: Signal<f32>,
    network_rx: Signal<f32>,
    network_tx: Signal<f32>,
    processes: Signal<Vec<Process>>,
    selected_process: Signal<Option<usize>>,
    uptime: Signal<u64>,
}

#[derive(Clone, Debug)]
struct Process {
    name: String,
    cpu: f32,
    memory: f32,
    status: String,
}

impl SystemMonitor {
    fn new() -> Self {
        Self {
            cpu_usage: Signal::new(0.0),
            memory_usage: Signal::new(0.0),
            disk_usage: Signal::new(0.0),
            network_rx: Signal::new(0.0),
            network_tx: Signal::new(0.0),
            processes: Signal::new(Self::generate_mock_processes()),
            selected_process: Signal::new(Some(0)),
            uptime: Signal::new(0),
        }
    }

    fn generate_mock_processes() -> Vec<Process> {
        vec![
            Process {
                name: "rsdrav".into(),
                cpu: 2.5,
                memory: 45.2,
                status: "Running".into(),
            },
            Process {
                name: "cargo".into(),
                cpu: 0.8,
                memory: 120.5,
                status: "Running".into(),
            },
            Process {
                name: "rust-analyzer".into(),
                cpu: 15.3,
                memory: 580.0,
                status: "Running".into(),
            },
            Process {
                name: "firefox".into(),
                cpu: 8.2,
                memory: 1250.0,
                status: "Running".into(),
            },
            Process {
                name: "systemd".into(),
                cpu: 0.1,
                memory: 12.0,
                status: "Running".into(),
            },
        ]
    }

    fn refresh_stats(&self) {
        // Simulate reading system stats
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Fake varying stats based on time
        self.cpu_usage.set(((now % 100) as f32) / 100.0);
        self.memory_usage.set(((now * 7 % 100) as f32) / 100.0);
        self.disk_usage.set(0.65);
        self.network_rx.set(((now % 50) as f32) / 10.0);
        self.network_tx.set(((now % 30) as f32) / 10.0);
        self.uptime.update(|v| *v += 1);
    }
}

impl Component for SystemMonitor {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Header
        let header = Text::new("=== System Monitor ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        // CPU Usage Section
        let cpu_panel = Panel::new()
            .title("CPU")
            .border_style(Style::default().fg(Color::GREEN))
            .child(
                VStack::new()
                    .push(
                        ProgressBar::new(self.cpu_usage.clone())
                            .label("Usage")
                            .width(30)
                            .filled_color(Color::GREEN),
                    )
                    .push(
                        Text::bind({
                            let c = self.cpu_usage.clone();
                            move || format!("  {:.1}%", c.get() * 100.0)
                        })
                        .fg(Color::GRAY),
                    ),
            );

        // Memory Usage Section
        let mem_panel = Panel::new()
            .title("Memory")
            .border_style(Style::default().fg(Color::CYAN))
            .child(
                VStack::new()
                    .push(
                        ProgressBar::new(self.memory_usage.clone())
                            .label("RAM")
                            .width(30)
                            .filled_color(Color::CYAN),
                    )
                    .push(
                        Text::bind({
                            let m = self.memory_usage.clone();
                            move || format!("  {:.1}% (8.2/16 GB)", m.get() * 100.0)
                        })
                        .fg(Color::GRAY),
                    ),
            );

        // Disk Usage Section
        let disk_panel = Panel::new()
            .title("Disk")
            .border_style(Style::default().fg(Color::MAGENTA))
            .child(
                VStack::new()
                    .push(
                        ProgressBar::new(self.disk_usage.clone())
                            .label("SSD")
                            .width(30)
                            .filled_color(Color::MAGENTA),
                    )
                    .push(Text::new("  65% (325/500 GB)").fg(Color::GRAY)),
            );

        // Network Section
        let net_section = VStack::new()
            .push(Text::new("Network:").fg(Color::YELLOW))
            .push(
                Text::bind({
                    let rx = self.network_rx.clone();
                    move || format!("  ↓ RX: {:.1} MB/s", rx.get())
                })
                .fg(Color::GREEN),
            )
            .push(
                Text::bind({
                    let tx = self.network_tx.clone();
                    move || format!("  ↑ TX: {:.1} MB/s", tx.get())
                })
                .fg(Color::BLUE),
            );

        // Process Table
        let process_table = Table::new(self.processes.clone(), self.selected_process.clone())
            .column(TableColumn::new("Process", 20).render(|p: &Process| p.name.clone()))
            .column(TableColumn::new("CPU%", 8).render(|p: &Process| format!("{:.1}%", p.cpu)))
            .column(
                TableColumn::new("Memory", 10).render(|p: &Process| format!("{:.1}MB", p.memory)),
            )
            .column(TableColumn::new("Status", 10).render(|p: &Process| p.status.clone()))
            .visible_height(5);

        let process_panel = Panel::new()
            .title("Top Processes")
            .border_style(Style::default().fg(Color::BLUE))
            .child(process_table);

        // Controls
        let controls = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Controls:").fg(Color::YELLOW))
            .push(Text::new("  r - Refresh stats").fg(Color::GRAY))
            .push(Text::new("  ↑/↓ - Navigate processes").fg(Color::GRAY))
            .push(Text::new("  q - Quit").fg(Color::GRAY));

        // Main layout
        VStack::new()
            .gap(1)
            .push(header)
            .push(Text::new(""))
            .push(cpu_panel)
            .push(mem_panel)
            .push(disk_panel)
            .push(Text::new(""))
            .push(net_section)
            .push(Text::new(""))
            .push(process_panel)
            .push(controls)
            .render(ctx)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Let process table handle navigation
        let mut process_table = Table::new(self.processes.clone(), self.selected_process.clone());
        let result = process_table.handle_event(event, ctx);
        if result != EventResult::Ignored {
            return result;
        }

        // Handle refresh
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.refresh_stats();
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }

    fn mount(&mut self, _ctx: &mut MountContext) {
        // Initialize with some stats
        self.refresh_stats();
    }
}
