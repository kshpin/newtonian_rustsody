extern crate sdl2;
extern crate image;

mod fractals;

use std::time::{Duration, Instant, SystemTime};

use fractals::Fractal;
use fractals::Rect;

use sdl2::rect::Rect as DrawRect;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{MouseButton, MouseState};

#[allow(unused_imports)]
use num::Complex;

fn main() {
    let max_sleep_duration = Duration::new(0, 1_000_000_000u32/60); // for 60fps, in nanos
    let scroll_scale = 0.15f64;
    let width = 800;
    let height = 800;

    // region window setup
    let sdl_context = sdl2::init().expect("sdl context");
    let mut events = sdl_context.event_pump().expect("event pump");
    let video_subsystem = sdl_context.video().expect("video subsystem");

    let window = video_subsystem.window("Newtonian Rustsody", width, height)
        .position_centered()
        .build()
        .expect("window");

    let mut canvas = window.into_canvas().present_vsync().build().expect("canvas");

    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_streaming(Some(PixelFormatEnum::RGB24), width, height)
        .expect("texture");

    let clear_color = Color::RGB(0, 0, 0);
    let white_color = Color::RGB(255, 255, 255);

    canvas.set_draw_color(Color::RGB(0, 255, 255)); // cyan so it stands out
    canvas.clear();
    canvas.present();
    // endregion window setup

    let mut pixels = [0u8; 800*800*3];
    let mut fractal = Fractal::with_coefficients(
        (width as usize, height as usize),
        Rect { left: -5f64, top: -5f64, right: 5f64, bottom: 5f64 },
        vec![
            Complex::new(-0.2796455185190574, -8.619337302126723),
            Complex::new(7.591418031049244, 4.167755685364256),
            Complex::new(-9.121138413779903, -6.79613957297315),
            Complex::new(9.197246762941262, 8.190568781916397),
            Complex::new(5.366325985514713, -1.1587722090698378),
        ]
    );

    let mut draw;

    let mut generate = true;
    let mut selecting = false;
    let mut selection = Rect { left: 0f64, top: 0f64, right: 0f64, bottom: 0f64 };
    let mut selection_center = (0f64, 0f64);

    let mut zoom_view = Rect { left: 0f64, top: 0f64, right: width as f64, bottom: height as f64 };

    'running: loop {
        let beginning = Instant::now();

        draw = true;
        let mouse = MouseState::new(&events);
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::G), .. } => {
                    generate = true;
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    zoom_view = Rect { left: 0f64, top: 0f64, right: width as f64, bottom: height as f64 };
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(ts) => ts.as_nanos(),
                        _ => 0
                    };

                    image::save_buffer(
                        format!("out/out_{}.png", timestamp),
                        &pixels,
                        800, 800,
                        image::ColorType::Rgb8
                    ).expect("saved image");
                },
                Event::MouseWheel { y, .. } => {
                    let s = y as f64;
                    let (x, y) = (
                        (mouse.x() as f64)*(zoom_view.right-zoom_view.left)/(width as f64) + zoom_view.left,
                        (mouse.y() as f64)*(zoom_view.bottom-zoom_view.top)/(height as f64) + zoom_view.top
                    );

                    selecting = false;
                    let scale = scroll_scale*s;
                    zoom_view = Rect {
                        left: scale*x + (1f64-scale)*zoom_view.left,
                        top: scale*y + (1f64-scale)*zoom_view.top,
                        right: scale*x + (1f64-scale)*zoom_view.right,
                        bottom: scale*y + (1f64-scale)*zoom_view.bottom
                    };
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    let (x, y) = (x as f64, y as f64);

                    selecting = true;
                    selection_center = (x, y);
                    selection = Rect { left: x, top: y, right: x, bottom: y };
                },
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => {
                    selecting = false;
                    zoom_view = Rect {
                        left: selection.left * (zoom_view.right - zoom_view.left)/(width as f64) + zoom_view.left,
                        top: selection.top * (zoom_view.bottom - zoom_view.top)/(height as f64) + zoom_view.top,
                        right: selection.right * (zoom_view.right - zoom_view.left)/(width as f64) + zoom_view.left,
                        bottom: selection.bottom * (zoom_view.bottom - zoom_view.top)/(height as f64) + zoom_view.top
                    };
                },
                Event::MouseMotion { x, y, .. } => {
                    let (x, y) = (x as f64, y as f64);

                    let offset = (x-selection_center.0).abs().max((y-selection_center.1).abs());
                    selection = Rect {
                        left: selection_center.0-offset,
                        top: selection_center.1-offset,
                        right: selection_center.0+offset,
                        bottom: selection_center.1+offset
                    };
                },
                _ => {
                    draw = false;
                }
            }
        }

        // region update

        if generate {
            generate = false;

            fractal.scale_view(Rect {
                left: zoom_view.left/(width as f64),
                top: zoom_view.top/(height as f64),
                right: zoom_view.right/(width as f64),
                bottom: zoom_view.bottom/(height as f64)
            });

            // reset the zoom
            zoom_view = Rect { left: 0f64, top: 0f64, right: width as f64, bottom: height as f64 };

            fractal.generate(&mut pixels);
            texture.update(None, &pixels, (width * 3) as usize).expect("update");
        }

        // endregion update

        if draw {
            canvas.set_draw_color(clear_color);
            canvas.clear();

            // region draw content --------------------

            canvas.copy(
                &texture,
                None,
                DrawRect::new(
                    -(zoom_view.left*(width as f64)/(zoom_view.right-zoom_view.left)) as i32,
                    -(zoom_view.top*(height as f64)/(zoom_view.bottom-zoom_view.top)) as i32,
                    ((width*width) as f64/(zoom_view.right-zoom_view.left)) as u32,
                    ((height*height) as f64/(zoom_view.bottom-zoom_view.top)) as u32
                )
            ).expect("copy");

            if selecting {
                canvas.set_draw_color(white_color);
                canvas.draw_rect(DrawRect::new(
                    selection.left.min(selection.right) as i32,
                    selection.top.min(selection.bottom) as i32,
                    (selection.right - selection.left).abs() as u32,
                    (selection.bottom - selection.top).abs() as u32
                )).expect("rectangle");
            }

            // endregion draw content -----------------

            canvas.present();
        }

        let time_passed = Instant::now().duration_since(beginning);
        if time_passed < max_sleep_duration {
            std::thread::sleep(max_sleep_duration - time_passed);
        }
    }
}
