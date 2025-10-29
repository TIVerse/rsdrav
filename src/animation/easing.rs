//! Easing functions for animations

use std::f32::consts::PI;

/// Easing function types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
}

impl EasingFunction {
    /// Apply the easing function to a normalized time value (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseOutQuad => t * (2.0 - t),
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = 2.0 * t - 2.0;
                    (t * t * t + 2.0) / 2.0
                }
            }
            EasingFunction::EaseInSine => 1.0 - ((t * PI) / 2.0).cos(),
            EasingFunction::EaseOutSine => ((t * PI) / 2.0).sin(),
            EasingFunction::EaseInOutSine => -(((t * PI).cos() - 1.0) / 2.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_quad() {
        let easing = EasingFunction::EaseInQuad;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.25);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_clamp() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(-1.0), 0.0);
        assert_eq!(easing.apply(2.0), 1.0);
    }
}
