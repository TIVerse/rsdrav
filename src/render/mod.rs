// Rendering core - buffer, backend abstraction, diff, and rendering

mod backend;
mod buffer;
mod diff;
mod renderer;

pub use backend::Backend;
pub use buffer::{Buffer, Cell};
pub use diff::{compute_diff, compute_diff_precise, DirtyRegion};
pub use renderer::Renderer;

#[cfg(feature = "crossterm")]
pub use backend::CrosstermBackend;
