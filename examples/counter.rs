//! Interactive Counter Example
//!
//! Demonstrates:
//! - Reactive state with Signal
//! - Button widget with click handlers
//! - VStack layout
//! - Component lifecycle
//!
//! Controls:
//! - Press '+' or '-' to change counter
//! - Press 'q' to quit

use rsdrav::prelude::*;

fn main() -> rsdrav::Result<()> {
    App::new()?.root(CounterApp::new()).run()
}

/// Main counter component
struct CounterApp {
    count: Signal<i32>,
}

impl CounterApp {
    fn new() -> Self {
        Self {
            count: Signal::new(0),
        }
    }
}

impl Component for CounterApp {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        // Build UI using VStack and widgets
        let count_text = Text::bind({
            let c = self.count.clone();
            move || format!("Count: {}", c.get())
        })
        .fg(Color::CYAN)
        .add_modifier(Modifier::BOLD);

        let instructions =
            Text::new("Press '+' to increment, '-' to decrement, 'q' to quit").fg(Color::GRAY);

        // Create button-like text (actual Button needs mouse support)
        let inc_btn = Text::new("[+] Increment").fg(Color::GREEN);

        let dec_btn = Text::new("[-] Decrement").fg(Color::RED);

        // Stack them vertically
        VStack::new()
            .gap(1)
            .push(Text::new("=== Counter Demo ===").fg(Color::YELLOW))
            .push(Text::new("")) // Spacer
            .push(count_text)
            .push(Text::new("")) // Spacer
            .push(inc_btn)
            .push(dec_btn)
            .push(Text::new("")) // Spacer
            .push(instructions)
            .render(_ctx)
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        // Handle keyboard input
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    self.count.update(|v| *v += 1);
                    return EventResult::Handled;
                }
                KeyCode::Char('-') | KeyCode::Char('_') => {
                    self.count.update(|v| *v -= 1);
                    return EventResult::Handled;
                }
                KeyCode::Up => {
                    self.count.update(|v| *v += 1);
                    return EventResult::Handled;
                }
                KeyCode::Down => {
                    self.count.update(|v| *v -= 1);
                    return EventResult::Handled;
                }
                KeyCode::Char('r') => {
                    // Reset to zero
                    self.count.set(0);
                    return EventResult::Handled;
                }
                _ => {}
            }
        }

        EventResult::Ignored
    }
}
