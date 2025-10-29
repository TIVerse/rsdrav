use crate::error::Result;
use crate::event::Event;
use std::time::Duration;

/// Backend abstraction for terminal control
///
/// Lets us swap between crossterm, termion, or test backends
pub trait Backend: Send + Sync {
    /// Enter raw mode (disable line buffering, echo)
    fn enter_raw_mode(&mut self) -> Result<()>;

    /// Leave raw mode
    fn leave_raw_mode(&mut self) -> Result<()>;

    /// Enter alternate screen
    fn enter_alt_screen(&mut self) -> Result<()>;

    /// Leave alternate screen
    fn leave_alt_screen(&mut self) -> Result<()>;

    /// Enable mouse capture
    fn enable_mouse(&mut self) -> Result<()>;

    /// Disable mouse capture
    fn disable_mouse(&mut self) -> Result<()>;

    /// Get terminal size
    fn size(&self) -> Result<(u16, u16)>;

    /// Clear screen
    fn clear(&mut self) -> Result<()>;

    /// Flush output
    fn flush(&mut self) -> Result<()>;

    /// Write bytes to terminal
    fn write(&mut self, content: &[u8]) -> Result<()>;

    /// Read event with timeout (returns None if timeout)
    fn read_event(&mut self, timeout: Duration) -> Result<Option<Event>>;

    /// Move cursor to position
    fn cursor_goto(&mut self, x: u16, y: u16) -> Result<()>;

    /// Show cursor
    fn cursor_show(&mut self) -> Result<()>;

    /// Hide cursor
    fn cursor_hide(&mut self) -> Result<()>;
}

#[cfg(feature = "crossterm")]
mod crossterm_impl {
    use super::*;
    use crate::event::Event;
    use crossterm::{cursor, event as ct_event, execute, queue, terminal};
    use std::io::{stdout, Stdout, Write};

    pub struct CrosstermBackend {
        stdout: Stdout,
    }

    impl CrosstermBackend {
        pub fn new() -> Result<Self> {
            Ok(Self { stdout: stdout() })
        }
    }

    impl Backend for CrosstermBackend {
        fn enter_raw_mode(&mut self) -> Result<()> {
            terminal::enable_raw_mode().map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn leave_raw_mode(&mut self) -> Result<()> {
            terminal::disable_raw_mode().map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn enter_alt_screen(&mut self) -> Result<()> {
            execute!(self.stdout, terminal::EnterAlternateScreen)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn leave_alt_screen(&mut self) -> Result<()> {
            execute!(self.stdout, terminal::LeaveAlternateScreen)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn enable_mouse(&mut self) -> Result<()> {
            execute!(self.stdout, ct_event::EnableMouseCapture)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn disable_mouse(&mut self) -> Result<()> {
            execute!(self.stdout, ct_event::DisableMouseCapture)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn size(&self) -> Result<(u16, u16)> {
            terminal::size().map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn clear(&mut self) -> Result<()> {
            execute!(self.stdout, terminal::Clear(terminal::ClearType::All))
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn flush(&mut self) -> Result<()> {
            self.stdout
                .flush()
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn write(&mut self, content: &[u8]) -> Result<()> {
            self.stdout
                .write_all(content)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn read_event(&mut self, timeout: Duration) -> Result<Option<Event>> {
            if ct_event::poll(timeout).map_err(|e| crate::error::Error::Event(e.to_string()))? {
                let ev = ct_event::read().map_err(|e| crate::error::Error::Event(e.to_string()))?;
                Ok(Some(Event::from_crossterm(ev)))
            } else {
                Ok(None)
            }
        }

        fn cursor_goto(&mut self, x: u16, y: u16) -> Result<()> {
            queue!(self.stdout, cursor::MoveTo(x, y))
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn cursor_show(&mut self) -> Result<()> {
            execute!(self.stdout, cursor::Show)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn cursor_hide(&mut self) -> Result<()> {
            execute!(self.stdout, cursor::Hide)
                .map_err(|e| crate::error::Error::Backend(e.to_string()))
        }
    }
}

#[cfg(feature = "crossterm")]
pub use crossterm_impl::CrosstermBackend;

#[cfg(feature = "termion")]
mod termion_impl {
    use super::*;
    use crate::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use std::io::{stdin, stdout, Stdin, Stdout, Write};
    use std::sync::mpsc::{channel, Receiver, TryRecvError};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use termion::event::{Event as TermionEvent, Key as TermionKey, MouseEvent as TermionMouse};
    use termion::input::{MouseTerminal, TermRead};
    use termion::raw::{IntoRawMode, RawTerminal};
    use termion::screen::AlternateScreen;

    pub struct TermionBackend {
        stdout: Option<MouseTerminal<RawTerminal<Stdout>>>,
        event_rx: Option<Arc<Mutex<Receiver<TermionEvent>>>>,
        in_alt_screen: bool,
    }

    impl TermionBackend {
        pub fn new() -> Result<Self> {
            Ok(Self {
                stdout: None,
                event_rx: None,
                in_alt_screen: false,
            })
        }

        // Helper to setup event reader thread
        fn setup_event_reader(&mut self) {
            let (tx, rx) = channel();
            self.event_rx = Some(Arc::new(Mutex::new(rx)));

            // Spawn thread to read events
            thread::spawn(move || {
                let stdin = std::io::stdin();
                for evt in stdin.events() {
                    if let Ok(evt) = evt {
                        if tx.send(evt).is_err() {
                            break; // Channel closed
                        }
                    }
                }
            });
        }
    }

    impl Backend for TermionBackend {
        fn enter_raw_mode(&mut self) -> Result<()> {
            // Termion handles this when creating RawTerminal
            let stdout = stdout()
                .into_raw_mode()
                .map_err(|e| crate::error::Error::Backend(e.to_string()))?;

            let mouse_terminal = MouseTerminal::from(stdout);
            self.stdout = Some(mouse_terminal);
            self.setup_event_reader();

            Ok(())
        }

        fn leave_raw_mode(&mut self) -> Result<()> {
            // Dropping stdout will restore terminal
            self.stdout = None;
            Ok(())
        }

        fn enter_alt_screen(&mut self) -> Result<()> {
            // Termion uses escape codes for alternate screen
            if let Some(ref mut stdout) = self.stdout {
                write!(stdout, "{}", termion::screen::ToAlternateScreen)
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
                self.in_alt_screen = true;
            }
            Ok(())
        }

        fn leave_alt_screen(&mut self) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                if self.in_alt_screen {
                    write!(stdout, "{}", termion::screen::ToMainScreen)
                        .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
                    self.in_alt_screen = false;
                }
            }
            Ok(())
        }

        fn enable_mouse(&mut self) -> Result<()> {
            // Already enabled via MouseTerminal
            Ok(())
        }

        fn disable_mouse(&mut self) -> Result<()> {
            // Handled when dropping stdout
            Ok(())
        }

        fn size(&self) -> Result<(u16, u16)> {
            termion::terminal_size().map_err(|e| crate::error::Error::Backend(e.to_string()))
        }

        fn clear(&mut self) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                write!(stdout, "{}", termion::clear::All)
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
            }
            Ok(())
        }

        fn flush(&mut self) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                stdout
                    .flush()
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
            }
            Ok(())
        }

        fn write(&mut self, content: &[u8]) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                stdout
                    .write_all(content)
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
            }
            Ok(())
        }

        fn read_event(&mut self, timeout: Duration) -> Result<Option<Event>> {
            // Use timeout with try_recv
            if let Some(ref rx) = self.event_rx {
                let start = std::time::Instant::now();
                loop {
                    let result = rx.lock().unwrap().try_recv();
                    match result {
                        Ok(evt) => return Ok(Some(convert_termion_event(evt))),
                        Err(TryRecvError::Empty) => {
                            if start.elapsed() >= timeout {
                                return Ok(None);
                            }
                            // Small sleep to avoid busy waiting
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(TryRecvError::Disconnected) => {
                            return Err(crate::error::Error::Event(
                                "Event channel disconnected".into(),
                            ));
                        }
                    }
                }
            }
            Ok(None)
        }

        fn cursor_goto(&mut self, x: u16, y: u16) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                // Termion uses 1-indexed coordinates
                write!(stdout, "{}", termion::cursor::Goto(x + 1, y + 1))
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
            }
            Ok(())
        }

        fn cursor_show(&mut self) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                write!(stdout, "{}", termion::cursor::Show)
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
            }
            Ok(())
        }

        fn cursor_hide(&mut self) -> Result<()> {
            if let Some(ref mut stdout) = self.stdout {
                write!(stdout, "{}", termion::cursor::Hide)
                    .map_err(|e| crate::error::Error::Backend(e.to_string()))?;
            }
            Ok(())
        }
    }

    // Convert termion events to our Event type
    fn convert_termion_event(evt: TermionEvent) -> Event {
        match evt {
            TermionEvent::Key(key) => Event::Key(convert_key(key)),
            TermionEvent::Mouse(mouse) => Event::Mouse(convert_mouse(mouse)),
            TermionEvent::Unsupported(_) => Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::empty(),
            }),
        }
    }

    fn convert_key(key: TermionKey) -> KeyEvent {
        match key {
            TermionKey::Char(c) => KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Ctrl(c) => KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::CONTROL,
            },
            TermionKey::Alt(c) => KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::ALT,
            },
            TermionKey::F(n) => KeyEvent {
                code: KeyCode::F(n),
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Backspace => KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Left => KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Right => KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Up => KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Down => KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Home => KeyEvent {
                code: KeyCode::Home,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::End => KeyEvent {
                code: KeyCode::End,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::PageUp => KeyEvent {
                code: KeyCode::PageUp,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::PageDown => KeyEvent {
                code: KeyCode::PageDown,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Delete => KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Insert => KeyEvent {
                code: KeyCode::Insert,
                modifiers: KeyModifiers::empty(),
            },
            TermionKey::Esc => KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::empty(),
            },
            _ => KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::empty(),
            },
        }
    }

    fn convert_mouse(mouse: TermionMouse) -> MouseEvent {
        use termion::event::MouseButton as TButton;

        match mouse {
            TermionMouse::Press(btn, x, y) => MouseEvent {
                kind: MouseEventKind::Down(convert_mouse_button(btn)),
                x: x.saturating_sub(1), // Termion uses 1-indexed
                y: y.saturating_sub(1),
                modifiers: KeyModifiers::empty(),
            },
            TermionMouse::Release(x, y) => MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                x: x.saturating_sub(1),
                y: y.saturating_sub(1),
                modifiers: KeyModifiers::empty(),
            },
            TermionMouse::Hold(x, y) => MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                x: x.saturating_sub(1),
                y: y.saturating_sub(1),
                modifiers: KeyModifiers::empty(),
            },
        }
    }

    fn convert_mouse_button(btn: termion::event::MouseButton) -> MouseButton {
        use termion::event::MouseButton as TButton;
        match btn {
            TButton::Left => MouseButton::Left,
            TButton::Right => MouseButton::Right,
            TButton::Middle => MouseButton::Middle,
            TButton::WheelUp => MouseButton::Left, // Approximation
            TButton::WheelDown => MouseButton::Left, // Approximation
            TButton::WheelLeft => MouseButton::Left, // Approximation
            TButton::WheelRight => MouseButton::Left, // Approximation
        }
    }
}

#[cfg(feature = "termion")]
pub use termion_impl::TermionBackend;
