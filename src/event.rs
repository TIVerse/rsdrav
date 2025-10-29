// Event types for keyboard, mouse, resize, etc.
// Full event routing system comes later

/// Result of event handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// Event was processed by this component
    Handled,
    /// Event was not relevant to this component
    Ignored,
    /// Event was processed, stop propagation
    Consumed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Esc,
    Null,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b0000_0001;
        const CONTROL = 0b0000_0010;
        const ALT = 0b0000_0100;
        const SUPER = 0b0000_1000;  // Windows key / Command
        const HYPER = 0b0001_0000;
        const META = 0b0010_0000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseEventKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollDown,
    ScrollUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub x: u16,
    pub y: u16,
    pub modifiers: KeyModifiers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
}

// Conversion from crossterm events
#[cfg(feature = "crossterm")]
impl Event {
    pub fn from_crossterm(ev: crossterm::event::Event) -> Self {
        use crossterm::event as ct;

        match ev {
            ct::Event::Key(key) => {
                let code = match key.code {
                    ct::KeyCode::Char(c) => KeyCode::Char(c),
                    ct::KeyCode::Backspace => KeyCode::Backspace,
                    ct::KeyCode::Enter => KeyCode::Enter,
                    ct::KeyCode::Left => KeyCode::Left,
                    ct::KeyCode::Right => KeyCode::Right,
                    ct::KeyCode::Up => KeyCode::Up,
                    ct::KeyCode::Down => KeyCode::Down,
                    ct::KeyCode::Home => KeyCode::Home,
                    ct::KeyCode::End => KeyCode::End,
                    ct::KeyCode::PageUp => KeyCode::PageUp,
                    ct::KeyCode::PageDown => KeyCode::PageDown,
                    ct::KeyCode::Tab => KeyCode::Tab,
                    ct::KeyCode::BackTab => KeyCode::BackTab,
                    ct::KeyCode::Delete => KeyCode::Delete,
                    ct::KeyCode::Insert => KeyCode::Insert,
                    ct::KeyCode::F(n) => KeyCode::F(n),
                    ct::KeyCode::Esc => KeyCode::Esc,
                    ct::KeyCode::Null => KeyCode::Null,
                    _ => KeyCode::Null, // fallback for stuff we don't handle yet
                };

                let mut mods = KeyModifiers::empty();
                if key.modifiers.contains(ct::KeyModifiers::SHIFT) {
                    mods |= KeyModifiers::SHIFT;
                }
                if key.modifiers.contains(ct::KeyModifiers::CONTROL) {
                    mods |= KeyModifiers::CONTROL;
                }
                if key.modifiers.contains(ct::KeyModifiers::ALT) {
                    mods |= KeyModifiers::ALT;
                }
                if key.modifiers.contains(ct::KeyModifiers::SUPER) {
                    mods |= KeyModifiers::SUPER;
                }
                if key.modifiers.contains(ct::KeyModifiers::HYPER) {
                    mods |= KeyModifiers::HYPER;
                }
                if key.modifiers.contains(ct::KeyModifiers::META) {
                    mods |= KeyModifiers::META;
                }

                Event::Key(KeyEvent::new(code, mods))
            }

            ct::Event::Mouse(mouse) => {
                use ct::MouseEventKind as MK;

                let kind = match mouse.kind {
                    MK::Down(ct::MouseButton::Left) => MouseEventKind::Down(MouseButton::Left),
                    MK::Down(ct::MouseButton::Right) => MouseEventKind::Down(MouseButton::Right),
                    MK::Down(ct::MouseButton::Middle) => MouseEventKind::Down(MouseButton::Middle),
                    MK::Up(ct::MouseButton::Left) => MouseEventKind::Up(MouseButton::Left),
                    MK::Up(ct::MouseButton::Right) => MouseEventKind::Up(MouseButton::Right),
                    MK::Up(ct::MouseButton::Middle) => MouseEventKind::Up(MouseButton::Middle),
                    MK::Drag(ct::MouseButton::Left) => MouseEventKind::Drag(MouseButton::Left),
                    MK::Drag(ct::MouseButton::Right) => MouseEventKind::Drag(MouseButton::Right),
                    MK::Drag(ct::MouseButton::Middle) => MouseEventKind::Drag(MouseButton::Middle),
                    MK::Moved => MouseEventKind::Moved,
                    MK::ScrollDown => MouseEventKind::ScrollDown,
                    MK::ScrollUp => MouseEventKind::ScrollUp,
                    _ => MouseEventKind::Moved, // fallback
                };

                let mut mods = KeyModifiers::empty();
                if mouse.modifiers.contains(ct::KeyModifiers::SHIFT) {
                    mods |= KeyModifiers::SHIFT;
                }
                if mouse.modifiers.contains(ct::KeyModifiers::CONTROL) {
                    mods |= KeyModifiers::CONTROL;
                }
                if mouse.modifiers.contains(ct::KeyModifiers::ALT) {
                    mods |= KeyModifiers::ALT;
                }

                Event::Mouse(MouseEvent {
                    kind,
                    x: mouse.column,
                    y: mouse.row,
                    modifiers: mods,
                })
            }

            ct::Event::Resize(w, h) => Event::Resize(w, h),
            ct::Event::FocusGained => Event::FocusGained,
            ct::Event::FocusLost => Event::FocusLost,
            ct::Event::Paste(s) => Event::Paste(s),
        }
    }
}
