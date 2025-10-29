use super::Command;
use crate::error::Result;
use crate::plugin::PluginManager;
use crate::state::Store;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::any::Any;
use std::sync::{Arc, Mutex};

/// Event bus for pub-sub messaging between commands
#[derive(Clone)]
pub struct EventBus {
    sender: Sender<(String, Vec<u8>)>,
    receiver: Arc<Mutex<Receiver<(String, Vec<u8>)>>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Publish an event
    pub fn publish(&self, event_name: impl Into<String>, data: Vec<u8>) {
        let _ = self.sender.send((event_name.into(), data));
    }

    /// Try to receive an event (non-blocking)
    pub fn try_recv(&self) -> Option<(String, Vec<u8>)> {
        self.receiver.lock().unwrap().try_recv().ok()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Context passed to command handlers
///
/// Provides access to app state and other resources.
pub struct CommandContext {
    pub store: Store,
    pub event_bus: EventBus,
    pub plugin_manager: Arc<Mutex<PluginManager>>,
}

impl CommandContext {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            event_bus: EventBus::new(),
            plugin_manager: Arc::new(Mutex::new(PluginManager::new())),
        }
    }

    pub fn with_plugin_manager(mut self, plugin_manager: Arc<Mutex<PluginManager>>) -> Self {
        self.plugin_manager = plugin_manager;
        self
    }
}

/// Result of command execution
///
/// Commands can return data, request UI updates, provide undo info, etc.
pub struct CommandResult {
    /// Optional success message
    pub message: Option<String>,

    /// Whether to mark UI as needing redraw
    pub needs_redraw: bool,

    /// Optional undo data for this command
    /// Stored as type-erased Any so different commands can use different types
    pub undo_data: Option<Box<dyn Any + Send + Sync>>,
}

impl CommandResult {
    pub fn success() -> Self {
        Self {
            message: None,
            needs_redraw: false,
            undo_data: None,
        }
    }

    pub fn success_with_message(msg: impl Into<String>) -> Self {
        Self {
            message: Some(msg.into()),
            needs_redraw: false,
            undo_data: None,
        }
    }

    pub fn with_redraw(mut self) -> Self {
        self.needs_redraw = true;
        self
    }

    pub fn with_undo(mut self, data: impl Any + Send + Sync + 'static) -> Self {
        self.undo_data = Some(Box::new(data));
        self
    }
}

/// Trait for command handlers
///
/// Implement this to define custom commands.
pub trait CommandHandler: Send + Sync {
    /// Execute the command
    fn execute(&mut self, cmd: Command, ctx: &mut CommandContext) -> Result<CommandResult>;

    /// Get command description for help
    fn description(&self) -> &str {
        "No description available"
    }

    /// Get usage string
    fn usage(&self) -> &str {
        ""
    }
}

// Example: Echo command handler
pub struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn execute(&mut self, cmd: Command, _ctx: &mut CommandContext) -> Result<CommandResult> {
        let message = if cmd.args.is_empty() {
            String::new()
        } else {
            cmd.args.join(" ")
        };

        Ok(CommandResult::success_with_message(message))
    }

    fn description(&self) -> &str {
        "Print arguments to output"
    }

    fn usage(&self) -> &str {
        "echo [args...]"
    }
}

// Example: Quit command handler
pub struct QuitHandler;

impl CommandHandler for QuitHandler {
    fn execute(&mut self, _cmd: Command, ctx: &mut CommandContext) -> Result<CommandResult> {
        // Signal quit by setting a flag in the store
        ctx.store.set("app:should_quit", true);
        Ok(CommandResult::success().with_redraw())
    }

    fn description(&self) -> &str {
        "Exit the application"
    }

    fn usage(&self) -> &str {
        "quit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Store;

    #[test]
    fn test_echo_handler() {
        let mut handler = EchoHandler;
        let mut ctx = CommandContext::new(Store::new());
        let cmd = Command::new("echo").arg("hello").arg("world");

        let result = handler.execute(cmd, &mut ctx).unwrap();
        assert_eq!(result.message, Some("hello world".to_string()));
    }

    #[test]
    fn test_quit_handler() {
        let mut handler = QuitHandler;
        let mut ctx = CommandContext::new(Store::new());
        let cmd = Command::new("quit");

        handler.execute(cmd, &mut ctx).unwrap();

        let should_quit: bool = ctx.store.get("app:should_quit").unwrap().get();
        assert!(should_quit);
    }
}
