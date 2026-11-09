//! LPPX2D — 2D graphics framework for LPPO.
//!
//! Provides window creation, a render loop, basic shapes, colors, and input
//! handling for 2D applications and games. Self-contained Cargo crate so it
//! builds as a standalone installed package.

////////////////////////////////////////////////////////////////////////////////
// Math
////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////
// Shapes
////////////////////////////////////////////////////////////////////////////////

/// A drawable 2D shape.
#[derive(Debug, Clone)]
pub enum Shape {
    Rectangle { position: Vec2, size: Vec2, color: Color },
    Circle { center: Vec2, radius: f32, color: Color },
    Line { from: Vec2, to: Vec2, color: Color, thickness: f32 },
    Point { position: Vec2, color: Color },
}

/// A 2D rendering context. Holds the shapes submitted for the current frame.
#[derive(Debug, Clone, Default)]
pub struct Canvas {
    pub shapes: Vec<Shape>,
    pub clear_color: Color,
}

impl Canvas {
    pub fn new() -> Self {
        Self { shapes: Vec::new(), clear_color: Color::black() }
    }
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
    }
    pub fn draw_rectangle(&mut self, position: Vec2, size: Vec2, color: Color) {
        self.shapes.push(Shape::Rectangle { position, size, color });
    }
    pub fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        self.shapes.push(Shape::Circle { center, radius, color });
    }
    pub fn draw_line(&mut self, from: Vec2, to: Vec2, color: Color) {
        self.shapes.push(Shape::Line { from, to, color, thickness: 1.0 });
    }
    pub fn draw_point(&mut self, position: Vec2, color: Color) {
        self.shapes.push(Shape::Point { position, color });
    }
    pub fn shape_count(&self) -> usize { self.shapes.len() }
    pub fn shapes(&self) -> &[Shape] { &self.shapes }
}

////////////////////////////////////////////////////////////////////////////////
// Window2d
////////////////////////////////////////////////////////////////////////////////

/// A 2D window that owns a canvas.
pub struct Window2d {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub canvas: Canvas,
}

impl Window2d {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            title: title.to_string(),
            canvas: Canvas::new(),
        }
    }
    pub fn info(&self) -> (String, u32, u32) {
        (self.title.clone(), self.width, self.height)
    }
    pub fn begin_frame(&mut self) {
        self.canvas.shapes.clear();
    }
    pub fn end_frame(&mut self) {
        // Null backend: nothing to present.
    }
}

////////////////////////////////////////////////////////////////////////////////
// Rendering
////////////////////////////////////////////////////////////////////////////////

/// Render a canvas. The default (null backend) is a no-op.
pub fn render(_canvas: &Canvas) {}

/// Convenience: run a simple frame — clear, draw, present.
pub fn frame(window: &mut Window2d, clear_color: Color, draw: impl FnOnce(&mut Canvas)) {
    window.begin_frame();
    window.canvas.clear(clear_color);
    draw(&mut window.canvas);
    render(&window.canvas);
    window.end_frame();
}

////////////////////////////////////////////////////////////////////////////////
// C-ABI exports for LPPO FFI integration
//
// These symbols match `lppo_interface.json` and are called by LPPO programs
// via `libloading`. The fixed signature is `extern "C" fn(i64, i64, i64,
// i64) -> i64`; LPPO pads missing arguments with 0.

#[no_mangle]
pub extern "C" fn x2d_window_create(_title_hash: i64, width: i64, height: i64, _: i64) -> i64 {
    println!("[x2d] window_create {}x{}", width, height);
    1
}

#[no_mangle]
pub extern "C" fn x2d_canvas_clear(r: i64, g: i64, b: i64, a: i64) -> i64 {
    println!("[x2d] canvas_clear rgba({}, {}, {}, {})", r, g, b, a);
    0
}

#[no_mangle]
pub extern "C" fn x2d_draw_rectangle(
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    r: i64,
    g: i64,
    b: i64,
    _: i64,
) -> i64 {
    println!("[x2d] draw_rectangle ({},{}) {}x{} rgba({},{},{})", x, y, w, h, r, g, b);
    0
}

#[no_mangle]
pub extern "C" fn x2d_draw_circle(cx: i64, cy: i64, radius: i64, _: i64) -> i64 {
    println!("[x2d] draw_circle ({},{}) r={}", cx, cy, radius);
    0
}

#[no_mangle]
pub extern "C" fn x2d_begin_frame(_win: i64) -> i64 {
    println!("[x2d] begin_frame");
    0
}

#[no_mangle]
pub extern "C" fn x2d_end_frame(_win: i64) -> i64 {
    println!("[x2d] end_frame");
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_records_shapes() {
        let mut canvas = Canvas::new();
        canvas.draw_rectangle(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0), Color::red());
        canvas.draw_circle(Vec2::new(5.0, 5.0), 3.0, Color::blue());
        assert_eq!(canvas.shape_count(), 2);
    }

    #[test]
    fn window2d_creation() {
        let mut w = Window2d::new("test", 800, 600);
        let (t, width, height) = w.info();
        assert_eq!(t, "test");
        assert_eq!(width, 800);
        assert_eq!(height, 600);
        w.begin_frame();
        w.canvas.draw_point(Vec2::new(1.0, 2.0), Color::green());
        assert_eq!(w.canvas.shape_count(), 1);
        w.end_frame();
    }
}
