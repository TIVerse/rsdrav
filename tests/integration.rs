// Integration tests for rsdrav
// Tests that multiple subsystems work together correctly

use rsdrav::prelude::*;

#[test]
fn test_app_creation() {
    let app = App::new();
    assert!(app.is_ok());
}

#[test]
fn test_signal_with_store() {
    let store = Store::new();

    // Create signal through store
    let count: Signal<i32> = store.get_or_create("count", 0);
    assert_eq!(count.get(), 0);

    // Update it
    count.set(42);

    // Get it again from store
    let count2: Signal<i32> = store.get("count").unwrap();
    assert_eq!(count2.get(), 42);
}

#[test]
fn test_derived_with_signals() {
    let a = Signal::new(5);
    let b = Signal::new(10);

    // Create derived computation
    let sum = {
        let a = a.clone();
        let b = b.clone();
        Derived::new(move || a.get() + b.get())
    };

    assert_eq!(sum.get(), 15);

    // Update source signals
    a.set(20);
    sum.invalidate();
    assert_eq!(sum.get(), 30);
}

#[test]
fn test_layout_with_buffer() {
    let mut buffer = Buffer::new(100, 50);

    // Create layout
    let row = Row::new().gap(2);
    let area = Rect::new(0, 0, 100, 50);
    let rects = row.layout(area, &[Length::Fill(1), Length::Fill(1), Length::Fill(1)]);

    assert_eq!(rects.len(), 3);

    // Draw into buffer using layout
    for (i, rect) in rects.iter().enumerate() {
        let ch = char::from_digit(i as u32, 10).unwrap();
        for y in rect.y..(rect.y + rect.height) {
            for x in rect.x..(rect.x + rect.width) {
                buffer.set(x, y, Cell::new(ch));
            }
        }
    }

    // Verify buffer was written
    assert_ne!(buffer.get(10, 10).unwrap().ch, '\0');
}

#[test]
fn test_command_with_store() {
    let store = Store::new();
    let mut ctx = CommandContext::new(store.clone());

    // Create a test counter
    store.set("test_count", 0);

    // Create command handler
    struct IncrementHandler;
    impl CommandHandler for IncrementHandler {
        fn execute(
            &mut self,
            _cmd: Command,
            ctx: &mut CommandContext,
        ) -> rsdrav::Result<CommandResult> {
            let count: Signal<i32> = ctx.store.get_or_create("test_count", 0);
            count.update(|val| *val += 1);
            Ok(CommandResult::success())
        }
    }

    let mut handler = IncrementHandler;
    let cmd = Command::new("increment");

    handler.execute(cmd, &mut ctx).unwrap();

    let count: Signal<i32> = store.get("test_count").unwrap();
    assert_eq!(count.get(), 1);
}

#[test]
fn test_style_rendering() {
    let style = Style::new()
        .fg(Color::RED)
        .bg(Color::BLUE)
        .add_modifier(Modifier::BOLD);

    let cell = Cell::with_style('X', style);

    assert_eq!(cell.ch, 'X');
    assert_eq!(cell.style.fg, Some(Color::RED));
    assert_eq!(cell.style.bg, Some(Color::BLUE));
    assert!(cell.style.modifiers.contains(Modifier::BOLD));
}

#[test]
fn test_diff_rendering_pipeline() {
    // Simulate a frame update
    let mut buf1 = Buffer::new(20, 10);
    let mut buf2 = Buffer::new(20, 10);

    // Initial frame
    for x in 0..20 {
        buf1.set(x, 5, Cell::new('='));
    }

    // Next frame - change one line
    buf2 = buf1.clone();
    for x in 0..20 {
        buf2.set(x, 5, Cell::new('-'));
    }

    // Compute diff
    let dirty = rsdrav::render::compute_diff(&buf1, &buf2);

    // Should have detected the change
    assert!(!dirty.is_empty());
    assert_eq!(dirty[0].rect.y, 5);
}

#[test]
fn test_complete_reactive_flow() {
    // Simulate a complete reactive update flow
    let store = Store::new();

    // 1. Create reactive state
    let count = store.get_or_create("count", 0);
    let doubled = {
        let count = count.clone();
        Derived::new(move || count.get() * 2)
    };

    // 2. Subscribe to changes
    let notifications = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let n = notifications.clone();
    let _sub = count.subscribe(move |_| {
        n.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });

    // 3. Update state
    count.set(5);

    // 4. Verify reactive updates
    assert_eq!(notifications.load(std::sync::atomic::Ordering::SeqCst), 1);

    doubled.invalidate();
    assert_eq!(doubled.get(), 10);

    // 5. Multiple updates
    count.update(|val| *val += 1);
    assert_eq!(count.get(), 6);
    assert_eq!(notifications.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[test]
fn test_rect_layout_math() {
    let area = Rect::new(0, 0, 100, 100);

    // Test splitting
    let (left, right) = area.split_h(50);
    assert_eq!(left.width, 50);
    assert_eq!(right.width, 50);
    assert_eq!(right.x, 50);

    // Test intersection
    let r1 = Rect::new(10, 10, 20, 20);
    let r2 = Rect::new(20, 20, 20, 20);
    let inter = r1.intersect(&r2).unwrap();
    assert_eq!(inter.x, 20);
    assert_eq!(inter.y, 20);
    assert_eq!(inter.width, 10);
    assert_eq!(inter.height, 10);

    // Test union
    let union = r1.union(&r2);
    assert_eq!(union.x, 10);
    assert_eq!(union.y, 10);
    assert_eq!(union.width, 30);
    assert_eq!(union.height, 30);
}

#[test]
fn test_event_conversions() {
    // Test key event creation
    let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    assert_eq!(key.code, KeyCode::Char('a'));
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));

    // Test modifier combinations
    let mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert!(mods.contains(KeyModifiers::CONTROL));
    assert!(mods.contains(KeyModifiers::SHIFT));
    assert!(!mods.contains(KeyModifiers::ALT));
}

#[test]
fn test_command_parsing_edge_cases() {
    use rsdrav::command::parse;

    // Empty strings
    assert!(parse("").is_err());
    assert!(parse("   ").is_err());

    // Single command
    let cmd = parse("quit").unwrap();
    assert_eq!(cmd.name, "quit");
    assert_eq!(cmd.args.len(), 0);

    // Multiple spaces
    let cmd = parse("echo    hello    world").unwrap();
    assert_eq!(cmd.args, vec!["hello", "world"]);

    // Quotes with spaces
    let cmd = parse(r#"set "my var" "some value""#).unwrap();
    assert_eq!(cmd.args, vec!["my var", "some value"]);

    // Escaped characters
    let cmd = parse(r"path /home/user\ name/file").unwrap();
    assert_eq!(cmd.args, vec!["/home/user name/file"]);
}
