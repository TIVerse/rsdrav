#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::type_complexity)]
#![allow(clippy::single_match)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::manual_unwrap_or_default)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::arc_with_non_send_sync)]

//! # rsdrav - Reactive Terminal UI Framework
//!
//! A next-generation TUI framework for Rust with integrated:
//! - Rendering (buffer, diff, backends)
//! - Reactive state (Signal, Derived, Store)
//! - Command engine with completion and undo
//! - Theme system with transitions
//! - Plugin system with capabilities
//!
//! ## Quick Start
//!
//! ```no_run
//! use rsdrav::prelude::*;
//!
//! fn main() -> rsdrav::Result<()> {
//!     let app = App::new()?;
//!     app.run()
//! }
//! ```

// Core modules
pub mod app;
pub mod error;
pub mod event;
pub mod render;
pub mod state;
pub mod theme;

pub mod animation;
pub mod command;
pub mod event_router;
pub mod focus;
pub mod layout;
pub mod plugin;
pub mod view;

#[cfg(feature = "tokio")]
pub mod async_support;

// Re-exports for convenience
pub use app::App;
pub use error::{Error, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::animation::{Animatable, EasingFunction, Timeline, Tween};
    pub use crate::app::App;
    pub use crate::command::{
        Command, CommandContext, CommandHandler, CommandRegistry, CommandResult,
    };
    pub use crate::error::{Error, Result};
    pub use crate::event::{
        Event, EventResult, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind,
    };
    pub use crate::event_router::{EventHandler, EventPhase, EventRouter, EventRoutingContext};
    pub use crate::focus::{ComponentId, FocusManager};
    pub use crate::layout::{
        Align, Column, Flex, FlexDirection, FlexItem, Justify, Length, Rect, Row, Stack,
    };
    pub use crate::plugin::{Capability, Plugin, PluginManager};
    pub use crate::render::{Buffer, Cell};
    pub use crate::state::{Derived, Signal, Store};
    pub use crate::theme::{Color, Modifier, Style};
    pub use crate::view::{
        Button, HStack, Input, List, Modal, Panel, ProgressBar, Scrollable, SortOrder, Table,
        TableColumn, Tabs, Text, VStack,
    };
    pub use crate::view::{
        Component, EventContext, MountContext, RenderContext, UpdateContext, ViewNode,
    };

    #[cfg(feature = "tokio")]
    pub use crate::async_support::{spawn_task, with_timeout, AsyncRuntime, AsyncTask};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buf = render::Buffer::new(80, 24);
        assert_eq!(buf.width, 80);
        assert_eq!(buf.height, 24);
    }
}
