//! Command help system
//!
//! Provides built-in help for registered commands.

use super::{Command, CommandRegistry};
use std::collections::HashMap;

/// Help information for a command
#[derive(Clone, Debug)]
pub struct CommandHelp {
    /// Command name
    pub name: String,
    /// Brief description
    pub description: String,
    /// Detailed usage
    pub usage: String,
    /// Examples
    pub examples: Vec<String>,
    /// Related commands
    pub see_also: Vec<String>,
}

impl CommandHelp {
    /// Create new command help
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            usage: String::new(),
            examples: Vec::new(),
            see_also: Vec::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set usage
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    /// Add an example
    pub fn example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Add a related command
    pub fn see_also(mut self, cmd: impl Into<String>) -> Self {
        self.see_also.push(cmd.into());
        self
    }

    /// Format as string
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Name and description
        output.push_str(&format!("Command: {}\n", self.name));
        if !self.description.is_empty() {
            output.push_str(&format!("\n{}\n", self.description));
        }

        // Usage
        if !self.usage.is_empty() {
            output.push_str(&format!("\nUsage:\n  {}\n", self.usage));
        }

        // Examples
        if !self.examples.is_empty() {
            output.push_str("\nExamples:\n");
            for ex in &self.examples {
                output.push_str(&format!("  {}\n", ex));
            }
        }

        // See also
        if !self.see_also.is_empty() {
            output.push_str(&format!("\nSee also: {}\n", self.see_also.join(", ")));
        }

        output
    }
}

/// Help system for managing command documentation
pub struct HelpSystem {
    help_texts: HashMap<String, CommandHelp>,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        let mut system = Self {
            help_texts: HashMap::new(),
        };

        // Register built-in help
        system.register(
            CommandHelp::new("help")
                .description("Show help for commands")
                .usage("help [command]")
                .example("help")
                .example("help quit")
        );

        system.register(
            CommandHelp::new("quit")
                .description("Exit the application")
                .usage("quit")
                .example("quit")
        );

        system
    }

    /// Register help for a command
    pub fn register(&mut self, help: CommandHelp) {
        self.help_texts.insert(help.name.clone(), help);
    }

    /// Get help for a command
    pub fn get(&self, command: &str) -> Option<&CommandHelp> {
        self.help_texts.get(command)
    }

    /// List all commands
    pub fn list_commands(&self) -> Vec<&str> {
        let mut commands: Vec<&str> = self.help_texts.keys().map(|s| s.as_str()).collect();
        commands.sort();
        commands
    }

    /// Generate help text for a command
    pub fn help_text(&self, command: Option<&str>) -> String {
        match command {
            Some(cmd) => {
                if let Some(help) = self.get(cmd) {
                    help.format()
                } else {
                    format!("No help available for '{}'\n\nTry 'help' to see all commands.", cmd)
                }
            }
            None => {
                // Show all commands
                let mut output = String::from("Available commands:\n\n");
                
                for cmd in self.list_commands() {
                    if let Some(help) = self.get(cmd) {
                        let desc = if help.description.is_empty() {
                            "(no description)"
                        } else {
                            &help.description
                        };
                        output.push_str(&format!("  {:12} - {}\n", cmd, desc));
                    }
                }
                
                output.push_str("\nUse 'help <command>' for more information.\n");
                output
            }
        }
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for CommandRegistry to add help support
pub trait CommandRegistryHelp {
    /// Get the help system
    fn help_system(&self) -> &HelpSystem;
    
    /// Get help system mutably
    fn help_system_mut(&mut self) -> &mut HelpSystem;
    
    /// Show help for a command
    fn show_help(&self, command: Option<&str>) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_help_creation() {
        let help = CommandHelp::new("test")
            .description("A test command")
            .usage("test <arg>")
            .example("test foo")
            .see_also("other");

        assert_eq!(help.name, "test");
        assert_eq!(help.description, "A test command");
        assert_eq!(help.examples.len(), 1);
        assert_eq!(help.see_also.len(), 1);
    }

    #[test]
    fn test_help_system() {
        let mut system = HelpSystem::new();
        
        system.register(
            CommandHelp::new("foo")
                .description("Foo command")
        );

        assert!(system.get("foo").is_some());
        assert!(system.get("bar").is_none());
    }

    #[test]
    fn test_help_text_formatting() {
        let help = CommandHelp::new("example")
            .description("Example command")
            .usage("example [options]")
            .example("example --verbose");

        let formatted = help.format();
        assert!(formatted.contains("Command: example"));
        assert!(formatted.contains("Example command"));
        assert!(formatted.contains("Usage:"));
        assert!(formatted.contains("Examples:"));
    }

    #[test]
    fn test_list_all_commands() {
        let system = HelpSystem::new();
        let help_text = system.help_text(None);
        
        assert!(help_text.contains("Available commands"));
        assert!(help_text.contains("help"));
        assert!(help_text.contains("quit"));
    }

    #[test]
    fn test_specific_command_help() {
        let system = HelpSystem::new();
        let help_text = system.help_text(Some("help"));
        
        assert!(help_text.contains("Command: help"));
        assert!(help_text.contains("Show help for commands"));
    }
}
