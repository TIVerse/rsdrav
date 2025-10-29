// Simple hello world example using the component system
// Shows a welcome message with styled text
// Press 'q' to quit

use rsdrav::prelude::*;

fn main() -> rsdrav::Result<()> {
    let app = App::new()?.root(HelloWorld).run()?;

    Ok(())
}

/// Simple hello world component
struct HelloWorld;

impl Component for HelloWorld {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        VStack::new()
            .push(
                Text::new("Welcome to rsdrav!")
                    .fg(Color::GREEN)
                    .add_modifier(Modifier::BOLD),
            )
            .push(Text::new(""))
            .push(Text::new("A reactive TUI framework for Rust").fg(Color::CYAN))
            .push(Text::new(""))
            .push(Text::new("Press 'q' to quit").fg(Color::GRAY))
            .render(_ctx)
    }
}
