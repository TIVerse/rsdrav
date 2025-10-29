# rsdrav - Complete Developer Guide

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core Concepts](#core-concepts)
3. [Widgets Guide](#widgets-guide)
4. [Layout System](#layout-system)
5. [Reactive State](#reactive-state)
6. [Event Handling](#event-handling)
7. [Focus Management](#focus-management)
8. [Advanced Topics](#advanced-topics)
9. [Plugin Development](#plugin-development)
10. [Performance](#performance)

---

## Quick Start

### Installation

```toml
[dependencies]
rsdrav = { version = "0.1", features = ["crossterm", "tokio"] }
```

### Hello World

```rust
use rsdrav::prelude::*;

fn main() -> Result<()> {
    App::new()?
        .root(HelloWorld)
        .run()
}

struct HelloWorld;

impl Component for HelloWorld {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        VStack::new()
            .push(Text::new("Hello, rsdrav!").fg(Color::GREEN).bold())
            .push(Text::new("Press 'q' to quit").fg(Color::GRAY))
            .render(ctx)
    }
}
```

---

## Core Concepts

### Component Lifecycle

Every component implements the `Component` trait:

```rust
pub trait Component {
    // Required: Render the component's UI
    fn render(&self, ctx: &RenderContext) -> ViewNode;
    
    // Optional: Handle input events
    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        EventResult::Ignored
    }
    
    // Optional: Called when component is mounted
    fn mount(&mut self, ctx: &mut MountContext) {}
    
    // Optional: Called when component is unmounted
    fn unmount(&mut self, ctx: &mut MountContext) {}
    
    // Optional: Called on state updates
    fn update(&mut self, ctx: &mut UpdateContext) {}
}
```

### Reactive State

State management uses `Signal` for automatic UI updates:

```rust
struct Counter {
    count: Signal<i32>,
}

impl Counter {
    fn new() -> Self {
        Self { count: Signal::new(0) }
    }
}

impl Component for Counter {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        VStack::new()
            // Text automatically updates when signal changes
            .push(Text::bind({
                let c = self.count.clone();
                move || format!("Count: {}", c.get())
            }))
            .render(ctx)
    }
    
    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Key(key) if key.code == KeyCode::Char('+') => {
                self.count.update(|v| *v += 1);
                EventResult::Handled
            }
            _ => EventResult::Ignored
        }
    }
}
```

---

## Widgets Guide

### Text

Display static or reactive text:

```rust
// Static text
Text::new("Hello").fg(Color::BLUE)

// Reactive text
Text::bind(move || format!("Count: {}", count.get()))

// Styled text
Text::new("Important!").fg(Color::RED).bold()
```

### Input

Text entry with validation:

```rust
let username = Signal::new(String::new());
let password = Signal::new(String::new());

Input::new(username.clone())
    .placeholder("Enter username")
    .max_length(20);

Input::new(password.clone())
    .password()  // Masked input
    .placeholder("Enter password");
```

### List

Scrollable lists with selection:

```rust
let items = Signal::new(vec!["Item 1", "Item 2", "Item 3"]);
let selected = Signal::new(Some(0));

List::new(items, selected)
    .render_item(|item, is_selected| {
        let style = if is_selected {
            Style::default().bg(Color::BLUE)
        } else {
            Style::default()
        };
        ViewNode::text_styled(format!("> {}", item), style)
    })
    .visible_height(10);
```

### Table

Multi-column data display:

```rust
let data = Signal::new(vec![
    User { name: "Alice", age: 30 },
    User { name: "Bob", age: 25 },
]);
let selected = Signal::new(Some(0));

Table::new(data, selected)
    .column(TableColumn::new("Name", 20)
        .render(|u: &User| u.name.clone()))
    .column(TableColumn::new("Age", 5)
        .render(|u: &User| u.age.to_string()));
```

### ProgressBar

Loading indicators:

```rust
let progress = Signal::new(0.75); // 75%

ProgressBar::new(progress)
    .label("Loading...")
    .width(40)
    .filled_color(Color::GREEN)
    .show_percentage(true);
```

### Panel

Bordered containers:

```rust
Panel::new()
    .title("Settings")
    .border_style(Style::default().fg(Color::CYAN))
    .child(content);
```

### Tabs

Multi-view navigation:

```rust
let selected_tab = Signal::new(0);

Tabs::new(selected_tab)
    .tab("Overview", overview_content)
    .tab("Details", details_content)
    .tab("Settings", settings_content);
```

### Modal

Dialog overlays:

```rust
let visible = Signal::new(true);

Modal::new(visible)
    .title("Confirm Action")
    .child(Text::new("Are you sure?"))
    .closable(true); // ESC to close
```

---

## Layout System

### Stack Layouts

```rust
// Vertical stack
VStack::new()
    .gap(1)  // Space between items
    .push(header)
    .push(content)
    .push(footer);

// Horizontal stack
HStack::new()
    .gap(2)
    .push(sidebar)
    .push(main_content);
```

### Flex Layout

CSS Flexbox-like responsive layouts:

```rust
let layout = Flex::new(FlexDirection::Row)
    .add(FlexItem::new()
        .grow(1.0)      // Takes 1 part
        .min(200))       // Min 200px
    .add(FlexItem::new()
        .grow(2.0)      // Takes 2 parts
        .max(800))       // Max 800px
    .add(FlexItem::new()
        .fixed(300));    // Fixed 300px

let rects = layout.calculate(container_rect);
```

### Row/Column Layouts

```rust
// Row layout
let row = Row::new(area)
    .add(Length::Fixed(20))
    .add(Length::Fill(1))
    .add(Length::Percent(0.3))
    .calculate();

// Column layout  
let col = Column::new(area)
    .add(Length::Fixed(3))   // Header
    .add(Length::Fill(1))    // Content
    .add(Length::Fixed(2))   // Footer
    .calculate();
```

---

## Reactive State

### Signal

Mutable reactive values:

```rust
let count = Signal::new(0);

// Read value
let value = count.get();

// Update value
count.set(42);
count.update(|v| *v += 1);

// Subscribe to changes
count.subscribe(|value| {
    println!("Count changed to: {}", value);
});
```

### Derived

Computed values that update automatically:

```rust
let a = Signal::new(2);
let b = Signal::new(3);

let sum = {
    let a = a.clone();
    let b = b.clone();
    Derived::new(move || a.get() + b.get())
};

assert_eq!(sum.get(), 5);

a.set(10);
sum.invalidate(); // Recompute
assert_eq!(sum.get(), 13);
```

### Store

Global state management:

```rust
let mut store = Store::new();

// Store any type
store.set("theme", Signal::new("dark"));
store.set("user", Signal::new(User::default()));

// Retrieve
let theme: Option<Signal<String>> = store.get("theme");
```

---

## Event Handling

### Keyboard Events

```rust
fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char('q') => {
                // Quit
                EventResult::Consumed
            }
            KeyCode::Enter => {
                // Submit
                EventResult::Handled
            }
            KeyCode::Up => {
                // Move up
                EventResult::Handled
            }
            _ => EventResult::Ignored
        },
        _ => EventResult::Ignored
    }
}
```

### Event Results

- `EventResult::Ignored` - Pass to parent
- `EventResult::Handled` - Stop propagation
- `EventResult::Consumed` - Fully consumed (e.g., quit)

---

## Focus Management

Track and navigate between components:

```rust
struct MyForm {
    focus: FocusManager,
    username_id: ComponentId,
    password_id: ComponentId,
}

impl MyForm {
    fn new() -> Self {
        let mut focus = FocusManager::new();
        let username_id = ComponentId::new(1);
        let password_id = ComponentId::new(2);
        
        focus.register(username_id, 0, true);
        focus.register(password_id, 1, true);
        
        Self { focus, username_id, password_id }
    }
}

// Tab/Shift+Tab automatically handled by framework
```

---

## Advanced Topics

### Async Operations

With `tokio` feature:

```rust
#[cfg(feature = "tokio")]
use rsdrav::prelude::*;

let runtime = AsyncRuntime::new()?;

// Spawn background task
runtime.spawn(async {
    let data = fetch_from_api().await?;
    update_ui(data);
});

// With timeout
with_timeout(
    Duration::from_secs(5),
    long_running_operation()
).await?;
```

### Plugins (Rhai)

Write plugins in Rhai script:

```rhai
// my_plugin.rhai
const PLUGIN_NAME = "my_plugin";

fn init() {
    log("Plugin loaded!");
}

fn process(data) {
    data.to_upper()
}
```

Load from Rust:

```rust
#[cfg(feature = "plugin-rhai")]
let mut loader = RhaiPluginLoader::new();
let plugins = loader.load_all()?;

for plugin in plugins {
    plugin.init()?;
}
```

### Animations

Smooth transitions:

```rust
let mut tween = Tween::new(0.0_f32, 100.0, Duration::from_secs(1))
    .easing(EasingFunction::EaseInOutQuad);

// Update in render loop
tween.update(delta_time);
let value = tween.value(); // Interpolated value
```

---

## Plugin Development

### Creating a Plugin

```rust
struct MyPlugin {
    name: String,
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn required_capabilities(&self) -> Vec<Capability> {
        vec![Capability::CustomWidgets]
    }
    
    fn init(&mut self) -> Result<()> {
        // Initialize plugin
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<()> {
        // Cleanup
        Ok(())
    }
}
```

### Plugin Manager

```rust
let mut manager = PluginManager::new();
manager.register(Box::new(MyPlugin::new()))?;
manager.init_all()?;
```

---

## Performance

### Benchmarking

Run benchmarks:

```bash
cargo bench
```

### Profiling

Use criterion for detailed performance analysis:

```rust
cargo bench --bench render_benchmark
```

### Memory

Check for leaks:

```bash
valgrind --leak-check=full ./target/release/my_app
```

---

## Example Gallery

See `examples/` directory:

- `hello.rs` - Basic app
- `counter.rs` - Reactive state
- `dashboard.rs` - Layout
- `login.rs` - Forms & validation
- `file_browser.rs` - Lists & navigation
- `system_monitor.rs` - Real-time updates
- `data_table.rs` - Structured data
- `command_palette.rs` - Advanced UI

---

## Best Practices

1. **Use Signals for state** - Automatic UI updates
2. **Keep components small** - Single responsibility
3. **Return EventResult correctly** - Proper event propagation
4. **Test with examples** - Verify behavior
5. **Profile before optimizing** - Measure first
6. **Use async for I/O** - Keep UI responsive
7. **Handle errors gracefully** - User-friendly messages

---

## Getting Help

- **GitHub**: https://github.com/yourusername/rsdrav
- **Docs**: Run `cargo doc --open`
- **Examples**: `cargo run --example <name>`
- **Issues**: Report bugs on GitHub

---

**Happy Building! 🚀**
