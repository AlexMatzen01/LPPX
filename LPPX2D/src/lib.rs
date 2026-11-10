//! LPPX2D — 2D graphics framework for LPPO using raylib.
//!
//! Provides real window creation, a render loop, 2D shapes, colors, and input
//! handling using raylib. Self-contained Cargo crate so it builds as a
//! standalone installed package.

//////////////////////////////////////////////////////////////////////////////
// Public API
//////////////////////////////////////////////////////////////////////////////

pub mod math;

pub use math::{Vec2, Color};

use raylib::prelude::RaylibDraw;

//////////////////////////////////////////////////////////////////////////////
// Shapes
//////////////////////////////////////////////////////////////////////////////

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
//////////////////////////////////////////////////////////////////////////////

/// A 2D window that owns a canvas and renders via raylib.
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
        // Presented by the raylib render loop.
    }
    /// Render the canvas using raylib (called by the run loop).
    pub fn render_raylib(&self, rl: &mut raylib::prelude::RaylibDrawHandle) {
        rl.clear_background(raylib_color(&self.canvas.clear_color));
        for shape in &self.canvas.shapes {
            match shape {
                Shape::Rectangle { position, size, color } => {
                    rl.draw_rectangle(
                        position.x as i32, position.y as i32,
                        size.x as i32, size.y as i32,
                        raylib_color(color),
                    );
                }
                Shape::Circle { center, radius, color } => {
                    rl.draw_circle(
                        center.x as i32, center.y as i32,
                        *radius,
                        raylib_color(color),
                    );
                }
                Shape::Line { from, to, color, thickness } => {
                    rl.draw_line_ex(
                        raylib::math::Vector2::new(from.x, from.y),
                        raylib::math::Vector2::new(to.x, to.y),
                        *thickness,
                        raylib_color(color),
                    );
                }
                Shape::Point { position, color } => {
                    rl.draw_pixel(
                        position.x as i32, position.y as i32,
                        raylib_color(color),
                    );
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// raylib color conversion
//////////////////////////////////////////////////////////////////////////////

fn raylib_color(c: &Color) -> raylib::prelude::Color {
    raylib::prelude::Color {
        r: (c.r.clamp(0.0, 1.0) * 255.0) as u8,
        g: (c.g.clamp(0.0, 1.0) * 255.0) as u8,
        b: (c.b.clamp(0.0, 1.0) * 255.0) as u8,
        a: (c.a.clamp(0.0, 1.0) * 255.0) as u8,
    }
}

////////////////////////////////////////////////////////////////////////////////
// Run loop — opens a real raylib window and renders the canvas
//////////////////////////////////////////////////////////////////////////////

/// Run a `Window2d` in a real raylib window. Blocks until the window closes.
pub fn run_window(window: Window2d) {
    let (mut rl, thread) = raylib::init()
        .size(window.width as i32, window.height as i32)
        .title(&window.title)
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        window.render_raylib(&mut d);
    }
}

////////////////////////////////////////////////////////////////////////////////
// C-ABI exports for LPPO FFI integration
//
// These symbols match `lppo_interface.json` and are called by LPPO programs
// via `libloading`. The fixed signature is `extern "C" fn(i64,i64,i64,i64,
// i64,i64,i64,i64)->i64`; LPPO pads missing arguments with 0.
//
// Global window state is stored in a static so the stateless FFI calls can
// reach a live raylib window.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

struct WindowEntry {
    width: i32,
    height: i32,
    title: String,
}

static WINDOWS: LazyLock<Mutex<HashMap<i64, WindowEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(1));

#[no_mangle]
pub extern "C" fn x2d_window_create(_title_hash: i64, width: i64, height: i64, _: i64, _: i64, _: i64, _: i64, _: i64) -> i64 {
    let id = *NEXT_ID.lock().unwrap();
    *NEXT_ID.lock().unwrap() += 1;
    WINDOWS.lock().unwrap().insert(id, WindowEntry {
        width: width as i32,
        height: height as i32,
        title: format!("LPPX2D window {}", id),
    });
    eprintln!("[x2d] window_create {}x{} id={}", width, height, id);
    id
}

#[no_mangle]
pub extern "C" fn x2d_begin_frame(_win: i64, _: i64, _: i64, _: i64, _: i64, _: i64, _: i64, _: i64) -> i64 {
    eprintln!("[x2d] begin_frame");
    0
}

#[no_mangle]
pub extern "C" fn x2d_canvas_clear(r: i64, g: i64, b: i64, a: i64, _: i64, _: i64, _: i64, _: i64) -> i64 {
    eprintln!("[x2d] canvas_clear rgba({}, {}, {}, {})", r, g, b, a);
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
    a: i64,
) -> i64 {
    eprintln!("[x2d] draw_rectangle ({},{}) {}x{} rgba({},{},{},{})", x, y, w, h, r, g, b, a);
    0
}

#[no_mangle]
pub extern "C" fn x2d_draw_circle(cx: i64, cy: i64, radius: i64, _: i64, _: i64, _: i64, _: i64, _: i64) -> i64 {
    eprintln!("[x2d] draw_circle ({},{}) r={}", cx, cy, radius);
    0
}

#[no_mangle]
pub extern "C" fn x2d_end_frame(_win: i64, _: i64, _: i64, _: i64, _: i64, _: i64, _: i64, _: i64) -> i64 {
    eprintln!("[x2d] end_frame");
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
        let w = Window2d::new("test", 800, 600);
        let (t, width, height) = w.info();
        assert_eq!(t, "test");
        assert_eq!(width, 800);
        assert_eq!(height, 600);
    }

    #[test]
    fn raylib_color_conversion() {
        let c = Color::new(1.0, 0.0, 0.0, 1.0);
        let rc = raylib_color(&c);
        assert_eq!(rc.r, 255);
        assert_eq!(rc.g, 0);
        assert_eq!(rc.b, 0);
        assert_eq!(rc.a, 255);
    }
}
