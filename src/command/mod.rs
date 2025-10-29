//! Command engine with parser, registry, completion, and undo/redo
//!
//! The command system is first-class in rsdrav - provides:
//! - Shell-like command parsing with quotes and escapes
//! - Type-safe command handlers with context
//! - Tab completion for commands and arguments
//! - Built-in undo/redo with state snapshots

mod complete;
mod handler;
mod undo;

pub use complete::{Completer, CompletionItem};
pub use handler::{CommandContext, CommandHandler, CommandResult};
pub use undo::{UndoAction, UndoStack};

use crate::error::{CommandError, Result};
use std::collections::HashMap;

/// Parsed command with name and arguments
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

/// Parse a command line into a Command
///
/// Supports shell-like syntax:
/// - `command arg1 arg2` - simple args
/// - `command "quoted arg"` - quoted args with spaces
/// - `command 'single quotes'` - single quotes
/// - `command arg\ with\ escape` - escaped spaces
///
/// TODO: could add piping, redirection, etc. later if needed
pub fn parse(input: &str) -> Result<Command> {
    let input = input.trim();

    if input.is_empty() {
        return Err(CommandError::Empty.into());
    }

    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote: Option<char> = None;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
            }
            '"' | '\'' => {
                if let Some(quote_char) = in_quote {
                    if quote_char == ch {
                        // End quote
                        in_quote = None;
                    } else {
                        // Different quote char inside quotes
                        current.push(ch);
                    }
                } else {
                    // Start quote
                    in_quote = Some(ch);
                }
            }
            ' ' | '\t' => {
                if in_quote.is_some() {
                    // Space inside quotes
                    current.push(ch);
                } else if !current.is_empty() {
                    // End of token
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Check for unclosed quote
    if in_quote.is_some() {
        return Err(CommandError::UnclosedQuote.into());
    }

    // Push last token
    if !current.is_empty() {
        tokens.push(current);
    }

    if tokens.is_empty() {
        return Err(CommandError::Empty.into());
    }

    let name = tokens[0].clone();
    let args = tokens[1..].to_vec();

    Ok(Command { name, args })
}

/// Command registry - maps command names to handlers
pub struct CommandRegistry {
    handlers: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register<H: CommandHandler + 'static>(&mut self, name: &str, handler: H) {
        self.handlers.insert(name.to_string(), Box::new(handler));
    }

    /// Execute a command by name
    pub fn execute(&mut self, cmd: Command, ctx: &mut CommandContext) -> Result<CommandResult> {
        let handler = self
            .handlers
            .get_mut(&cmd.name)
            .ok_or_else(|| CommandError::NotFound(cmd.name.clone()))?;

        handler.execute(cmd, ctx)
    }

    /// Execute a command from a string
    pub fn execute_line(&mut self, line: &str, ctx: &mut CommandContext) -> Result<CommandResult> {
        let cmd = parse(line)?;
        self.execute(cmd, ctx)
    }

    /// Get list of registered command names
    pub fn command_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Check if a command exists
    pub fn has_command(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let cmd = parse("echo hello world").unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello", "world"]);
    }

    #[test]
    fn test_parse_quoted() {
        let cmd = parse(r#"echo "hello world""#).unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_single_quotes() {
        let cmd = parse("echo 'hello world'").unwrap();
        assert_eq!(cmd.name, "echo");
        assert_eq!(cmd.args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_mixed_quotes() {
        let cmd = parse(r#"cmd "arg 1" arg2 'arg 3'"#).unwrap();
        assert_eq!(cmd.name, "cmd");
        assert_eq!(cmd.args, vec!["arg 1", "arg2", "arg 3"]);
    }

    #[test]
    fn test_parse_escaped() {
        let cmd = parse(r"echo hello\ world").unwrap();
        assert_eq!(cmd.args, vec!["hello world"]);
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn test_parse_unclosed_quote() {
        assert!(parse(r#"echo "unclosed"#).is_err());
    }

    #[test]
    fn test_command_builder() {
        let cmd = Command::new("test").arg("arg1").arg("arg2");

        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.args, vec!["arg1", "arg2"]);
    }
}
