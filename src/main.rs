extern crate sdl2;
extern crate image;

use std::time::{Duration, Instant, SystemTime};

use gfx_core::format::{Format, SurfaceType, ChannelType};

use ggez::{Context, ContextBuilder, GameResult};
use ggez::conf::NumSamples;
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Drawable, DrawParam, Rect, Mesh, BlendMode, DrawMode, Canvas, Color};
use ggez::timer;

mod fractals;
use fractals::Fractal;
use fractals::Rectangle;


#[allow(unused_imports)]
use num::Complex;

struct App {
    width: u32,
    height: u32,

    mouse: (f64, f64),
    scroll_scale: f64,

    draw: bool,
    generate: bool,

    fractal: Fractal,

    selecting: bool,
    selection: Rectangle<f64>,
    selection_center: (f64, f64),

    zoom_view: Rectangle<f64>,
}

impl App {
    pub fn new(ctx: &mut Context, width: u32, height: u32, scroll_scale: f64) -> App {
        /*let mut fractal = Fractal::with_coefficients(
            (width as usize, height as usize),
            Rect { left: -5f64, top: -5f64, right: 5f64, bottom: 5f64 },
            vec![
                Complex::new(-0.2796455185190574, -8.619337302126723),
                Complex::new(7.591418031049244, 4.167755685364256),
                Complex::new(-9.121138413779903, -6.79613957297315),
                Complex::new(9.197246762941262, 8.190568781916397),
                Complex::new(5.366325985514713, -1.1587722090698378),
            ]
        );*/

        App {
            width,
            height,

            mouse: (0f64, 0f64),
            scroll_scale,

            draw: true,
            generate: true,

            fractal: Fractal::with_random_coefficients(
                ctx,
                (width as usize, height as usize),
                Rectangle { left: -5f64, top: -5f64, right: 5f64, bottom: 5f64 },
                4
            ),
            /*
            Fractal::with_coefficients(
                (width as usize, height as usize),
                Rect { left: -5f64, top: -5f64, right: 5f64, bottom: 5f64 },
                vec![
                    Complex::new(-0.2796455185190574, -8.619337302126723),
                    Complex::new(7.591418031049244, 4.167755685364256),
                    Complex::new(-9.121138413779903, -6.79613957297315),
                    Complex::new(9.197246762941262, 8.190568781916397),
                    Complex::new(5.366325985514713, -1.1587722090698378),
                ]
            )
            */

            selecting: false,
            selection: Rectangle { left: 0f64, top: 0f64, right: 0f64, bottom: 0f64 },
            selection_center: (0f64, 0f64),

            zoom_view: Rectangle { left: 0f64, top: 0f64, right: width as f64, bottom: height as f64 },
        }
    }
}

impl EventHandler for App {
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Q => {
                event::quit(ctx);
            },
            KeyCode::G => {
                self.generate = true;
            },
            KeyCode::R => {
                self.zoom_view = Rectangle { left: 0f64, top: 0f64, right: self.width as f64, bottom: self.height as f64 };
            },
            KeyCode::S => {
                let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(ts) => ts.as_nanos(),
                    _ => 0
                };
                self.fractal.save_to_file(format!("out_{}.png", timestamp).as_str());
            },
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            let (x, y) = (x as f64, y as f64);

            self.selecting = true;
            self.selection_center = (x, y);
            self.selection = Rectangle { left: x, top: y, right: x, bottom: y };
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.selecting = false;
            self.zoom_view = Rectangle {
                left: self.selection.left * (self.zoom_view.right - self.zoom_view.left)/(self.width as f64) + self.zoom_view.left,
                top: self.selection.top * (self.zoom_view.bottom - self.zoom_view.top)/(self.height as f64) + self.zoom_view.top,
                right: self.selection.right * (self.zoom_view.right - self.zoom_view.left)/(self.width as f64) + self.zoom_view.left,
                bottom: self.selection.bottom * (self.zoom_view.bottom - self.zoom_view.top)/(self.height as f64) + self.zoom_view.top
            };
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.mouse = (x as f64, y as f64);

        let offset = (self.mouse.0-self.selection_center.0).abs().max((self.mouse.1-self.selection_center.1).abs());
        self.selection = Rectangle {
            left: self.selection_center.0-offset,
            top: self.selection_center.1-offset,
            right: self.selection_center.0+offset,
            bottom: self.selection_center.1+offset
        };
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        let s = y as f64;
        let (x, y) = (
            (self.mouse.0 as f64)*(self.zoom_view.right-self.zoom_view.left)/(self.width as f64) + self.zoom_view.left,
            (self.mouse.1 as f64)*(self.zoom_view.bottom-self.zoom_view.top)/(self.height as f64) + self.zoom_view.top
        );

        self.selecting = false;
        let scale = self.scroll_scale*s;
        self.zoom_view = Rectangle {
            left: scale*x + (1f64-scale)*self.zoom_view.left,
            top: scale*y + (1f64-scale)*self.zoom_view.top,
            right: scale*x + (1f64-scale)*self.zoom_view.right,
            bottom: scale*y + (1f64-scale)*self.zoom_view.bottom
        };
    }

    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.generate {
            self.generate = false;

            self.fractal.scale_view(Rectangle {
                left: self.zoom_view.left/(self.width as f64),
                top: self.zoom_view.top/(self.height as f64),
                right: self.zoom_view.right/(self.width as f64),
                bottom: self.zoom_view.bottom/(self.height as f64)
            });

            // reset the zoom
            self.zoom_view = Rectangle { left: 0f64, top: 0f64, right: self.width as f64, bottom: self.height as f64 };

            self.fractal.generate();

            self.draw = true;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::Color { r: 0f32, g: 0f32, b: 0f32, a: 1f32});

        let fractal_canvas = Canvas::new(
            ctx,
            self.width as u16,
            self.height as u16,
            NumSamples::One,
            Format(
                SurfaceType::R8_G8_B8_A8,
                ChannelType::Uint,
            )
        ).expect("fractal canvas");

        graphics::set_canvas(ctx, Some(&fractal_canvas));

        /*
        -(self.zoom_view.left*(self.width as f64)/(self.zoom_view.right-self.zoom_view.left)) as i32,
        -(self.zoom_view.top*(self.height as f64)/(self.zoom_view.bottom-self.zoom_view.top)) as i32,
        ((self.width*self.width) as f64/(self.zoom_view.right-self.zoom_view.left)) as u32,
        ((self.height*self.height) as f64/(self.zoom_view.bottom-self.zoom_view.top)) as u32
        */

        graphics::draw(ctx, &self.fractal, DrawParam::default()).expect("drawn fractal");
        graphics::set_canvas(ctx, None);

        if self.selecting {
            let bounds = Rect::new(
                self.selection.left.min(self.selection.right) as f32,
                self.selection.top.min(self.selection.bottom) as f32,
                (self.selection.right - self.selection.left).abs() as f32,
                (self.selection.bottom - self.selection.top).abs() as f32
            );

            let selection_rect = Mesh::new_rectangle(ctx, DrawMode::stroke(1f32), bounds, Color::WHITE).expect("selection rectangle");
            graphics::draw(ctx, &selection_rect, DrawParam::default()).expect("drawn selection rectangle");
        }

        timer::yield_now();
        graphics::present(ctx)
    }
}

fn main() {
    let scroll_scale = 0.15f64;
    let width = 800;
    let height = 800;

    let (mut ctx, event_loop) = ContextBuilder::new("newtonian_rustsody", "kshpin").build().expect("context and event loop");
    let app = App::new(&mut ctx, width, height, scroll_scale);
    event::run(ctx, event_loop, app);
}
