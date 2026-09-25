use crate::Coordinate;
use crate::color::{self, INTERIOR};
use crate::perturbation::ReferenceOrbit;
use image::{Rgb, RgbImage};
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Viewport {
    pub center_x: Coordinate,
    pub center_y: Coordinate,
    pub zoom: f64,
}

impl Viewport {
    pub fn from_f64(center_x: f64, center_y: f64, zoom: f64) -> Self {
        Self {
            center_x: Coordinate::from_f64(center_x),
            center_y: Coordinate::from_f64(center_y),
            zoom,
        }
    }

    /// Moves the center by `(dx, dy)` in the complex plane.
    pub fn pan(&self, dx: f64, dy: f64) -> Self {
        let scale = center_digits(self.zoom);
        Self {
            center_x: self.center_x.offset(dx, scale),
            center_y: self.center_y.offset(dy, scale),
            zoom: self.zoom,
        }
    }

    /// Zooms by `factor` (clamped to [`MAX_ZOOM`]) around the point `(dx, dy)` away from the
    /// center, which stays in place on screen.
    pub fn zoom_at(&self, dx: f64, dy: f64, factor: f64) -> Self {
        let zoom = (self.zoom * factor).min(MAX_ZOOM);
        let keep = 1.0 - self.zoom / zoom;
        let scale = center_digits(zoom);
        Self {
            center_x: self.center_x.offset(dx * keep, scale),
            center_y: self.center_y.offset(dy * keep, scale),
            zoom,
        }
    }
}

/// Decimal places that keep the center well below a pixel at `zoom`, for images up to
/// 100,000 pixels wide.
fn center_digits(zoom: f64) -> u32 {
    zoom.max(1.0).log10().ceil() as u32 + 10
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

pub(crate) const ESCAPE_RADIUS_SQR: f64 = 100.0 * 100.0;
/// Beyond this zoom, `f64` pixel coordinates lose precision and rendering switches to
/// perturbation around a high-precision reference orbit.
pub const PERTURBATION_ZOOM: f64 = 1e10;
/// Deepest supported zoom: pixel offsets and derivatives must still fit in `f64`.
pub const MAX_ZOOM: f64 = 1e250;
const LANES: usize = 8;
const PROBE_COLUMNS: usize = 64;
/// Palette bands across the middle 80% of a view's escape iterations. Outside this range the
/// zoom-based band width is replaced, so deep views neither wash out into a single band nor
/// turn into noise.
const MIN_BANDS: f64 = 4.0;
const MAX_BANDS: f64 = 48.0;
/// Palette position of the view's 10th percentile once bands are fitted to the view, in the
/// blues rather than the dark end of the palette.
const ANCHOR_POSITION: f64 = 4.0;
/// Fewer escaped probe points than this are too few to fit bands to.
const MIN_PROBE_ESCAPES: usize = 32;
const CYCLE_CHECK_START: usize = 16;
pub(crate) const BASE_VIEW_WIDTH: f64 = 3.0;
const LIGHT_ANGLE_DEGREES: f64 = 45.0;

/// The band width and phase for a view, given its zoom-based band width and the probe's smooth
/// escape iterations. Keeps the zoom-based width while it gives between [`MIN_BANDS`] and
/// [`MAX_BANDS`] bands; otherwise clamps to that range and blends the phase towards anchoring
/// the 10th percentile at [`ANCHOR_POSITION`], so colors change continuously with zoom.
fn color_bands(zoom_width: f64, smooth: &mut [f64]) -> (f64, f64) {
    if smooth.len() < MIN_PROBE_ESCAPES {
        return (zoom_width, 0.0);
    }
    smooth.sort_by(f64::total_cmp);
    let percentile = |p: f64| smooth[((smooth.len() - 1) as f64 * p).round() as usize];
    let (low, high) = (percentile(0.1), percentile(0.9));
    let spread = high - low;
    if spread <= 0.0 {
        return (zoom_width, 0.0);
    }
    let width = zoom_width.clamp(spread / MAX_BANDS, spread / MIN_BANDS);
    let anchoring = 1.0 - width.min(zoom_width) / width.max(zoom_width);
    (width, anchoring * (ANCHOR_POSITION - low / width))
}

pub fn render(view: &Viewport, opts: &RenderOptions) -> RgbImage {
    Renderer::new(view, opts).render()
}

/// Renders the rows starting at `first_row` into `rows`, a packed RGB buffer holding a whole
/// number of rows of the full `opts.width` x `opts.height` image.
pub fn render_rows(view: &Viewport, opts: &RenderOptions, first_row: u32, rows: &mut [u8]) {
    Renderer::new(view, opts).render_rows(first_row, rows);
}

/// A prepared view: the pixel grid and, for deep zooms, the reference orbit, which can be
/// shared between renderers of the same center.
pub struct Renderer {
    view: Viewport,
    opts: RenderOptions,
    frame: Frame,
    orbit: Option<Arc<ReferenceOrbit>>,
}

impl Renderer {
    pub fn new(view: &Viewport, opts: &RenderOptions) -> Self {
        Self::reusing(view, opts, None)
    }

    pub fn view(&self) -> &Viewport {
        &self.view
    }

    /// Like [`Renderer::new`], but reuses the reference orbit and color bands of `previous`
    /// where they still apply, e.g. for other bands or resolutions of the same view.
    pub fn reusing(view: &Viewport, opts: &RenderOptions, previous: Option<&Renderer>) -> Self {
        // Rounded up so slightly different resolutions of a view can share one orbit
        let aspect = (opts.height as f64 / opts.width.max(1) as f64 * 8.0).ceil() / 8.0;
        let orbit = (view.zoom >= PERTURBATION_ZOOM).then(|| {
            match previous.and_then(|p| p.orbit.as_ref()) {
                Some(orbit) if orbit.matches(view, opts.max_iterations, aspect) => orbit.clone(),
                _ => Arc::new(ReferenceOrbit::compute(view, opts.max_iterations, aspect)),
            }
        });
        let mut renderer = Self {
            view: view.clone(),
            opts: *opts,
            frame: Frame::new(view, opts),
            orbit,
        };
        let same_bands = previous.filter(|p| {
            p.view == *view
                && p.opts.max_iterations == opts.max_iterations
                && p.probe_rows() == renderer.probe_rows()
        });
        (renderer.frame.band_scale, renderer.frame.band_phase) = match same_bands {
            Some(p) => (p.frame.band_scale, p.frame.band_phase),
            None => color_bands(renderer.frame.band_scale, &mut renderer.probe()),
        };
        renderer
    }

    fn probe_rows(&self) -> usize {
        let aspect = self.opts.height as f64 / self.opts.width as f64;
        ((PROBE_COLUMNS as f64 * aspect).round() as usize).clamp(1, 4 * PROBE_COLUMNS)
    }

    /// Smooth escape iterations of a coarse grid spanning the view, independent of the output
    /// resolution apart from its aspect ratio.
    fn probe(&self) -> Vec<f64> {
        let (width, height) = (self.opts.width as f64, self.opts.height as f64);
        let rows = self.probe_rows();
        let frame = &self.frame;
        let max_iterations = self.opts.max_iterations;
        (0..rows)
            .into_par_iter()
            .flat_map_iter(|row| {
                let y = (row as f64 + 0.5) * height / rows as f64;
                let x = |column: usize| (column as f64 + 0.5) * width / PROBE_COLUMNS as f64;
                let escapes: Vec<Option<Escape>> = match &self.orbit {
                    Some(orbit) => (0..PROBE_COLUMNS)
                        .map(|column| {
                            let (dx, dy) = frame.offset_at(x(column), y);
                            orbit.escape::<false>(dx, dy, max_iterations)
                        })
                        .collect(),
                    None => (0..PROBE_COLUMNS / LANES)
                        .flat_map(|block| {
                            let ci = frame.center_y + frame.offset_at(0.0, y).1;
                            let cr = std::array::from_fn(|lane| {
                                frame.center_x + frame.offset_at(x(block * LANES + lane), y).0
                            });
                            escape_lanes::<false>(&cr, ci, max_iterations)
                        })
                        .collect(),
                };
                escapes
                    .into_iter()
                    .flatten()
                    .map(|escape| escape.smooth_iterations())
            })
            .collect()
    }

    pub fn reference_orbit(&self) -> Option<&Arc<ReferenceOrbit>> {
        self.orbit.as_ref()
    }

    pub fn render(&self) -> RgbImage {
        let mut img = RgbImage::new(self.opts.width, self.opts.height);
        self.render_rows(0, &mut img);
        img
    }

    /// See [`render_rows`].
    pub fn render_rows(&self, first_row: u32, rows: &mut [u8]) {
        let row_len = self.opts.width as usize * 3;
        if row_len == 0 || rows.is_empty() {
            return;
        }
        assert!(
            rows.len().is_multiple_of(row_len)
                && first_row as usize + rows.len() / row_len <= self.opts.height as usize,
            "row buffer must hold whole rows within the image"
        );
        match (self.opts.shading, &self.orbit) {
            (Shading::Flat, None) => self.render_direct::<false>(first_row, rows),
            (Shading::Normal, None) => self.render_direct::<true>(first_row, rows),
            (Shading::Flat, Some(orbit)) => self.render_perturbed::<false>(orbit, first_row, rows),
            (Shading::Normal, Some(orbit)) => self.render_perturbed::<true>(orbit, first_row, rows),
        }
    }

    fn render_direct<const NORMAL: bool>(&self, first_row: u32, rows: &mut [u8]) {
        let frame = &self.frame;
        let max_iterations = self.opts.max_iterations;
        rows.par_chunks_mut(self.opts.width as usize * 3)
            .enumerate()
            .for_each(|(offset, row)| {
                let ci = frame.imag(first_row as usize + offset);
                for (block, pixels) in row.chunks_mut(3 * LANES).enumerate() {
                    let cr = std::array::from_fn(|lane| frame.real(block * LANES + lane));
                    let escapes = escape_lanes::<NORMAL>(&cr, ci, max_iterations);
                    let (pixels, _) = pixels.as_chunks_mut::<3>();
                    for (pixel, escape) in pixels.iter_mut().zip(&escapes) {
                        *pixel = frame.color::<NORMAL>(escape.as_ref()).0;
                    }
                }
            });
    }

    fn render_perturbed<const NORMAL: bool>(
        &self,
        orbit: &ReferenceOrbit,
        first_row: u32,
        rows: &mut [u8],
    ) {
        let frame = &self.frame;
        let max_iterations = self.opts.max_iterations;
        rows.par_chunks_mut(self.opts.width as usize * 3)
            .enumerate()
            .for_each(|(offset, row)| {
                let dci = frame.offset_y(first_row as usize + offset);
                let (pixels, _) = row.as_chunks_mut::<3>();
                for (x, pixel) in pixels.iter_mut().enumerate() {
                    let escape = orbit.escape::<NORMAL>(frame.offset_x(x), dci, max_iterations);
                    *pixel = frame.color::<NORMAL>(escape.as_ref()).0;
                }
            });
    }
}

struct Frame {
    left: f64,
    top: f64,
    center_x: f64,
    center_y: f64,
    pixel_size: f64,
    band_scale: f64,
    band_phase: f64,
    light: (f64, f64),
}

impl Frame {
    fn new(view: &Viewport, opts: &RenderOptions) -> Self {
        let angle = LIGHT_ANGLE_DEGREES.to_radians();
        Self {
            left: opts.width as f64 / 2.0,
            top: opts.height as f64 / 2.0,
            center_x: view.center_x.to_f64(),
            center_y: view.center_y.to_f64(),
            pixel_size: BASE_VIEW_WIDTH / view.zoom / opts.width as f64,
            band_scale: (view.zoom + 1.0).log2(),
            band_phase: 0.0,
            light: (angle.cos(), angle.sin()),
        }
    }

    fn offset_x(&self, x: usize) -> f64 {
        (x as f64 - self.left) * self.pixel_size
    }

    fn offset_y(&self, y: usize) -> f64 {
        (y as f64 - self.top) * self.pixel_size
    }

    fn offset_at(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.left) * self.pixel_size,
            (y - self.top) * self.pixel_size,
        )
    }

    fn real(&self, x: usize) -> f64 {
        self.center_x + self.offset_x(x)
    }

    fn imag(&self, y: usize) -> f64 {
        self.center_y + self.offset_y(y)
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
        escape.smooth_iterations() / self.band_scale + self.band_phase
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Escape {
    pub(crate) iterations: usize,
    pub(crate) norm_sqr: f64,
    pub(crate) z: (f64, f64),
    pub(crate) derivative: (f64, f64),
}

impl Escape {
    fn smooth_iterations(&self) -> f64 {
        let log_modulus = self.norm_sqr.ln() * 0.5;
        let nu = (log_modulus / std::f64::consts::LN_2).log2();
        self.iterations as f64 + 1.0 - nu
    }

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
        assert_matches_reference(Viewport::from_f64(-0.75, 0.0, 1.0), 67, 53);
    }

    #[test]
    fn lanes_match_scalar_iteration_on_deep_zoom() {
        let view = Viewport::from_f64(-1.249559196, 0.030466443, 1.73e6);
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
        let img = render(&Viewport::from_f64(-0.75, 0.0, 1.0), &opts);
        assert_eq!(img.dimensions(), (21, 9));
    }

    #[test]
    fn row_bands_match_full_render() {
        let view = Viewport::from_f64(-0.7453, 0.1127, 150.0);
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
        render_rows(&Viewport::from_f64(0.0, 0.0, 1.0), &opts, 0, &mut [0; 5]);
    }

    #[test]
    fn zoom_at_keeps_the_anchor_in_place() {
        let view = Viewport::from_f64(-0.5, 0.25, 4.0);
        let zoomed = view.zoom_at(0.1, -0.2, 8.0);
        assert_eq!(zoomed.zoom, 32.0);
        assert_eq!(zoomed.center_x.to_string(), "-0.4125");
        assert_eq!(zoomed.center_y.to_string(), "0.075");
        assert_eq!(view.zoom_at(0.0, 0.0, 1e300).zoom, MAX_ZOOM);
    }

    #[test]
    fn deep_pans_keep_full_precision() {
        let view = Viewport {
            zoom: 1e40,
            ..Viewport::from_f64(-0.75, 0.1, 1.0)
        };
        let moved = view.pan(3e-41, -1e-45);
        assert_eq!(
            moved.center_x.to_string(),
            "-0.74999999999999999999999999999999999999997"
        );
        assert_eq!(
            moved.center_y.to_string(),
            "0.099999999999999999999999999999999999999999999"
        );
        assert_eq!(moved.pan(-3e-41, 1e-45), view);
    }

    fn spread_iterations(low: f64, high: f64) -> Vec<f64> {
        (0..=100)
            .map(|i| low + (high - low) * i as f64 / 100.0)
            .collect()
    }

    #[test]
    fn color_bands_keep_zoom_width_within_range() {
        let (width, phase) = color_bands(20.0, &mut spread_iterations(300.0, 1000.0));
        assert_eq!((width, phase), (20.0, 0.0));
    }

    #[test]
    fn color_bands_widen_and_narrow_to_the_view() {
        let mut few = spread_iterations(100.0, 110.0);
        let (width, phase) = color_bands(1e6, &mut few);
        assert!((width - 8.0 / MIN_BANDS).abs() < 1e-9);
        assert!((101.0 / width + phase - ANCHOR_POSITION).abs() < 1e-3);

        let mut many = spread_iterations(15000.0, 30000.0);
        let (width, _) = color_bands(55.0, &mut many);
        assert!((width - 12000.0 / MAX_BANDS).abs() < 1e-6);
    }

    #[test]
    fn color_bands_change_continuously_with_zoom() {
        let mut iterations = spread_iterations(100.0, 110.0);
        let limit = 8.0 / MIN_BANDS;
        let position = |zoom_width: f64, iterations: &mut [f64]| {
            let (width, phase) = color_bands(zoom_width, iterations);
            105.0 / width + phase
        };
        let below = position(limit * (1.0 - 1e-9), &mut iterations);
        let above = position(limit * (1.0 + 1e-9), &mut iterations);
        assert!((below - above).abs() < 1e-6, "{below} vs {above}");
    }

    #[test]
    fn color_bands_ignore_views_with_few_escapes() {
        assert_eq!(color_bands(133.0, &mut [100.0; 10]), (133.0, 0.0));
        assert_eq!(color_bands(133.0, &mut [100.0; 100]), (133.0, 0.0));
    }

    #[test]
    fn reusing_shares_bands_between_resolutions() {
        let view = Viewport {
            center_x: "0".parse().unwrap(),
            center_y: "1".parse().unwrap(),
            zoom: 1e40,
        };
        let full = RenderOptions {
            width: 800,
            height: 600,
            max_iterations: 500,
            shading: Shading::Normal,
        };
        let preview = RenderOptions {
            width: 200,
            height: 150,
            ..full
        };
        let first = Renderer::new(&view, &full);
        let second = Renderer::reusing(&view, &preview, Some(&first));
        assert!(Arc::ptr_eq(
            first.reference_orbit().unwrap(),
            second.reference_orbit().unwrap()
        ));
        assert_eq!(first.frame.band_scale, second.frame.band_scale);
        assert_eq!(first.frame.band_phase, second.frame.band_phase);
        assert_ne!(first.frame.band_scale, (view.zoom + 1.0).log2());
    }

    #[test]
    fn render_handles_empty_images() {
        let opts = RenderOptions {
            width: 0,
            height: 5,
            ..RenderOptions::default()
        };
        let img = render(&Viewport::from_f64(0.0, 0.0, 1.0), &opts);
        assert_eq!(img.dimensions(), (0, 5));
    }
}
