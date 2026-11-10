//! Math types for LPPX2D.

/// A 2D vector / point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// An RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn white() -> Self { Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 } }
    pub fn black() -> Self { Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } }
    pub fn red() -> Self { Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 } }
    pub fn green() -> Self { Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 } }
    pub fn blue() -> Self { Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 } }
}
