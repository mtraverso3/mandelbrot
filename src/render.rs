use crate::color::{self, INTERIOR};
use image::{Rgb, RgbImage};
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
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

const ESCAPE_RADIUS_SQR: f64 = 100.0 * 100.0;
const LANES: usize = 8;
const CYCLE_CHECK_START: usize = 16;
const BASE_VIEW_WIDTH: f64 = 3.0;
const LIGHT_ANGLE_DEGREES: f64 = 45.0;

pub fn render(view: &Viewport, opts: &RenderOptions) -> RgbImage {
    let mut img = RgbImage::new(opts.width, opts.height);
    render_rows(view, opts, 0, &mut img);
    img
}

/// Renders the rows starting at `first_row` into `rows`, a packed RGB buffer holding a whole
/// number of rows of the full `opts.width` x `opts.height` image.
pub fn render_rows(view: &Viewport, opts: &RenderOptions, first_row: u32, rows: &mut [u8]) {
    let row_len = opts.width as usize * 3;
    if row_len == 0 || rows.is_empty() {
        return;
    }
    assert!(
        rows.len().is_multiple_of(row_len)
            && first_row as usize + rows.len() / row_len <= opts.height as usize,
        "row buffer must hold whole rows within the image"
    );
    match opts.shading {
        Shading::Flat => render_rows_with::<false>(view, opts, first_row, rows),
        Shading::Normal => render_rows_with::<true>(view, opts, first_row, rows),
    }
}

fn render_rows_with<const NORMAL: bool>(
    view: &Viewport,
    opts: &RenderOptions,
    first_row: u32,
    rows: &mut [u8],
) {
    let frame = Frame::new(view, opts);

    rows.par_chunks_mut(opts.width as usize * 3)
        .enumerate()
        .for_each(|(offset, row)| {
            let ci = frame.imag(first_row as usize + offset);
            for (block, pixels) in row.chunks_mut(3 * LANES).enumerate() {
                let cr = std::array::from_fn(|lane| frame.real(block * LANES + lane));
                let escapes = escape_lanes::<NORMAL>(&cr, ci, opts.max_iterations);
                let (pixels, _) = pixels.as_chunks_mut::<3>();
                for (pixel, escape) in pixels.iter_mut().zip(&escapes) {
                    *pixel = frame.color::<NORMAL>(escape.as_ref()).0;
                }
            }
        });
}

struct Frame {
    left: f64,
    top: f64,
    center_x: f64,
    center_y: f64,
    pixel_size: f64,
    band_scale: f64,
    light: (f64, f64),
}

impl Frame {
    fn new(view: &Viewport, opts: &RenderOptions) -> Self {
        let angle = LIGHT_ANGLE_DEGREES.to_radians();
        Self {
            left: opts.width as f64 / 2.0,
            top: opts.height as f64 / 2.0,
            center_x: view.center_x,
            center_y: view.center_y,
            pixel_size: BASE_VIEW_WIDTH / view.zoom / opts.width as f64,
            band_scale: (view.zoom + 1.0).log2(),
            light: (angle.cos(), angle.sin()),
        }
    }

    fn real(&self, x: usize) -> f64 {
        self.center_x + (x as f64 - self.left) * self.pixel_size
    }

    fn imag(&self, y: usize) -> f64 {
        self.center_y + (y as f64 - self.top) * self.pixel_size
    }

    fn color<const NORMAL: bool>(&self, escape: Option<&Escape>) -> Rgb<u8> {
        let Some(escape) = escape else {
            return INTERIOR;
        };
        let base = color::palette(self.smooth_position(escape));
        if NORMAL {
            color::shade(base, escape.normal(), self.light)
        } else {
            base
        }
    }

    fn smooth_position(&self, escape: &Escape) -> f64 {
        let log_modulus = escape.norm_sqr.ln() * 0.5;
        let nu = (log_modulus / std::f64::consts::LN_2).log2();
        (escape.iterations as f64 + 1.0 - nu) / self.band_scale
    }
}

#[derive(Clone, Copy)]
struct Escape {
    iterations: usize,
    norm_sqr: f64,
    z: (f64, f64),
    derivative: (f64, f64),
}

impl Escape {
    fn normal(&self) -> (f64, f64) {
        let (zr, zi) = self.z;
        let (dr, di) = self.derivative;
        let der_norm_sqr = dr * dr + di * di;
        let ur = (zr * dr + zi * di) / der_norm_sqr;
        let ui = (zi * dr - zr * di) / der_norm_sqr;
        let length = ur.hypot(ui);
        (ur / length, ui / length)
    }
}

type Lanes = [f64; LANES];

/// Iterates a block of horizontally adjacent pixels in lockstep so the compiler can vectorize
/// the arithmetic; every lane performs exactly the scalar operations.
struct Block {
    cr: Lanes,
    ci: Lanes,
    zr: Lanes,
    zi: Lanes,
    dr: Lanes,
    di: Lanes,
    checkpoint_r: Lanes,
    checkpoint_i: Lanes,
    active: usize,
}

impl Block {
    fn new(cr: &Lanes, ci: f64) -> Self {
        let mut block = Self {
            cr: *cr,
            ci: [ci; LANES],
            zr: [0.0; LANES],
            zi: [0.0; LANES],
            dr: [1.0; LANES],
            di: [0.0; LANES],
            checkpoint_r: [0.0; LANES],
            checkpoint_i: [0.0; LANES],
            active: LANES,
        };
        for lane in 0..LANES {
            if in_main_cardioid_or_bulb(block.cr[lane], block.ci[lane]) {
                block.retire(lane);
            }
        }
        block
    }

    /// Parks a finished lane on the fixed point z = 0 with a NaN checkpoint, so it never
    /// triggers the escape or cycle checks again.
    fn retire(&mut self, lane: usize) {
        debug_assert!(self.is_active(lane));
        self.cr[lane] = 0.0;
        self.ci[lane] = 0.0;
        self.zr[lane] = 0.0;
        self.zi[lane] = 0.0;
        self.checkpoint_r[lane] = f64::NAN;
        self.active -= 1;
    }

    fn is_active(&self, lane: usize) -> bool {
        !self.checkpoint_r[lane].is_nan()
    }

    fn is_cycling(&self, lane: usize) -> bool {
        self.zr[lane] == self.checkpoint_r[lane] && self.zi[lane] == self.checkpoint_i[lane]
    }
}

fn escape_lanes<const TRACK_DERIVATIVE: bool>(
    cr: &Lanes,
    ci: f64,
    max_iterations: usize,
) -> [Option<Escape>; LANES] {
    let mut escapes = [None; LANES];
    let mut b = Block::new(cr, ci);
    let mut next_checkpoint = CYCLE_CHECK_START;

    for n in 0..max_iterations {
        if b.active == 0 {
            break;
        }
        let mut zr2: Lanes = std::array::from_fn(|i| b.zr[i] * b.zr[i]);
        let mut zi2: Lanes = std::array::from_fn(|i| b.zi[i] * b.zi[i]);
        let norm_sqr: Lanes = std::array::from_fn(|i| zr2[i] + zi2[i]);

        if norm_sqr.iter().any(|&r| r > ESCAPE_RADIUS_SQR) {
            for lane in 0..LANES {
                if norm_sqr[lane] > ESCAPE_RADIUS_SQR {
                    escapes[lane] = Some(Escape {
                        iterations: n,
                        norm_sqr: norm_sqr[lane],
                        z: (b.zr[lane], b.zi[lane]),
                        derivative: (b.dr[lane], b.di[lane]),
                    });
                    b.retire(lane);
                    (zr2[lane], zi2[lane]) = (0.0, 0.0);
                }
            }
        }

        if TRACK_DERIVATIVE {
            for i in 0..LANES {
                let (dr2, di2) = (b.dr[i] * 2.0, b.di[i] * 2.0);
                b.dr[i] = dr2 * b.zr[i] - di2 * b.zi[i] + 1.0;
                b.di[i] = dr2 * b.zi[i] + di2 * b.zr[i];
            }
        }
        for i in 0..LANES {
            let zi = b.zr[i] * b.zi[i] + b.zi[i] * b.zr[i] + b.ci[i];
            b.zr[i] = zr2[i] - zi2[i] + b.cr[i];
            b.zi[i] = zi;
        }

        // An exactly repeated state means the orbit is periodic and can never escape.
        if (0..LANES).any(|lane| b.is_cycling(lane)) {
            for lane in 0..LANES {
                if b.is_cycling(lane) {
                    b.retire(lane);
                }
            }
        }
        if n == next_checkpoint {
            for i in 0..LANES {
                if b.is_active(i) {
                    b.checkpoint_r[i] = b.zr[i];
                    b.checkpoint_i[i] = b.zi[i];
                }
            }
            next_checkpoint *= 2;
        }
    }
    escapes
}

fn in_main_cardioid_or_bulb(cr: f64, ci: f64) -> bool {
    let ci2 = ci * ci;
    let shifted = cr - 0.25;
    let q = shifted * shifted + ci2;
    let in_cardioid = q * (q + shifted) <= 0.25 * ci2;
    let in_period2_bulb = (cr + 1.0) * (cr + 1.0) + ci2 <= 0.0625;
    in_cardioid || in_period2_bulb
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_escape(cr: f64, ci: f64, max_iterations: usize) -> Option<(usize, f64)> {
        let (mut zr, mut zi) = (0.0f64, 0.0f64);
        for n in 0..max_iterations {
            let norm_sqr = zr * zr + zi * zi;
            if norm_sqr > ESCAPE_RADIUS_SQR {
                return Some((n, norm_sqr));
            }
            (zr, zi) = (zr * zr - zi * zi + cr, zr * zi + zi * zr + ci);
        }
        None
    }

    fn assert_matches_reference(view: Viewport, width: u32, height: u32) {
        let opts = RenderOptions {
            width,
            height,
            max_iterations: 500,
            shading: Shading::Flat,
        };
        let frame = Frame::new(&view, &opts);
        for y in 0..height as usize {
            let ci = frame.imag(y);
            for x0 in (0..width as usize).step_by(LANES) {
                let cr = std::array::from_fn(|lane| frame.real(x0 + lane));
                let escapes = escape_lanes::<true>(&cr, ci, opts.max_iterations);
                for (lane, escape) in escapes.iter().enumerate() {
                    let actual = escape.map(|e| (e.iterations, e.norm_sqr));
                    let expected = reference_escape(cr[lane], ci, opts.max_iterations);
                    assert_eq!(actual, expected, "pixel ({}, {y})", x0 + lane);
                }
            }
        }
    }

    #[test]
    fn lanes_match_scalar_iteration_on_full_view() {
        assert_matches_reference(
            Viewport {
                center_x: -0.75,
                center_y: 0.0,
                zoom: 1.0,
            },
            67,
            53,
        );
    }

    #[test]
    fn lanes_match_scalar_iteration_on_deep_zoom() {
        let view = Viewport {
            center_x: -1.249559196,
            center_y: 0.030466443,
            zoom: 1.73e6,
        };
        assert_matches_reference(view, 45, 37);
    }

    #[test]
    fn cardioid_and_bulb_points_never_escape() {
        for (cr, ci) in [
            (0.0, 0.0),
            (-0.5, 0.3),
            (0.2, 0.0),
            (-1.0, 0.0),
            (-1.1, 0.1),
        ] {
            assert!(in_main_cardioid_or_bulb(cr, ci));
            assert_eq!(reference_escape(cr, ci, 5000), None);
        }
        for (cr, ci) in [(0.3, 0.0), (-0.75, 0.2), (-1.3, 0.0), (0.0, 1.0)] {
            assert!(!in_main_cardioid_or_bulb(cr, ci));
        }
    }

    #[test]
    fn render_produces_requested_dimensions() {
        let opts = RenderOptions {
            width: 21,
            height: 9,
            max_iterations: 50,
            shading: Shading::Normal,
        };
        let img = render(
            &Viewport {
                center_x: -0.75,
                center_y: 0.0,
                zoom: 1.0,
            },
            &opts,
        );
        assert_eq!(img.dimensions(), (21, 9));
    }

    #[test]
    fn row_bands_match_full_render() {
        let view = Viewport {
            center_x: -0.7453,
            center_y: 0.1127,
            zoom: 150.0,
        };
        let opts = RenderOptions {
            width: 37,
            height: 23,
            max_iterations: 400,
            shading: Shading::Normal,
        };
        let full = render(&view, &opts);
        let row_len = opts.width as usize * 3;
        let mut stitched = Vec::new();
        for (first_row, rows) in [(0, 5), (5, 1), (6, 10), (16, 7)] {
            let mut band = vec![0; row_len * rows];
            render_rows(&view, &opts, first_row, &mut band);
            stitched.extend(band);
        }
        assert_eq!(stitched, full.into_raw());
    }

    #[test]
    #[should_panic(expected = "whole rows")]
    fn render_rows_rejects_partial_rows() {
        let opts = RenderOptions {
            width: 4,
            height: 4,
            ..RenderOptions::default()
        };
        render_rows(
            &Viewport {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
            },
            &opts,
            0,
            &mut [0; 5],
        );
    }

    #[test]
    fn render_handles_empty_images() {
        let opts = RenderOptions {
            width: 0,
            height: 5,
            ..RenderOptions::default()
        };
        let img = render(
            &Viewport {
                center_x: 0.0,
                center_y: 0.0,
                zoom: 1.0,
            },
            &opts,
        );
        assert_eq!(img.dimensions(), (0, 5));
    }
}
