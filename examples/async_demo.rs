//! Async Example
//!
//! Demonstrates async support with tokio feature
//! Requires: --features tokio
//!
//! Shows:
//! - Background async tasks
//! - Async data fetching simulation
//! - Signal updates from async context

#[cfg(feature = "tokio")]
use rsdrav::async_support::AsyncRuntime;
#[cfg(feature = "tokio")]
use rsdrav::prelude::*;
#[cfg(feature = "tokio")]
use std::time::Duration;

#[cfg(not(feature = "tokio"))]
fn main() {
    eprintln!("This example requires the 'tokio' feature.");
    eprintln!("Run with: cargo run --example async_demo --features tokio");
    std::process::exit(1);
}

#[cfg(feature = "tokio")]
fn main() -> rsdrav::Result<()> {
    // Create app with async support
    let app = App::new()?.with_async()?;

    // Spawn background task
    let runtime = AsyncRuntime::new()?;
    let counter = Signal::new(0);

    let counter_clone = counter.clone();
    runtime.spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            counter_clone.update(|v| *v += 1);
        }
    });

    app.root(AsyncDemo::new(counter)).run()
}

#[cfg(feature = "tokio")]
struct AsyncDemo {
    counter: Signal<i32>,
    data: Signal<String>,
}

#[cfg(feature = "tokio")]
impl AsyncDemo {
    fn new(counter: Signal<i32>) -> Self {
        Self {
            counter,
            data: Signal::new("Waiting...".to_string()),
        }
    }

    #[allow(dead_code)]
    async fn fetch_data(&self) {
        use tokio::time::{sleep, Duration};

        self.data.set("Fetching...".to_string());
        sleep(Duration::from_secs(2)).await;
        self.data.set("Data loaded!".to_string());
    }
}

#[cfg(feature = "tokio")]
impl Component for AsyncDemo {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        let title = Text::new("=== Async Demo ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        let counter_text = Text::bind({
            let c = self.counter.clone();
            move || format!("Background counter: {}", c.get())
        })
        .fg(Color::GREEN);

        let data_text = Text::bind({
            let d = self.data.clone();
            move || format!("Status: {}", d.get())
        })
        .fg(Color::CYAN);

        let instructions = VStack::new()
            .push(Text::new(""))
            .push(Text::new("This example shows async background tasks").fg(Color::GRAY))
            .push(Text::new("The counter increments every second via async task").fg(Color::GRAY))
            .push(Text::new(""))
            .push(Text::new("Press 'q' to quit").fg(Color::GRAY));

        VStack::new()
            .gap(1)
            .push(title)
            .push(Text::new(""))
            .push(counter_text)
            .push(data_text)
            .push(instructions)
            .render(_ctx)
    }

    fn handle_event(&mut self, _event: &Event, _ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }
}
