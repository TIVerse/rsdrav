//! Stress test for stability and performance
//!
//! Run with: cargo run --release --bin stress_test

use rsdrav::prelude::*;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("🔥 rsdrav Stress Test Suite");
    println!("===========================\n");

    // Test 1: Rapid state updates
    test_rapid_updates()?;

    // Test 2: Large component tree
    test_deep_nesting()?;

    // Test 3: Many simultaneous signals
    test_signal_storm()?;

    // Test 4: Event flooding
    test_event_flood()?;

    // Test 5: Continuous rendering
    test_continuous_render()?;

    println!("\n✅ All stress tests passed!");

    Ok(())
}

fn test_rapid_updates() -> Result<()> {
    println!("Test 1: Rapid State Updates");

    let sig = Signal::new(0);
    let updates = 100_000;

    let start = Instant::now();

    for i in 0..updates {
        sig.set(i);
    }

    let elapsed = start.elapsed();
    let rate = updates as f64 / elapsed.as_secs_f64();

    println!("  ✓ {} updates in {:?}", updates, elapsed);
    println!("  ✓ Rate: {:.0} updates/sec", rate);

    assert!(rate > 100_000.0, "Update rate too slow!");

    Ok(())
}

fn test_deep_nesting() -> Result<()> {
    println!("\nTest 2: Deep Component Nesting");

    fn create_nested(depth: usize) -> VStack {
        if depth == 0 {
            VStack::new().push(Text::new("Leaf"))
        } else {
            VStack::new()
                .push(Text::new(format!("Level {}", depth)))
                .push(create_nested(depth - 1))
        }
    }

    let max_depth = 100;
    let start = Instant::now();

    let tree = create_nested(max_depth);

    let mut buffer = Buffer::new(80, 24);
    let store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let ctx = RenderContext::new(&mut buffer, area, &store);

    let _node = tree.render(&ctx);

    println!(
        "  ✓ Rendered {} deep tree in {:?}",
        max_depth,
        start.elapsed()
    );

    Ok(())
}

fn test_signal_storm() -> Result<()> {
    println!("\nTest 3: Signal Storm");

    let count = 1_000;
    let start = Instant::now();

    // Create many signals
    let signals: Vec<Signal<i32>> = (0..count).map(Signal::new).collect();

    // Update all simultaneously
    for (i, sig) in signals.iter().enumerate() {
        sig.set(i as i32 * 2);
    }

    // Read all
    let sum: i32 = signals.iter().map(|s| s.get()).sum();

    println!("  ✓ Managed {} signals in {:?}", count, start.elapsed());
    println!("  ✓ Sum: {}", sum);

    Ok(())
}

fn test_event_flood() -> Result<()> {
    println!("\nTest 4: Event Flood");

    struct EventCounter {
        count: Signal<usize>,
    }

    impl EventCounter {
        fn new() -> Self {
            Self {
                count: Signal::new(0),
            }
        }
    }

    impl Component for EventCounter {
        fn render(&self, _ctx: &RenderContext) -> ViewNode {
            ViewNode::text(format!("Count: {}", self.count.get()))
        }

        fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
            if matches!(event, Event::Key(_)) {
                self.count.update(|v| *v += 1);
                EventResult::Handled
            } else {
                EventResult::Ignored
            }
        }
    }

    let mut comp = EventCounter::new();
    let mut store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let mut ctx = EventContext {
        store: &mut store,
        area,
    };

    let event_count = 10_000;
    let start = Instant::now();

    for _ in 0..event_count {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        comp.handle_event(&event, &mut ctx);
    }

    println!(
        "  ✓ Processed {} events in {:?}",
        event_count,
        start.elapsed()
    );
    println!("  ✓ Final count: {}", comp.count.get());

    Ok(())
}

fn test_continuous_render() -> Result<()> {
    println!("\nTest 5: Continuous Rendering");

    let count = Signal::new(0);
    let duration = Duration::from_secs(2);
    let start = Instant::now();
    let mut frames = 0;

    let mut buffer = Buffer::new(120, 40);
    let store = Store::new();
    let area = Rect::new(0, 0, 120, 40);

    while start.elapsed() < duration {
        count.update(|v| *v += 1);

        let text = Text::bind({
            let c = count.clone();
            move || format!("Frame: {}", c.get())
        });

        let ctx = RenderContext::new(&mut buffer, area, &store);
        let _node = text.render(&ctx);

        frames += 1;
    }

    let fps = frames as f64 / duration.as_secs_f64();

    println!("  ✓ Rendered {} frames in {:?}", frames, duration);
    println!("  ✓ Average FPS: {:.1}", fps);

    assert!(fps > 1000.0, "Render rate too slow!");

    Ok(())
}
