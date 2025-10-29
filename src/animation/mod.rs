//! Animation system for smooth transitions
//!
//! Provides easing functions and tween support for animated UI transitions.

use std::time::Duration;

mod easing;
pub use easing::*;

/// Animation tween for interpolating values over time
pub struct Tween<T> {
    start: T,
    end: T,
    duration: Duration,
    elapsed: Duration,
    easing: EasingFunction,
}

impl<T: Animatable> Tween<T> {
    /// Create a new tween animation
    pub fn new(start: T, end: T, duration: Duration) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: Duration::ZERO,
            easing: EasingFunction::Linear,
        }
    }

    /// Set the easing function
    pub fn easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// Update the animation
    pub fn update(&mut self, delta: Duration) {
        self.elapsed = (self.elapsed + delta).min(self.duration);
    }

    /// Get the current interpolated value
    pub fn value(&self) -> T {
        let t = if self.duration.as_secs_f32() > 0.0 {
            self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
        } else {
            1.0
        };

        let eased = self.easing.apply(t);
        self.start.lerp(&self.end, eased)
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Trait for types that can be animated
pub trait Animatable: Clone {
    /// Linear interpolation between self and other
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Animatable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Animatable for i32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        ((*self as f32) + ((*other as f32) - (*self as f32)) * t) as i32
    }
}

impl Animatable for u16 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        ((*self as f32) + ((*other as f32) - (*self as f32)) * t) as u16
    }
}

/// Animation timeline for managing multiple tweens
pub struct Timeline {
    animations: Vec<Box<dyn Animation>>,
}

impl Timeline {
    /// Create a new timeline
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    /// Add an animation to the timeline
    pub fn add<A: Animation + 'static>(&mut self, animation: A) {
        self.animations.push(Box::new(animation));
    }

    /// Update all animations
    pub fn update(&mut self, delta: Duration) {
        // Update all animations and remove completed ones
        self.animations.retain_mut(|anim| {
            anim.update(delta);
            !anim.is_complete()
        });
    }

    /// Get number of active animations
    pub fn count(&self) -> usize {
        self.animations.len()
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }

    /// Check if all animations are complete
    pub fn is_complete(&self) -> bool {
        self.animations.is_empty()
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for animations
pub trait Animation: Send + Sync {
    /// Update the animation
    fn update(&mut self, delta: Duration);

    /// Check if animation is complete
    fn is_complete(&self) -> bool;
}

impl<T: Animatable + Send + Sync + 'static> Animation for Tween<T> {
    fn update(&mut self, delta: Duration) {
        Tween::update(self, delta);
    }

    fn is_complete(&self) -> bool {
        Tween::is_complete(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tween_f32() {
        let mut tween = Tween::new(0.0_f32, 100.0_f32, Duration::from_secs(1));

        assert_eq!(tween.value(), 0.0);
        assert!(!tween.is_complete());

        tween.update(Duration::from_millis(500));
        let mid = tween.value();
        assert!(mid > 45.0 && mid < 55.0); // Approximately 50.0

        tween.update(Duration::from_millis(500));
        assert_eq!(tween.value(), 100.0);
        assert!(tween.is_complete());
    }

    #[test]
    fn test_tween_i32() {
        let mut tween = Tween::new(0_i32, 100_i32, Duration::from_secs(1));

        tween.update(Duration::from_millis(500));
        let mid = tween.value();
        assert!((45..=55).contains(&mid));
    }

    #[test]
    fn test_timeline() {
        let mut timeline = Timeline::new();

        let tween1 = Tween::new(0.0_f32, 10.0_f32, Duration::from_millis(100));
        let tween2 = Tween::new(0.0_f32, 20.0_f32, Duration::from_millis(200));

        timeline.add(tween1);
        timeline.add(tween2);

        assert!(!timeline.is_complete());

        timeline.update(Duration::from_millis(100));
        timeline.update(Duration::from_millis(100));

        assert!(timeline.is_complete());
    }
}
