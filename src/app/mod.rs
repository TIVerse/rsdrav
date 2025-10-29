use crate::animation::Timeline;
use crate::error::Result;
use crate::event::{Event, KeyCode, KeyModifiers};
use crate::focus::FocusManager;
use crate::layout::Rect;
use crate::render::{Backend, Buffer, Renderer};
use crate::state::Store;
use crate::view::{Component, EventContext, MountContext, RenderContext, UpdateContext};
use std::time::{Duration, Instant};

#[cfg(feature = "tokio")]
use crate::async_support::AsyncRuntime;

pub struct App {
    backend: Box<dyn Backend>,
    buffer: Buffer,
    prev_buffer: Buffer,
    renderer: Renderer,
    should_quit: bool,
    root: Option<Box<dyn Component>>,
    store: Store,
    focus: FocusManager,
    timeline: Timeline,
    last_tick: Instant,
    #[cfg(feature = "tokio")]
    async_runtime: Option<AsyncRuntime>,
}

impl App {
    pub fn new() -> Result<Self> {
        // Default to crossterm if available, otherwise termion
        #[cfg(all(feature = "crossterm", not(feature = "termion")))]
        let backend: Box<dyn Backend> = Box::new(crate::render::CrosstermBackend::new()?);

        #[cfg(all(feature = "termion", not(feature = "crossterm")))]
        let backend: Box<dyn Backend> = Box::new(crate::render::TermionBackend::new()?);

        #[cfg(all(feature = "crossterm", feature = "termion"))]
        let backend: Box<dyn Backend> = Box::new(crate::render::CrosstermBackend::new()?); // Prefer crossterm

        #[cfg(not(any(feature = "crossterm", feature = "termion")))]
        compile_error!("No backend feature enabled! Enable 'crossterm' or 'termion'");

        // Start with a default size, will resize on first frame
        let buffer = Buffer::new(80, 24);
        let prev_buffer = Buffer::new(80, 24);
        let renderer = Renderer::new();

        Ok(Self {
            backend,
            buffer,
            prev_buffer,
            renderer,
            should_quit: false,
            root: None,
            store: Store::new(),
            focus: FocusManager::new(),
            timeline: Timeline::new(),
            last_tick: Instant::now(),
            #[cfg(feature = "tokio")]
            async_runtime: None,
        })
    }

    /// Set the root component for the app
    pub fn root(mut self, component: impl Component + 'static) -> Self {
        self.root = Some(Box::new(component));
        self
    }

    /// Get access to the store for registering signals
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Get mutable access to the store
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }

    /// Get access to the focus manager
    pub fn focus(&self) -> &FocusManager {
        &self.focus
    }

    /// Get mutable access to the focus manager
    pub fn focus_mut(&mut self) -> &mut FocusManager {
        &mut self.focus
    }

    /// Get access to the animation timeline
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    /// Get mutable access to the animation timeline
    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    /// Enable async support (requires tokio feature)
    #[cfg(feature = "tokio")]
    pub fn with_async(mut self) -> Result<Self> {
        self.async_runtime = Some(AsyncRuntime::new()?);
        Ok(self)
    }

    pub fn run(mut self) -> Result<()> {
        // Setup terminal
        self.backend.enter_raw_mode()?;
        self.backend.enter_alt_screen()?;
        self.backend.cursor_hide()?;
        self.backend.clear()?;

        // Mount the root component if present
        if let Some(ref mut root) = self.root {
            let mut mount_ctx = MountContext {
                store: &mut self.store,
            };
            root.mount(&mut mount_ctx);
        }

        // Install panic hook to restore terminal
        // This is important - if we panic without cleanup, the terminal stays messed up
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Try to restore terminal before panicking
            #[cfg(feature = "crossterm")]
            {
                let _ = crossterm::terminal::disable_raw_mode();
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::terminal::LeaveAlternateScreen,
                    crossterm::cursor::Show
                );
            }
            original_hook(panic_info);
        }));

        // Main loop
        let tick_rate = Duration::from_millis(16); // ~60 FPS
        while !self.should_quit {
            let frame_start = Instant::now();

            // Check terminal size and resize buffer if needed
            let (w, h) = self.backend.size()?;
            if self.buffer.width != w || self.buffer.height != h {
                self.buffer.resize(w, h);
                self.prev_buffer.resize(w, h);
            }

            // Update animations
            let delta = frame_start.duration_since(self.last_tick);
            self.timeline.update(delta);
            self.last_tick = frame_start;

            // Poll for events
            if let Some(event) = self.backend.read_event(tick_rate)? {
                self.handle_event(event)?;
            }

            // Render frame
            if self.root.is_some() {
                self.render_component_frame()?;
            } else {
                // Fallback to test pattern if no root component
                self.render_test_frame()?;
            }

            // Sleep to maintain frame rate
            let elapsed = frame_start.elapsed();
            if elapsed < tick_rate {
                std::thread::sleep(tick_rate - elapsed);
            }
        }

        // Cleanup
        // Unmount root component
        if let Some(ref mut root) = self.root {
            let mut mount_ctx = MountContext {
                store: &mut self.store,
            };
            root.unmount(&mut mount_ctx);
        }

        self.cleanup()?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        // Give root component first chance to handle the event
        if let Some(ref mut root) = self.root {
            let w = self.buffer.width;
            let h = self.buffer.height;
            let area = Rect::new(0, 0, w, h);

            let mut event_ctx = EventContext {
                store: &mut self.store,
                area,
            };

            use crate::event::EventResult;
            match root.handle_event(&event, &mut event_ctx) {
                EventResult::Consumed | EventResult::Handled => {
                    return Ok(());
                }
                EventResult::Ignored => {
                    // Fall through to default handling
                }
            }
        }

        // Handle focus navigation with Tab/Shift+Tab
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.focus.focus_prev();
                    } else {
                        self.focus.focus_next();
                    }
                    return Ok(());
                }
                _ => {}
            },
            _ => {}
        }

        // Default event handling - quit on 'q' or Ctrl+C
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn render_component_frame(&mut self) -> Result<()> {
        // Clear buffer
        self.buffer.clear();

        if let Some(ref root) = self.root {
            let w = self.buffer.width;
            let h = self.buffer.height;
            let area = Rect::new(0, 0, w, h);

            // Render component to view tree
            let render_ctx = RenderContext::new(&mut self.buffer, area, &self.store);
            let view_tree = root.render(&render_ctx);

            // Render view tree to buffer
            let mut render_ctx = RenderContext::new(&mut self.buffer, area, &self.store);
            view_tree.render(&mut render_ctx);
        }

        // Render using the efficient diff-based renderer
        self.renderer
            .render(self.backend.as_mut(), Some(&self.prev_buffer), &self.buffer)?;

        // Swap buffers for next frame
        std::mem::swap(&mut self.buffer, &mut self.prev_buffer);

        Ok(())
    }

    fn render_test_frame(&mut self) -> Result<()> {
        // Simple test pattern so we know it's working
        // Clear buffer
        self.buffer.clear();

        // Draw a border and some text
        use crate::render::Cell;
        use crate::theme::{Color, Style};

        let w = self.buffer.width;
        let h = self.buffer.height;

        // Top border
        for x in 0..w {
            self.buffer.set(x, 0, Cell::new('─'));
        }

        // Bottom border
        for x in 0..w {
            self.buffer.set(x, h.saturating_sub(1), Cell::new('─'));
        }

        // Left/right borders
        for y in 0..h {
            self.buffer.set(0, y, Cell::new('│'));
            self.buffer.set(w.saturating_sub(1), y, Cell::new('│'));
        }

        // Corners
        self.buffer.set(0, 0, Cell::new('┌'));
        self.buffer.set(w.saturating_sub(1), 0, Cell::new('┐'));
        self.buffer.set(0, h.saturating_sub(1), Cell::new('└'));
        self.buffer
            .set(w.saturating_sub(1), h.saturating_sub(1), Cell::new('┘'));

        // Welcome text
        let msg = "rsdrav v0.1.0 - Press 'q' to quit";
        let x_offset = (w / 2).saturating_sub((msg.len() / 2) as u16);
        let y_offset = h / 2;

        for (i, ch) in msg.chars().enumerate() {
            let style = Style::new().fg(Color::CYAN);
            self.buffer
                .set(x_offset + i as u16, y_offset, Cell::with_style(ch, style));
        }

        // Render using the efficient diff-based renderer
        self.renderer
            .render(self.backend.as_mut(), Some(&self.prev_buffer), &self.buffer)?;

        // Swap buffers for next frame
        std::mem::swap(&mut self.buffer, &mut self.prev_buffer);

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        self.backend.cursor_show()?;
        self.backend.leave_alt_screen()?;
        self.backend.leave_raw_mode()?;
        Ok(())
    }
}

// Ensure cleanup happens even on panic
impl Drop for App {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
