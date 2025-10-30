//! Memory leak detection test
//!
//! Run with: cargo run --release --bin memory_test
//!
//! Monitor with:
//! - valgrind --leak-check=full --show-leak-kinds=all ./target/release/memory_test
//! - heaptrack ./target/release/memory_test

use rsdrav::prelude::*;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("Memory Leak Detection Test");
    println!("===========================\n");

    // Test 1: Signal creation and dropping
    test_signal_lifecycle();

    // Test 2: Component lifecycle
    test_component_lifecycle();

    // Test 3: Subscription cleanup
    test_subscription_cleanup();

    // Test 4: Large data structures
    test_large_data();

    // Test 5: Rapid creation/destruction
    test_rapid_churn();

    println!("\n✅ All memory tests completed!");
    println!("Run with valgrind/heaptrack for detailed analysis");

    Ok(())
}

fn test_signal_lifecycle() {
    println!("Test 1: Signal Lifecycle");

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let sig = Signal::new(i);
        let _value = sig.get();
        // Signal should be dropped here
    }

    println!(
        "  Created and dropped {} signals in {:?}",
        iterations,
        start.elapsed()
    );
}

fn test_component_lifecycle() {
    println!("\nTest 2: Component Lifecycle");

    struct TestComponent {
        _data: Signal<Vec<u8>>,
    }

    impl TestComponent {
        fn new() -> Self {
            Self {
                _data: Signal::new(vec![0u8; 1024]), // 1KB per component
            }
        }
    }

    impl Component for TestComponent {
        fn render(&self, _ctx: &RenderContext) -> ViewNode {
            ViewNode::text("test")
        }
    }

    let iterations = 1_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _comp = TestComponent::new();
        // Should be dropped with data
    }

    println!(
        "  Created and dropped {} components in {:?}",
        iterations,
        start.elapsed()
    );
}

fn test_subscription_cleanup() {
    println!("\nTest 3: Subscription Cleanup");

    let sig = Signal::new(0);
    let iterations = 1_000;

    let start = Instant::now();

    for i in 0..iterations {
        let subscription_id = sig.subscribe(move |v| {
            let _ = v + i; // Capture i to make unique closure
        });

        // In real code, subscription should be dropped when component is dropped
        drop(subscription_id);
    }

    println!(
        "  Created and dropped {} subscriptions in {:?}",
        iterations,
        start.elapsed()
    );
}

fn test_large_data() {
    println!("\nTest 4: Large Data Structures");

    // Simulate large list widget
    let size = 10_000;
    let items: Vec<String> = (0..size).map(|i| format!("Item {}", i)).collect();

    let start = Instant::now();

    {
        let items_signal = Signal::new(items.clone());
        let selected = Signal::new(Some(0));

        let _list = List::new(items_signal, selected);

        // List and all data should be dropped here
    }

    println!(
        "  Created and dropped large list ({} items) in {:?}",
        size,
        start.elapsed()
    );
}

fn test_rapid_churn() {
    println!("\nTest 5: Rapid Creation/Destruction");

    let duration = Duration::from_secs(2);
    let start = Instant::now();
    let mut count = 0;

    while start.elapsed() < duration {
        // Rapidly create and drop components
        let sig = Signal::new(count);
        let _text = Text::new(format!("Count: {}", sig.get()));

        let items = Signal::new(vec![1, 2, 3]);
        let selected = Signal::new(Some(0));
        let _list = List::new(items, selected);

        count += 1;
    }

    println!("  Churned {} components in {:?}", count, duration);
}
