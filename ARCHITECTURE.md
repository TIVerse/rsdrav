# rsdrav Architecture

## Overview

rsdrav is a reactive terminal UI framework built with a component-based architecture and integrated state management.

## System Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Application                        │
│  (User Components, Business Logic)                  │
└────────────────┬────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────┐
│              Component System                        │
│  - Component Trait                                   │
│  - Lifecycle (mount/update/render/unmount)          │
│  - Event Handling                                    │
└────────────────┬────────────────────────────────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼────┐  ┌───▼────┐  ┌───▼────┐
│ State  │  │ Layout │  │ Events │
│ Signal │  │  Flex  │  │  Keys  │
│Derived │  │  Row   │  │ Mouse  │
│ Store  │  │Column  │  │ Focus  │
└───┬────┘  └───┬────┘  └───┬────┘
    │           │            │
    └───────────┼────────────┘
                │
┌───────────────▼────────────────────────────────────┐
│             Render Pipeline                         │
│  ViewNode → Buffer → Diff → Backend                │
└─────────────────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────┐
│              Terminal                                │
│  (Crossterm / Termion)                              │
└─────────────────────────────────────────────────────┘
```

## Core Modules

### 1. Render System

**Location**: `src/render/`

**Purpose**: Efficient terminal rendering with minimal writes

**Components**:
- `Buffer` - Virtual grid of cells
- `Cell` - Single character with style
- `Diff` - Line-based change detection
- `Backend` - Terminal abstraction

**Algorithm**: Line hashing for O(n) diff

```rust
// Pseudo-code
fn render_frame(old_buffer, new_buffer) {
    for line in 0..height {
        if hash(old_buffer[line]) != hash(new_buffer[line]) {
            // Find exact changes
            let changes = precise_diff(old_buffer[line], new_buffer[line]);
            // Write only changed regions
            backend.write_changes(changes);
        }
    }
}
```

### 2. Reactive State

**Location**: `src/state/`

**Purpose**: Automatic UI updates on state changes

**Types**:

**Signal<T>**:
```rust
struct Signal<T> {
    value: Arc<RwLock<T>>,
    version: Arc<AtomicU64>,
    subscribers: Vec<Subscription>,
}
```
- Mutable reactive value
- Version tracking for change detection
- Subscription system for notifications

**Derived<T>**:
```rust
struct Derived<T> {
    compute: Box<dyn Fn() -> T>,
    cache: Option<T>,
    dirty: bool,
}
```
- Computed values
- Lazy evaluation with caching
- Dependency tracking

**Store**:
```rust
struct Store {
    data: HashMap<String, Box<dyn Any>>,
}
```
- Type-safe global state container
- Runtime type checking

### 3. Layout System

**Location**: `src/layout/`

**Types**:

**Rect**: Basic rectangle primitive

**Containers**:
- `Row` - Horizontal distribution
- `Column` - Vertical distribution
- `Stack` - Layered composition

**Flex**: CSS Flexbox-inspired layout
```rust
struct Flex {
    direction: FlexDirection,
    items: Vec<FlexItem>,
}

struct FlexItem {
    grow: f32,    // Flex grow factor
    shrink: f32,  // Flex shrink factor
    basis: Length, // Base size
    min/max: Option<u16>, // Constraints
}
```

Algorithm:
1. Calculate base sizes from `basis`
2. Distribute remaining space by `grow` factor
3. Shrink if overflow by `shrink` factor
4. Apply min/max constraints
5. Convert to rectangles

### 4. Event System

**Location**: `src/event/`

**Flow**:
```
Terminal Input → Backend → Event → App → FocusManager → Component
```

**Types**:
- `KeyEvent` - Keyboard input
- `MouseEvent` - Mouse actions
- `Event` - Unified event type

**Propagation**:
1. Focused component receives event first
2. If `Ignored`, bubble to parent
3. If `Handled`, stop propagation
4. If `Consumed`, terminate app (e.g., quit)

### 5. Focus Management

**Location**: `src/focus.rs`

**Purpose**: Track and navigate focusable components

```rust
struct FocusManager {
    components: Vec<(ComponentId, usize, bool)>, // id, order, focusable
    current: Option<ComponentId>,
}
```

**Navigation**:
- Tab: Focus next
- Shift+Tab: Focus previous
- Automatic skip non-focusable
- Respects tab order

### 6. Command System

**Location**: `src/command/`

**Purpose**: Shell-like command execution

**Components**:
- `Parser` - Parse command strings
- `Registry` - Command handlers
- `Completer` - Tab completion
- `UndoStack` - Undo/redo

**Example**:
```
Input: `set theme "dark mode"`
Parse: Command { name: "set", args: ["theme", "dark mode"] }
Execute: handler.execute(&context, &args)
```

### 7. Plugin System

**Location**: `src/plugin/`

**Architecture**: Capability-based security

```rust
trait Plugin {
    fn name(&self) -> &str;
    fn required_capabilities(&self) -> Vec<Capability>;
    fn init(&mut self) -> Result<()>;
}
```

**Capabilities**:
- `FileRead` - Read filesystem
- `FileWrite` - Write filesystem
- `Network` - Network access
- `CustomWidgets` - Register widgets
- `RegisterCommands` - Add commands

**Loaders**:
- `RhaiLoader` - Rhai scripts
- `DylibLoader` - Native dynamic libraries (future)
- `WasmLoader` - WebAssembly (future)

### 8. Animation System

**Location**: `src/animation/`

**Types**:

**Tween**:
```rust
struct Tween<T> {
    start: T,
    end: T,
    duration: Duration,
    elapsed: Duration,
    easing: EasingFunction,
}
```

**Easing Functions**:
- Linear
- Quad (In/Out/InOut)
- Cubic
- Sine

**Timeline**: Sequential animation composition

### 9. Async Support

**Location**: `src/async_support/`

**Purpose**: Background tasks without blocking UI

```rust
#[cfg(feature = "tokio")]
struct AsyncRuntime {
    runtime: tokio::Runtime,
}
```

**Features**:
- Spawn async tasks
- Timeout support
- Sync/async bridge

## Component Lifecycle

```rust
Component Created
    │
    ▼
mount() ◄──────────┐
    │              │
    ▼              │
render() ◄────┐    │
    │         │    │
    ▼         │    │
handle_event()│    │
    │         │    │
    ├─ State ─┘    │
    │              │
    ▼              │
update() ──────────┘
    │
    ▼
unmount()
    │
    ▼
Destroyed
```

## Performance Optimizations

### 1. Diff-Based Rendering

Only write changed cells to terminal:
- Line hashing for quick comparison
- Precise diff for exact changes
- Batched writes

**Complexity**: O(n) where n = screen size

### 2. Signal Versioning

Avoid redundant renders:
```rust
if signal.version() == last_seen_version {
    return cached_result;
}
```

### 3. Derived Caching

Compute once, use many:
```rust
if !derived.is_dirty() {
    return cached_value;
}
```

### 4. Layout Caching

Cache layout calculations when constraints unchanged

### 5. Event Batching

Group multiple events before render

## Memory Management

### Reference Counting

`Signal<T>` uses `Arc<RwLock<T>>` for shared ownership

### Subscription Cleanup

Weak references prevent memory leaks:
```rust
struct Subscription {
    callback: Weak<dyn Fn(&T)>,
}
```

Auto-cleanup when component destroyed

### Buffer Reuse

Swap buffers instead of allocating:
```rust
std::mem::swap(&mut old_buffer, &mut new_buffer);
```

## Thread Safety

### Signals

Thread-safe via `Arc<RwLock<T>>`

### Event Loop

Single-threaded event processing

### Async Tasks

Tokio runtime for concurrent operations

## Testing Strategy

### Unit Tests

Each module has `#[cfg(test)] mod tests`

### Integration Tests

`tests/integration_tests.rs` - End-to-end

### Property Tests

`proptest` for fuzzing (partial)

### Benchmarks

`criterion` for performance regression

## Error Handling

### Error Types

```rust
pub enum Error {
    Io(std::io::Error),
    Parse(String),
    Plugin(PluginError),
    // ...
}
```

### Result Type

`pub type Result<T> = std::result::Result<T, Error>;`

### Panic Safety

No panics in library code - use `Result`

## Build Features

```toml
[features]
default = ["crossterm"]
crossterm = ["dep:crossterm"]
termion = ["dep:termion"]
tokio = ["dep:tokio"]
plugin-rhai = ["dep:rhai"]
plugin-dylib = ["dep:libloading"]
```

Enables optional functionality without bloat

## Future Architecture

### Planned Additions

1. **Virtual Scrolling** - Large lists
2. **Incremental Rendering** - Partial updates
3. **GPU Acceleration** - Sixel/Kitty graphics
4. **Web Backend** - WASM compilation
5. **Hot Reload** - Live component updates

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup

## References

- Spec: [definition.md](definition.md)
- Guide: [GUIDE.md](GUIDE.md)
- README: [README.md](README.md)
