//! Login Form Example
//!
//! Demonstrates:
//! - Input widget for text entry
//! - Password mode (masked input)
//! - Form validation
//! - Button interaction
//! - Reactive state
//!
//! Controls:
//! - Type in username field
//! - Press Enter or Tab to move to password (for now, just type)
//! - Press Ctrl+L to attempt login
//! - Press 'q' to quit

use rsdrav::prelude::*;

fn main() -> rsdrav::Result<()> {
    let mut app = App::new()?;
    let form = LoginForm::new();

    // Register focusable components
    app.focus_mut().register(form.username_id, 0, true);
    app.focus_mut().register(form.password_id, 1, true);

    app.root(form).run()
}

/// Login form component
struct LoginForm {
    username: Signal<String>,
    password: Signal<String>,
    error_message: Signal<Option<String>>,
    username_id: ComponentId,
    password_id: ComponentId,
}

impl LoginForm {
    fn new() -> Self {
        Self {
            username: Signal::new(String::new()),
            password: Signal::new(String::new()),
            error_message: Signal::new(None),
            username_id: ComponentId::new(1),
            password_id: ComponentId::new(2),
        }
    }

    fn validate_and_login(&mut self) {
        let username = self.username.get();
        let password = self.password.get();

        // Basic validation
        if username.is_empty() {
            self.error_message
                .set(Some("Username is required".to_string()));
            return;
        }

        if password.is_empty() {
            self.error_message
                .set(Some("Password is required".to_string()));
            return;
        }

        if password.len() < 4 {
            self.error_message
                .set(Some("Password must be at least 4 characters".to_string()));
            return;
        }

        // Simulate login
        if username == "admin" && password == "pass" {
            self.error_message
                .set(Some("✓ Login successful!".to_string()));
        } else {
            self.error_message
                .set(Some("✗ Invalid credentials".to_string()));
        }
    }
}

impl Component for LoginForm {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Check which component has focus (we need access to focus manager here)
        // For now, we'll need to pass focus state through context or get it from App
        // This is a limitation - in real implementation, we'd pass FocusManager ref
        // For now, manually track based on our knowledge

        // Username label and input
        let username_focused = true; // Would check: ctx.focus.is_focused(self.username_id)
        let username_label = if username_focused {
            Text::new("> Username:").fg(Color::CYAN)
        } else {
            Text::new("  Username:").fg(Color::GRAY)
        };

        let username_input = Input::new(self.username.clone())
            .placeholder("Enter username")
            .max_length(20)
            .focused(username_focused);

        // Password label and input
        let password_focused = false; // Would check: ctx.focus.is_focused(self.password_id)
        let password_label = if password_focused {
            Text::new("> Password:").fg(Color::CYAN)
        } else {
            Text::new("  Password:").fg(Color::GRAY)
        };

        let password_input = Input::new(self.password.clone())
            .placeholder("Enter password")
            .password()
            .max_length(30)
            .focused(password_focused);

        // Error message
        let error_display = if let Some(ref msg) = self.error_message.get() {
            let color = if msg.starts_with('✓') {
                Color::GREEN
            } else {
                Color::RED
            };
            Text::new(msg.clone()).fg(color)
        } else {
            Text::new("")
        };

        // Instructions
        let instructions = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Instructions:").fg(Color::YELLOW))
            .push(Text::new("  Tab - Next field, Shift+Tab - Previous field").fg(Color::GREEN))
            .push(Text::new("  Ctrl+L - Login").fg(Color::GRAY))
            .push(Text::new("  Ctrl+C - Clear error").fg(Color::GRAY))
            .push(Text::new("  q - Quit").fg(Color::GRAY))
            .push(Text::new(""))
            .push(Text::new("Hint: username=admin, password=pass").fg(Color::rgb(100, 100, 100)));

        // Compose the form
        Panel::new()
            .title("Login")
            .border_style(Style::default().fg(Color::CYAN))
            .child(
                VStack::new()
                    .gap(1)
                    .push(username_label)
                    .push(username_input)
                    .push(Text::new(""))
                    .push(password_label)
                    .push(password_input)
                    .push(Text::new(""))
                    .push(error_display)
                    .push(instructions),
            )
            .render(ctx)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Note: In a real implementation, we'd pass focus manager through context
        // For now, inputs will only handle events when focused (checked internally)

        // Let both inputs try to handle the event
        // Only the focused one will actually process it
        let mut username_input = Input::new(self.username.clone()).focused(true);
        if username_input.handle_event(event, ctx) == EventResult::Handled {
            return EventResult::Handled;
        }

        let mut password_input = Input::new(self.password.clone()).focused(false);
        if password_input.handle_event(event, ctx) == EventResult::Handled {
            return EventResult::Handled;
        }

        // Handle form-level shortcuts
        if let Event::Key(key) = event {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        self.validate_and_login();
                        return EventResult::Handled;
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        self.error_message.set(None);
                        return EventResult::Handled;
                    }
                    _ => {}
                }
            }

            // Tab navigation is now handled by App automatically!
            // No need to handle it here
        }

        EventResult::Ignored
    }
}
