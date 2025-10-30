//! Animation Example
//!
//! Demonstrates:
//! - Color transitions with tweens
//! - Animated progress bar
//! - Timeline management
//! - Easing functions
//!
//! Controls:
//! - Press 's' to start animations
//! - Press 'r' to reset
//! - Press 'q' to quit

use rsdrav::animation::{EasingFunction, Tween};
use rsdrav::prelude::*;
use std::time::Duration;

fn main() -> rsdrav::Result<()> {
    App::new()?.root(AnimationDemo::new()).run()
}

struct AnimationDemo {
    progress: Signal<f32>,
    color: Signal<Color>,
    color_tween: Option<Tween<Color>>,
    progress_tween: Option<Tween<f32>>,
}

impl AnimationDemo {
    fn new() -> Self {
        Self {
            progress: Signal::new(0.0),
            color: Signal::new(Color::RED),
            color_tween: None,
            progress_tween: None,
        }
    }

    fn start_animations(&mut self) {
        // Animate progress from 0 to 100
        self.progress_tween = Some(
            Tween::new(0.0_f32, 100.0_f32, Duration::from_secs(3))
                .easing(EasingFunction::EaseInOutCubic),
        );

        // Animate color from red to blue
        self.color_tween = Some(
            Tween::new(Color::RED, Color::BLUE, Duration::from_secs(3))
                .easing(EasingFunction::EaseInOutQuad),
        );
    }

    fn reset(&mut self) {
        self.progress.set(0.0);
        self.color.set(Color::RED);
        self.color_tween = None;
        self.progress_tween = None;
    }
}

impl Component for AnimationDemo {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        let title = Text::new("=== Animation Demo ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        let progress_text = Text::bind({
            let p = self.progress.clone();
            move || format!("Progress: {:.1}%", p.get())
        })
        .fg(Color::GREEN);

        let progress_bar = ProgressBar::new(self.progress.clone())
            .width(40)
            .show_percentage(true);

        let color_display = Panel::new().title("Color Animation").child(
            Text::new("████████████")
                .fg(self.color.get())
                .add_modifier(Modifier::BOLD),
        );

        let instructions = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Controls:").fg(Color::YELLOW))
            .push(Text::new("  s - Start animations").fg(Color::GRAY))
            .push(Text::new("  r - Reset").fg(Color::GRAY))
            .push(Text::new("  q - Quit").fg(Color::GRAY));

        VStack::new()
            .gap(1)
            .push(title)
            .push(Text::new(""))
            .push(progress_text)
            .push(progress_bar)
            .push(Text::new(""))
            .push(color_display)
            .push(instructions)
            .render(_ctx)
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.start_animations();
                    return EventResult::Handled;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.reset();
                    return EventResult::Handled;
                }
                _ => {}
            }
        }

        EventResult::Ignored
    }

    fn update(&mut self, _ctx: &mut UpdateContext) -> bool {
        // Update animations manually here
        // In a real app, this would be driven by the timeline in App
        let delta = Duration::from_millis(16); // Approximate frame time

        if let Some(ref mut tween) = self.progress_tween {
            tween.update(delta);
            self.progress.set(tween.value());

            if tween.is_complete() {
                self.progress_tween = None;
            }
        }

        if let Some(ref mut tween) = self.color_tween {
            tween.update(delta);
            self.color.set(tween.value());

            if tween.is_complete() {
                self.color_tween = None;
            }
        }

        // Always re-render if we have active animations
        self.progress_tween.is_some() || self.color_tween.is_some()
    }
}
