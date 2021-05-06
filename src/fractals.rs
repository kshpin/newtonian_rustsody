use rand::Rng;
use num::complex::Complex;
use num::pow;

use std::time::Instant;

use ggez::{Context, GameResult};
use ggez::graphics::{self, Drawable, DrawParam, Image, Rect, BlendMode};

#[derive(Debug)]
pub struct Rectangle<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

pub struct Fractal {
    size: (usize, usize),
    view: Rectangle<f64>,

    coefficients: Vec<Complex<f64>>,

    roots: Vec<Complex<f64>>,

    pixels: Vec<u8>,
}

impl Fractal {
    const DIFF_STEP: f64 = 1e-8;
    const TOLERANCE: f64 = 1e-4;
    const MAX_ITERS: u32 = 100;

    #[allow(dead_code)]
    pub fn with_random_coefficients(_ctx: &mut Context, size: (usize, usize), view: Rectangle<f64>, degree: u32) -> Fractal {
        let mut coefficients = Vec::with_capacity((degree+1) as usize);

        let mut rng = rand::thread_rng();

        for _ in 0..degree+1 {
            coefficients.push(Complex::new(rng.gen_range(-10f64, 10f64), rng.gen_range(-10f64, 10f64)));
        }

        println!("{:#?}", coefficients);

        Fractal {
            size,
            view,
            coefficients,
            roots: Vec::new(),
            pixels: vec![0u8; size.0*size.1*4],
        }
    }

    #[allow(dead_code)]
    pub fn with_coefficients(_ctx: &mut Context, size: (usize, usize), view: Rectangle<f64>, coefficients: Vec<Complex<f64>>) -> Fractal {
        Fractal {
            size,
            view,
            coefficients,
            roots: Vec::new(),
            pixels: vec![0u8; size.0*size.1*4],
        }
    }

    #[allow(dead_code)]
    pub fn set_view(&mut self, view: Rectangle<f64>) {
        self.view = view;
    }

    #[allow(dead_code)]
    pub fn scale_view(&mut self, scale: Rectangle<f64>) {
        let width = self.view.right - self.view.left;
        let height = self.view.bottom - self.view.top;

        self.view = Rectangle {
            left: self.view.left + scale.left*width,
            top: self.view.top + scale.top*height,
            right: self.view.left + scale.right*width,
            bottom: self.view.top + scale.bottom*height,
        };
    }

    pub fn save_to_file(&self, filename: &str) {
        image::save_buffer(
            format!("out/{}", filename),
            &self.pixels,
            800, 800,
            image::ColorType::Rgb8
        ).expect("saved image");
    }

    fn source(&self, z: Complex<f64>) -> Complex<f64> {
        let mut result = Complex::new(0f64, 0f64);

        for (i, c) in self.coefficients.iter().enumerate() {
            result += c*pow(z, i);
        }

        result
    }

    fn source_deriv_reciprocal(&self, z: Complex<f64>) -> Complex<f64> {
        Self::DIFF_STEP / (self.source(z+Self::DIFF_STEP) - self.source(z))
    }

    fn get_root(&self, s: Complex<f64>) -> Option<(Complex<f64>, u32)> {
        let mut z = s;
        let mut z_prev;

        for i in 0..Self::MAX_ITERS {
            z_prev = z;
            z = z - /*Self::a **/ self.source(z) * self.source_deriv_reciprocal(z);

            if (z-z_prev).norm_sqr() < Self::TOLERANCE*Self::TOLERANCE {
                return Some((z, i));
            }
        }

        None
    }

    pub fn generate(&mut self) {
        let x_scale = (self.view.right-self.view.left) / (self.size.0 as f64);
        let y_scale = (self.view.bottom-self.view.top) / (self.size.1 as f64);

        let mut candidates: Vec<Option<(usize, u32)>> = Vec::with_capacity(self.size.0*self.size.1);
        //let mut roots: Vec<Complex<f64>> = Vec::new();

        let beginning = Instant::now();

        for y in 0..self.size.1 {
            for x in 0..self.size.0 {
                let candidate = self.get_root(Complex::new(
                    (x as f64)*x_scale + self.view.left,
                    (y as f64)*y_scale + self.view.top)
                );

                if let Some((root, iters)) = candidate {
                    let mut exists = false;
                    for (index, r) in self.roots.iter().enumerate() {
                        if (root-r).norm_sqr() < 4f64*Self::TOLERANCE*Self::TOLERANCE {
                            exists = true;
                            candidates.push(Some((index, iters))); // this root already found
                            break;
                        }
                    }

                    if !exists {
                        candidates.push(Some((self.roots.len(), iters))); // this root is new
                        self.roots.push(root);
                    }
                } else {
                    candidates.push(None); // doesn't converge
                }
            }
        }

        println!("roots: {}", Instant::now().duration_since(beginning).as_micros());
        let beginning = Instant::now();

        //let colors: Vec<(u8, u8, u8)> = vec![(0xfe, 0xc4, 0x18), (0x06, 0xb6, 0xef), (0x81, 0x5b, 0xa4), (0x5b, 0xc4, 0xbf)]; // forgot
        //let colors: Vec<(u8, u8, u8)> = vec![(0xcf, 0x6a, 0x4c), (0x8f, 0x9d, 0x6a), (0x75, 0x87, 0xa6), (0x9b, 0x85, 0x9d)]; // dusty
        let colors: Vec<(u8, u8, u8)> = vec![(0xcf, 0x6a, 0x4c), (0xf9, 0xee, 0x98), (0x75, 0x87, 0xa6), (0x9b, 0x85, 0x9d)]; // yellow dusty
        //let colors: Vec<(u8, u8, u8)> = vec![(0xd7, 0x37, 0x37), (0x51, 0x6a, 0xec), (0xb8, 0x54, 0xd4), (0x7b, 0x59, 0xc0)]; // red sinister
        //let colors: Vec<(u8, u8, u8)> = vec![(0xe5, 0x8b, 0xf2), (0x6a, 0xdb, 0xde), (0x9d, 0x83, 0xf0), (0x9b, 0x85, 0x9d)]; // grape popsicle (first 3)
        //let colors: Vec<(u8, u8, u8)> = vec![(0xe5, 0x8b, 0xf2), (0x6a, 0xdb, 0xde), (0x9d, 0x83, 0xf0), (0x9b, 0x85, 0x9d)]; // grape popsicle (first 3)
        //let colors: Vec<(u8, u8, u8)> = vec![(0xcf, 0x6a, 0x4c), (0xf9, 0xee, 0x98), (0x75, 0x87, 0xa6), (0x9b, 0x85, 0x9d), (0x5f, 0x5a, 0x60)]; // yellow dusty
        //let colors: Vec<(u8, u8, u8)> = vec![(0xff, 0xb3, 0x3c), (0xfa, 0xe6, 0x70), (0xcc, 0xeb, 0x61), (0xff, 0x9a, 0x81), (0x8d, 0xe9, 0x87)]; // candymelon

        /*let num_colors = roots.len();
        let mut colors: Vec<(u8, u8, u8)> = Vec::with_capacity(num_colors);

        let init_angle = 0.4f64;
        let spread = 0.4f64;

        for i in 0..num_colors {
            colors.push(hsv_to_rgb((spread*(i as f64)/(num_colors as f64) + init_angle, 0.8f64, 255f64)));
        }*/

        let mut pixel_index = 0;
        for c in &candidates {
            match c {
                None => {
                    self.pixels[pixel_index    ] = 0;
                    self.pixels[pixel_index + 1] = 0;
                    self.pixels[pixel_index + 2] = 0;
                },
                Some((root_index, iters)) => {
                    let dist = (-4f64 * (*iters as f64) / (Self::MAX_ITERS as f64)).exp();

                    let color = colors[*root_index];

                    self.pixels[pixel_index    ] = (dist*(color.0 as f64)) as u8;
                    self.pixels[pixel_index + 1] = (dist*(color.1 as f64)) as u8;
                    self.pixels[pixel_index + 2] = (dist*(color.2 as f64)) as u8;
                }
            }

            self.pixels[pixel_index + 3] = 1; // alpha

            pixel_index += 4;
        }

        println!("texture: {}", Instant::now().duration_since(beginning).as_micros());
    }
}

impl Drawable for Fractal {
    fn draw(&self, ctx: &mut Context, param: DrawParam) -> GameResult {
        let image = Image::from_rgba8(ctx, self.size.0 as u16, self.size.1 as u16, &self.pixels).expect("fractal image");
        graphics::draw(ctx, &image, param)
    }

    fn dimensions(&self, _ctx: &mut Context) -> Option<Rect> {
        Some(Rect::new(0f32, 0f32, self.size.0 as f32, self.size.1 as f32))
    }

    fn set_blend_mode(&mut self, _mode: Option<BlendMode>) {
        // TODO: figure out what to do here
    }

    fn blend_mode(&self) -> Option<BlendMode> {
        // TODO: figure out what to do here
        None
    }
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn hsv_to_rgb(hsv: (f64, f64, f64)) -> (u8, u8, u8) {
    let (H, S, V) = hsv;

    let C = S * V;

    let part = 6.0*(H%1f64);
    let X = C*(1f64 - ((part%2f64) - 1f64).abs());

    let mut color = (0f64, 0f64, 0f64);

    if part < 1f64 {
        color.0 = C; color.1 = X;
    } else if part < 2f64 {
        color.0 = X; color.1 = C;
    } else if part < 3f64 {
        color.1 = C; color.2 = X;
    } else if part < 4f64 {
        color.1 = X; color.2 = C;
    } else if part < 5f64 {
        color.0 = X; color.2 = C;
    } else {
        color.0 = C; color.2 = X;
    }

    let m = V-C;
    color.0 += m;
    color.1 += m;
    color.2 += m;

    (color.0 as u8, color.1 as u8, color.2 as u8)
}
