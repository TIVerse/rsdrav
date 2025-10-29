//! Visual regression tests using insta snapshots
//!
//! These tests capture the rendered output of widgets and compare
//! against saved snapshots to detect visual regressions.

use insta::assert_snapshot;
use rsdrav::prelude::*;

/// Helper to render a component to a string
fn render_to_string(component: impl Component, width: u16, height: u16) -> String {
    let mut buffer = Buffer::new(width, height);
    let store = Store::new();
    let area = Rect::new(0, 0, width, height);
    let ctx = RenderContext::new(&mut buffer, area, &store);

    let node = component.render(&ctx);

    // Convert ViewNode to string representation
    format!("{:?}", node)
}

#[test]
fn test_text_widget_visual() {
    let text = Text::new("Hello, World!").fg(Color::GREEN);
    let output = render_to_string(text, 80, 24);

    assert_snapshot!(output);
}

#[test]
fn test_vstack_visual() {
    let stack = VStack::new()
        .push(Text::new("Line 1"))
        .push(Text::new("Line 2"))
        .push(Text::new("Line 3"));

    let output = render_to_string(stack, 80, 24);
    assert_snapshot!(output);
}

#[test]
fn test_panel_visual() {
    let panel = Panel::new()
        .title("Test Panel")
        .child(Text::new("Content inside panel"));

    let output = render_to_string(panel, 40, 10);
    assert_snapshot!(output);
}

#[test]
fn test_input_widget_visual() {
    let value = Signal::new("test input".to_string());
    let input = Input::new(value).placeholder("Enter text").focused(true);

    let output = render_to_string(input, 50, 3);
    assert_snapshot!(output);
}

#[test]
fn test_progress_bar_visual() {
    let progress = Signal::new(0.65);
    let bar = ProgressBar::new(progress)
        .label("Loading")
        .width(30)
        .show_percentage(true);

    let output = render_to_string(bar, 50, 3);
    assert_snapshot!(output);
}

#[test]
fn test_list_widget_visual() {
    let items = Signal::new(vec!["Item 1", "Item 2", "Item 3", "Item 4"]);
    let selected = Signal::new(Some(1));

    let list = List::new(items, selected).visible_height(5);

    let output = render_to_string(list, 40, 10);
    assert_snapshot!(output);
}

#[test]
fn test_table_widget_visual() {
    #[derive(Clone)]
    struct Row {
        name: String,
        value: i32,
    }

    let data = Signal::new(vec![
        Row {
            name: "Alpha".into(),
            value: 10,
        },
        Row {
            name: "Beta".into(),
            value: 20,
        },
        Row {
            name: "Gamma".into(),
            value: 30,
        },
    ]);
    let selected = Signal::new(Some(0));

    let table = Table::new(data, selected)
        .column(TableColumn::new("Name", 15).render(|r: &Row| r.name.clone()))
        .column(TableColumn::new("Value", 10).render(|r: &Row| r.value.to_string()))
        .visible_height(5);

    let output = render_to_string(table, 40, 10);
    assert_snapshot!(output);
}

#[test]
fn test_modal_widget_visual() {
    let visible = Signal::new(true);
    let modal = Modal::new(visible)
        .title("Confirmation")
        .child(Text::new("Are you sure?"));

    let output = render_to_string(modal, 50, 10);
    assert_snapshot!(output);
}

#[test]
fn test_complex_layout_visual() {
    let layout = VStack::new()
        .gap(1)
        .push(
            Text::new("=== Header ===")
                .fg(Color::YELLOW)
                .add_modifier(Modifier::BOLD),
        )
        .push(
            Panel::new().title("Main").child(
                HStack::new()
                    .push(Text::new("Left"))
                    .push(Text::new("Right")),
            ),
        )
        .push(Text::new("Footer").fg(Color::GRAY));

    let output = render_to_string(layout, 60, 15);
    assert_snapshot!(output);
}

#[test]
fn test_reactive_text_visual() {
    let count = Signal::new(42);
    let text = Text::bind({
        let c = count.clone();
        move || format!("Count: {}", c.get())
    });

    let output = render_to_string(text, 30, 3);
    assert_snapshot!(output);

    // Change value and test again
    count.set(100);
    let text2 = Text::bind({
        let c = count.clone();
        move || format!("Count: {}", c.get())
    });

    let output2 = render_to_string(text2, 30, 3);
    assert_snapshot!(output2);
}
