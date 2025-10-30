//! Integration tests for rsdrav
//!
//! Tests end-to-end functionality of the framework.

use rsdrav::command::parse;
use rsdrav::prelude::*;

#[test]
fn test_app_creation() {
    let result = App::new();
    assert!(result.is_ok(), "App creation should succeed");
}

#[test]
fn test_signal_reactivity() {
    let sig = Signal::new(0);
    assert_eq!(sig.get(), 0);

    sig.set(42);
    assert_eq!(sig.get(), 42);

    sig.update(|v| *v += 1);
    assert_eq!(sig.get(), 43);
}

#[test]
fn test_derived_computation() {
    let a = Signal::new(2);
    let b = Signal::new(3);

    let sum = {
        let a = a.clone();
        let b = b.clone();
        Derived::new(move || a.get() + b.get())
    };

    assert_eq!(sum.get(), 5);

    a.set(10);
    sum.invalidate();
    assert_eq!(sum.get(), 13);
}

#[test]
fn test_store_operations() {
    let store = Store::new();

    // Set a value using the store
    store.set("count", 0);

    assert!(store.contains("count"));

    let retrieved: Option<Signal<i32>> = store.get("count");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().get(), 0);
}

#[test]
fn test_component_lifecycle() {
    struct TestComponent {
        mounted: Signal<bool>,
    }

    impl TestComponent {
        fn new() -> Self {
            Self {
                mounted: Signal::new(false),
            }
        }
    }

    impl Component for TestComponent {
        fn render(&self, _ctx: &RenderContext) -> ViewNode {
            ViewNode::text("Test")
        }

        fn mount(&mut self, _ctx: &mut MountContext) {
            self.mounted.set(true);
        }

        fn unmount(&mut self, _ctx: &mut MountContext) {
            self.mounted.set(false);
        }
    }

    let mut comp = TestComponent::new();
    assert!(!comp.mounted.get());

    let mut store = Store::new();
    let mut ctx = MountContext { store: &mut store };

    comp.mount(&mut ctx);
    assert!(comp.mounted.get());

    comp.unmount(&mut ctx);
    assert!(!comp.mounted.get());
}

#[test]
fn test_focus_manager() {
    let mut focus = FocusManager::new();

    let id1 = ComponentId::new(1);
    let id2 = ComponentId::new(2);
    let id3 = ComponentId::new(3);

    focus.register(id1, 0, true);
    focus.register(id2, 1, true);
    focus.register(id3, 2, true);

    assert_eq!(focus.current(), Some(id1));

    focus.focus_next();
    assert_eq!(focus.current(), Some(id2));

    focus.focus_next();
    assert_eq!(focus.current(), Some(id3));

    focus.focus_prev();
    assert_eq!(focus.current(), Some(id2));
}

#[test]
fn test_widget_composition() {
    let stack = VStack::new()
        .push(Text::new("Header"))
        .push(Text::new("Body"))
        .push(Text::new("Footer"));

    let mut buffer = Buffer::new(80, 24);
    let store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let ctx = RenderContext::new(&mut buffer, area, &store);

    let node = stack.render(&ctx);

    match node {
        ViewNode::Container { children, .. } => {
            assert_eq!(children.len(), 3);
        }
        _ => panic!("Expected container node"),
    }
}

#[test]
fn test_input_widget() {
    let value = Signal::new(String::new());
    let mut input = Input::new(value.clone()).focused(true);

    let mut store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let mut ctx = EventContext {
        store: &mut store,
        area,
    };

    let event = Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
    let result = input.handle_event(&event, &mut ctx);

    assert_eq!(result, EventResult::Handled);
    assert_eq!(value.get(), "a");
}

#[test]
fn test_list_navigation() {
    let items = Signal::new(vec![1, 2, 3, 4, 5]);
    let selected = Signal::new(Some(0));

    let mut list = List::new(items, selected.clone());

    let mut store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let mut ctx = EventContext {
        store: &mut store,
        area,
    };

    let down = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
    list.handle_event(&down, &mut ctx);

    assert_eq!(selected.get(), Some(1));
}

#[test]
fn test_table_rendering() {
    #[derive(Clone)]
    struct Row {
        name: String,
        value: i32,
    }

    let data = Signal::new(vec![
        Row {
            name: "A".into(),
            value: 1,
        },
        Row {
            name: "B".into(),
            value: 2,
        },
    ]);
    let selected = Signal::new(Some(0));

    let table = Table::new(data, selected)
        .column(TableColumn::new("Name", 10).render(|r: &Row| r.name.clone()))
        .column(TableColumn::new("Value", 10).render(|r: &Row| r.value.to_string()));

    let mut buffer = Buffer::new(80, 24);
    let store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let ctx = RenderContext::new(&mut buffer, area, &store);

    let _node = table.render(&ctx);
    // Rendering should not panic
}

#[test]
fn test_progress_bar() {
    let progress = Signal::new(0.5);
    let bar = ProgressBar::new(progress).width(20);

    let mut buffer = Buffer::new(80, 24);
    let store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let ctx = RenderContext::new(&mut buffer, area, &store);

    let _node = bar.render(&ctx);
    // Rendering should not panic
}

#[test]
fn test_tabs_switching() {
    let selected = Signal::new(0);
    let mut tabs = Tabs::new(selected.clone())
        .tab("Tab1", Text::new("Content 1"))
        .tab("Tab2", Text::new("Content 2"))
        .tab("Tab3", Text::new("Content 3"));

    let mut store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let mut ctx = EventContext {
        store: &mut store,
        area,
    };

    let tab_event = Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));
    tabs.handle_event(&tab_event, &mut ctx);

    assert_eq!(selected.get(), 1);
}

#[test]
fn test_modal_visibility() {
    let visible = Signal::new(true);
    let modal = Modal::new(visible.clone())
        .title("Test")
        .child(Text::new("Content"));

    let mut buffer = Buffer::new(80, 24);
    let store = Store::new();
    let area = Rect::new(0, 0, 80, 24);
    let ctx = RenderContext::new(&mut buffer, area, &store);

    let _node = modal.render(&ctx);

    // Should be visible
    assert!(visible.get());
}

#[test]
fn test_command_parsing() {
    let result = parse("test arg1 arg2");
    assert!(result.is_ok());

    let cmd = result.unwrap();
    assert_eq!(cmd.name, "test");
    assert_eq!(cmd.args, vec!["arg1", "arg2"]);
}

#[test]
fn test_layout_calculations() {
    let rect = Rect::new(0, 0, 100, 50);

    let inner = rect.inner(2);
    assert_eq!(inner.width, 96);
    assert_eq!(inner.height, 46);

    let (left, right) = rect.split_h(50);
    assert_eq!(left.width, 50);
    assert_eq!(right.width, 50);
}

#[test]
fn test_buffer_resize() {
    let mut buf = Buffer::new(80, 24);
    assert_eq!(buf.width, 80);
    assert_eq!(buf.height, 24);

    buf.resize(120, 40);
    assert_eq!(buf.width, 120);
    assert_eq!(buf.height, 40);
}

#[test]
fn test_error_types() {
    use rsdrav::error::*;

    let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
    let err: Error = io_err.into();

    assert!(matches!(err, Error::Io(_)));
}
