// Basic theme/style types
// Full theme system comes later, but we need these for rendering

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    // Some common colors - makes life easier
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const GRAY: Self = Self::rgb(128, 128, 128);
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Modifier: u8 {
        const BOLD = 0b0000_0001;
        const DIM = 0b0000_0010;
        const ITALIC = 0b0000_0100;
        const UNDERLINE = 0b0000_1000;
        const BLINK = 0b0001_0000;
        const REVERSE = 0b0010_0000;
        const HIDDEN = 0b0100_0000;
        const STRIKETHROUGH = 0b1000_0000;
    }
}

/// Style for a cell - foreground, background, modifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifiers: Modifier,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers |= modifier;
        self
    }

    pub fn remove_modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers &= !modifier;
        self
    }
}

// Make Color animatable for smooth color transitions
impl crate::animation::Animatable for Color {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let r = (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8;
        let g = (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8;
        let b = (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8;
        Color { r, g, b }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Animatable;

    #[test]
    fn test_color_interpolation() {
        let black = Color::BLACK;
        let white = Color::WHITE;

        let mid = black.lerp(&white, 0.5);
        assert!(mid.r > 120 && mid.r < 135);
        assert!(mid.g > 120 && mid.g < 135);
        assert!(mid.b > 120 && mid.b < 135);
    }
}
