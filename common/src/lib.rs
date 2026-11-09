//! Shared functionality for LPPX extensions.
//!
//! Provides math types, windowing traits, input handling, and timing
//! primitives that LPPX2D and LPPX3D both depend on. Keeping this here avoids
//! duplicating the same implementation across extensions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

////////////////////////////////////////////////////////////////////////////////
// Math
////////////////////////////////////////////////////////////////////////////////

/// A 2D vector / point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    pub fn add(self, other: Vec2) -> Vec2 {
        Vec2 { x: self.x + other.x, y: self.y + other.y }
    }
    pub fn sub(self, other: Vec2) -> Vec2 {
        Vec2 { x: self.x - other.x, y: self.y - other.y }
    }
    pub fn mul(self, s: f32) -> Vec2 {
        Vec2 { x: self.x * s, y: self.y * s }
    }
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn distance(self, other: Vec2) -> f32 {
        self.sub(other).length()
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 { self.add(rhs) }
}
impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 { self.sub(rhs) }
}
impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 { self.mul(rhs) }
}

/// A 3D vector / point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
    pub fn mul(self, s: f32) -> Vec3 {
        Vec3 { x: self.x * s, y: self.y * s, z: self.z * s }
    }
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 0.0 { self.mul(1.0 / l) } else { self }
    }
}

impl std::ops::Add for Vec3 { type Output = Vec3; fn add(self, rhs: Vec3) -> Vec3 { self.add(rhs) } }
impl std::ops::Sub for Vec3 { type Output = Vec3; fn sub(self, rhs: Vec3) -> Vec3 { self.sub(rhs) } }
impl std::ops::Mul<f32> for Vec3 { type Output = Vec3; fn mul(self, rhs: f32) -> Vec3 { self.mul(rhs) } }

/// A 4x4 column-major matrix.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat4 {
    pub cols: [Vec4; 4],
}

/// A 4D vector used by Mat4 columns.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            cols: [
                Vec4 { x: 1.0, y: 0.0, z: 0.0, w: 0.0 },
                Vec4 { x: 0.0, y: 1.0, z: 0.0, w: 0.0 },
                Vec4 { x: 0.0, y: 0.0, z: 1.0, w: 0.0 },
                Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            ],
        }
    }
    pub fn translation(v: Vec3) -> Self {
        let m = Self::identity();
        Self {
            cols: [
                m.cols[0],
                m.cols[1],
                m.cols[2],
                Vec4 { x: v.x, y: v.y, z: v.z, w: 1.0 },
            ],
        }
    }
}

/// An RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
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

////////////////////////////////////////////////////////////////////////////////
// Windowing
////////////////////////////////////////////////////////////////////////////////

/// Capabilities reported by a window backend.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

/// Events produced by a window.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    Resized { width: u32, height: u32 },
    Closed,
    RedrawRequested,
}

/// Abstraction over a window backend. Extensions implement this for each
/// graphics API (wgpu, software, null, ...).
pub trait Window: Send + Sync {
    fn info(&self) -> WindowInfo;
    fn poll_events(&mut self) -> Vec<WindowEvent>;
    fn prepare_frame(&mut self);
    fn present(&mut self);
}

/// Create a default window. Returns a boxed window backend.
pub fn create_window(title: &str, width: u32, height: u32) -> Box<dyn Window> {
    Box::new(NullWindow::new(title, width, height))
}

/// A no-op window backend used when no graphics backend is enabled. It compiles
/// everywhere and lets the package build/run headless.
pub struct NullWindow {
    info: WindowInfo,
    closed: bool,
}

impl NullWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            info: WindowInfo {
                title: title.to_string(),
                width,
                height,
            },
            closed: false,
        }
    }
}

impl Window for NullWindow {
    fn info(&self) -> WindowInfo { self.info.clone() }
    fn poll_events(&mut self) -> Vec<WindowEvent> { Vec::new() }
    fn prepare_frame(&mut self) {}
    fn present(&mut self) {}
}

////////////////////////////////////////////////////////////////////////////////
// Input
////////////////////////////////////////////////////////////////////////////////

/// State of a keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Simple keyboard input snapshot.
#[derive(Debug, Clone, Default)]
pub struct Keyboard {
    pub keys: std::collections::HashMap<String, KeyState>,
}

impl Keyboard {
    pub fn is_pressed(&self, key: &str) -> bool {
        self.keys.get(key) == Some(&KeyState::Pressed)
    }
}

/// Simple mouse input snapshot.
#[derive(Debug, Clone, Default)]
pub struct Mouse {
    pub x: f32,
    pub y: f32,
    pub buttons: std::collections::HashMap<u8, KeyState>,
}

impl Mouse {
    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    pub fn button_pressed(&self, button: u8) -> bool {
        self.buttons.get(&button) == Some(&KeyState::Pressed)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Timing
////////////////////////////////////////////////////////////////////////////////

/// A frame timer that tracks delta time and frames per second.
#[derive(Debug, Clone)]
pub struct Timer {
    last: Instant,
    delta: Duration,
    frames: u64,
    accumulated: Duration,
    fps: u64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            last: Instant::now(),
            delta: Duration::ZERO,
            frames: 0,
            accumulated: Duration::ZERO,
            fps: 0,
        }
    }
    /// Call once per frame. Returns the delta time since the last call.
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        self.delta = now.duration_since(self.last);
        self.last = now;
        self.frames += 1;
        self.accumulated += self.delta;
        if self.accumulated >= Duration::from_secs(1) {
            self.fps = self.frames;
            self.frames = 0;
            self.accumulated = Duration::ZERO;
        }
        self.delta
    }
    pub fn delta(&self) -> Duration { self.delta }
    pub fn fps(&self) -> u64 { self.fps }
}

impl Default for Timer {
    fn default() -> Self { Self::new() }
}
