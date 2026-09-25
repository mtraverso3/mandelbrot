use image::{Rgb, RgbImage};
use num::Complex;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shading {
    Flat,
    Normal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub max_iterations: usize,
    pub shading: Shading,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 4096,
            height: 3280,
            max_iterations: 1500,
            shading: Shading::Normal,
        }
    }
}

// Normal map parameters
const HEIGHT_FACTOR: f64 = 1.0; // h2 from wiki
const LIGHT_ANGLE: f64 = 45.0; // degrees
const ESCAPE_RADIUS: f64 = 100.0;

pub fn render(view: &Viewport, opts: &RenderOptions) -> RgbImage {
    let mut img = RgbImage::new(opts.width, opts.height);
    let mut rows: Vec<&mut [u8]> = img.as_mut().chunks_mut(opts.width as usize * 3).collect();

    // Pre-calculate light direction vector
    let angle_rad = LIGHT_ANGLE * 2.0 * std::f64::consts::PI / 360.0;
    let light_v = Complex::new(angle_rad.cos(), angle_rad.sin());

    rows.par_iter_mut().enumerate().for_each(|(y, row)| {
        for x in 0..opts.width as usize {
            let (re, im) = scale_pixel(x as u32, y as u32, view, opts);
            let c = Complex::new(re, im);

            let color = match opts.shading {
                Shading::Normal => {
                    mandelbrot_color_with_normal(c, view.zoom, &light_v, opts.max_iterations)
                }
                Shading::Flat => mandelbrot_color(c, view.zoom, opts.max_iterations),
            };
            let offset = x * 3;
            row[offset] = color[0]; // Red
            row[offset + 1] = color[1]; // Green
            row[offset + 2] = color[2]; // Blue
        }
    });

    img
}

fn scale_pixel(x: u32, y: u32, view: &Viewport, opts: &RenderOptions) -> (f64, f64) {
    let x = x as f64;
    let y = y as f64;
    let image_width = opts.width as f64;
    let image_height = opts.height as f64;

    // Base scale factor (3.0 is the initial view width)
    let scale = 3.0 / view.zoom / image_width;

    // Calculate coordinates relative to center
    let scaled_x = view.center_x + (x - image_width / 2.0) * scale;
    let scaled_y = view.center_y + (y - image_height / 2.0) * scale;

    (scaled_x, scaled_y)
}

fn get_color(n: usize) -> Rgb<u8> {
    let mapping: [Rgb<u8>; 16] = [
        Rgb([66, 30, 15]),
        Rgb([25, 7, 26]),
        Rgb([9, 1, 47]),
        Rgb([4, 4, 73]),
        Rgb([0, 7, 100]),
        Rgb([12, 44, 138]),
        Rgb([24, 82, 177]),
        Rgb([57, 125, 209]),
        Rgb([134, 181, 229]),
        Rgb([211, 236, 248]),
        Rgb([241, 233, 191]),
        Rgb([248, 201, 95]),
        Rgb([255, 170, 0]),
        Rgb([204, 128, 0]),
        Rgb([153, 87, 0]),
        Rgb([106, 52, 3]),
    ];
    mapping[(n) % 16]
}

fn mandelbrot_color(c: Complex<f64>, zoom: f64, max_iterations: usize) -> Rgb<u8> {
    let mut z: Complex<f64> = Complex { re: 0.0, im: 0.0 };
    for n in 0..max_iterations {
        let r2 = z.norm_sqr();
        if r2 > ESCAPE_RADIUS * ESCAPE_RADIUS {
            let log_zn = r2.ln() * 0.5;
            let nu = (log_zn / 2.0f64.ln()).log2();
            let iteration = (n as f64 + 1.0 - nu) / (zoom + 1.0).log2();

            let color1 = get_color(iteration as usize);
            let color2 = get_color(iteration as usize + 1);
            return linear_interpolate(color1, color2, iteration % 1.0);
        }
        z = z * z + c;
    }
    Rgb([255, 255, 255])
}

const AMBIENT_LIGHT: f64 = 0.3; // Value between 0 and 1 for minimum brightness

fn mandelbrot_color_with_normal(
    c: Complex<f64>,
    zoom: f64,
    light_v: &Complex<f64>,
    max_iterations: usize,
) -> Rgb<u8> {
    let mut z = Complex::new(0.0, 0.0);
    let mut der = Complex::new(1.0, 0.0);

    for n in 0..max_iterations {
        let r2 = z.norm_sqr();
        if r2 > ESCAPE_RADIUS * ESCAPE_RADIUS {
            // Calculate normal vector
            let u = z / der;
            let u = u / u.norm();

            // Calculate lighting (dot product)
            let t = u.re * light_v.re + u.im * light_v.im + HEIGHT_FACTOR;
            let t = t / (1.0 + HEIGHT_FACTOR); // Rescale

            // Apply ambient light and clamp
            let light_factor = (t * (1.0 - AMBIENT_LIGHT) + AMBIENT_LIGHT)
                .max(0.0)
                .min(1.0);

            // Get base color
            let log_zn = r2.ln() * 0.5;
            let nu = (log_zn / 2.0f64.ln()).log2();
            let iteration = (n as f64 + 1.0 - nu) / (zoom + 1.0).log2();

            let color1 = get_color(iteration as usize);
            let color2 = get_color(iteration as usize + 1);
            let base_color = linear_interpolate(color1, color2, iteration % 1.0);

            // Apply lighting with ambient component
            let brightness_boost = 1.3;
            return Rgb([
                ((base_color[0] as f64 * light_factor * brightness_boost) as u32).min(255) as u8,
                ((base_color[1] as f64 * light_factor * brightness_boost) as u32).min(255) as u8,
                ((base_color[2] as f64 * light_factor * brightness_boost) as u32).min(255) as u8,
            ]);
        }

        der = der * 2.0 * z + 1.0;
        z = z * z + c;
    }

    Rgb([255, 255, 255])
}

fn linear_interpolate(color1: Rgb<u8>, color2: Rgb<u8>, t: f64) -> Rgb<u8> {
    let r = color1[0] as f64 * (1.0 - t) + color2[0] as f64 * t;
    let g = color1[1] as f64 * (1.0 - t) + color2[1] as f64 * t;
    let b = color1[2] as f64 * (1.0 - t) + color2[2] as f64 * t;
    Rgb([r as u8, g as u8, b as u8])
}

pub fn downsample(img: &RgbImage) -> RgbImage {
    image::imageops::resize(
        img,
        img.width() / 2,
        img.height() / 2,
        image::imageops::FilterType::Lanczos3,
    )
}
