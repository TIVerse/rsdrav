// Demo showcasing rsdrav features
//
// Shows: reactive state, layout system, command execution, efficient rendering
// Run with: cargo run --example demo

use rsdrav::prelude::*;

fn main() -> rsdrav::Result<()> {
    // Create app with reactive state
    let store = Store::new();

    // Create some reactive signals
    let count = store.get_or_create("count", 0);
    let message = store.get_or_create("message", "Welcome to rsdrav!".to_string());

    // Set up command registry
    let mut registry = CommandRegistry::new();
    registry.register("increment", IncrementCommand);
    registry.register("decrement", DecrementCommand);
    registry.register("message", SetMessageCommand);

    println!("rsdrav demo - framework capabilities showcase");
    println!("----------------------------------------------");
    println!("✓ Reactive state management (Signal/Derived/Store)");
    println!("✓ Layout system (Row/Column/Stack containers)");
    println!("✓ Command engine with parser and handlers");
    println!("✓ Efficient diff-based rendering");
    println!("✓ ANSI color and style support");
    println!("");
    println!("Current count: {}", count.get());
    println!("Current message: {}", message.get());
    println!("");
    println!("Run the app with: cargo run --example hello");

    Ok(())
}

// Example command handlers

struct IncrementCommand;

impl CommandHandler for IncrementCommand {
    fn execute(
        &mut self,
        _cmd: Command,
        ctx: &mut CommandContext,
    ) -> rsdrav::Result<CommandResult> {
        let count: Signal<i32> = ctx.store.get_or_create("count", 0);
        count.update(|val| *val += 1);
        Ok(CommandResult::success_with_message(format!("Count: {}", count.get())).with_redraw())
    }

    fn description(&self) -> &str {
        "Increment the counter"
    }
}

struct DecrementCommand;

impl CommandHandler for DecrementCommand {
    fn execute(
        &mut self,
        _cmd: Command,
        ctx: &mut CommandContext,
    ) -> rsdrav::Result<CommandResult> {
        let count: Signal<i32> = ctx.store.get_or_create("count", 0);
        count.update(|val| *val -= 1);
        Ok(CommandResult::success_with_message(format!("Count: {}", count.get())).with_redraw())
    }

    fn description(&self) -> &str {
        "Decrement the counter"
    }
}

struct SetMessageCommand;

impl CommandHandler for SetMessageCommand {
    fn execute(&mut self, cmd: Command, ctx: &mut CommandContext) -> rsdrav::Result<CommandResult> {
        if cmd.args.is_empty() {
            return Ok(CommandResult::success_with_message("Usage: message <text>"));
        }

        let new_message = cmd.args.join(" ");
        let message: Signal<String> = ctx.store.get_or_create("message", String::new());
        message.set(new_message.clone());

        Ok(
            CommandResult::success_with_message(format!("Message set to: {}", new_message))
                .with_redraw(),
        )
    }

    fn description(&self) -> &str {
        "Set the message text"
    }
}
