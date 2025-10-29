use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Event error: {0}")]
    Event(String),

    #[error("Command error: {0}")]
    Command(#[from] CommandError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    #[error("State error: {0}")]
    State(String),

    #[error("Layout error: {0}")]
    Layout(String),
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Command not found: {0}")]
    NotFound(String),

    #[error("Invalid args: {0}")]
    InvalidArgs(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Unclosed quote in command")]
    UnclosedQuote,

    #[error("Empty command")]
    Empty,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Plugin timeout")]
    Timeout,

    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
}
