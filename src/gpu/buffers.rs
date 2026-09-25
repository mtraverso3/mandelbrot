//! Host-side mirrors of the shader's structs, and the buffers one render binds.

use super::{BULB_CHECK_ZOOM, GpuRenderer};
use crate::ReferenceOrbit;
use crate::render::{Renderer, Shading};
use bytemuck::{Pod, Zeroable};
use wgpu::BufferUsages as Usage;
use wgpu::util::DeviceExt;

/// Size of `State` in the shader.
const STATE_SIZE: usize = 72;
const PALETTE_SIZE: f64 = 16.0;
const ITERATION_BLOCK: f64 = 4096.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Params {
    pub(super) width: u32,
    pub(super) first_row: u32,
    pub(super) rows: u32,
    max_iterations: u32,
    last: u32,
    pixel_size: f32,
    left: f32,
    top: f32,
    slice_steps: u32,
    pub(super) first_slice: u32,
    bla_levels: u32,
    max_skip_radius_sqr: f32,
    center: [f32; 2],
    check_bulbs: u32,
    normal_shading: u32,
    light: [f32; 2],
    inv_band_scale: f32,
    band_cycle: f32,
    band_phase: f32,
    pixel_mantissa: f32,
    pixel_exponent: i32,
    _padding: u32,
}

impl Params {
    /// Parameters for the first slice of the whole image; bands set their own rows.
    pub(super) fn new(renderer: &Renderer, orbit: &ReferenceOrbit, slice_steps: u32) -> Self {
        let opts = renderer.options();
        let view = renderer.view();
        let (band_scale, band_phase) = renderer.color_bands();
        let light = renderer.light();
        let (pixel_mantissa, pixel_exponent) = split(renderer.pixel_size());
        let (left, top) = renderer.origin();
        Self {
            width: opts.width,
            first_row: 0,
            rows: opts.height,
            max_iterations: opts.max_iterations.min(u32::MAX as usize - 1) as u32,
            last: (orbit.points().len() - 1) as u32,
            pixel_size: renderer.pixel_size() as f32,
            left: left as f32,
            top: top as f32,
            slice_steps,
            first_slice: 1,
            bla_levels: orbit.bla().levels().len() as u32,
            max_skip_radius_sqr: max_skip_radius_sqr(orbit),
            center: [view.center_x.to_f64() as f32, view.center_y.to_f64() as f32],
            check_bulbs: (view.zoom < BULB_CHECK_ZOOM) as u32,
            normal_shading: (opts.shading == Shading::Normal) as u32,
            light: [light.0 as f32, light.1 as f32],
            inv_band_scale: (1.0 / band_scale) as f32,
            band_cycle: (ITERATION_BLOCK / band_scale).rem_euclid(PALETTE_SIZE) as f32,
            band_phase: band_phase.rem_euclid(PALETTE_SIZE) as f32,
            pixel_mantissa,
            pixel_exponent,
            _padding: 0,
        }
    }
}

/// Blocks are never valid further out than their first half, so the largest radius on the
/// first merged level bounds every skip.
fn max_skip_radius_sqr(orbit: &ReferenceOrbit) -> f32 {
    orbit.bla().levels().get(1).map_or(0.0, |level| {
        level
            .iter()
            .map(|step| step.radius_sqr as f32)
            .fold(0.0, f32::max)
    })
}

/// `x` as an f32 mantissa and a power of two, for values past f32 range.
fn split(x: f64) -> (f32, i32) {
    // Blocks whose coefficients overflowed have no radius, so are never used
    if x == 0.0 || !x.is_finite() {
        return (x as f32, 0);
    }
    let exponent = x.abs().log2().floor() as i32 + 1;
    ((x / 2f64.powi(exponent)) as f32, exponent)
}

/// A complex number as an f32 mantissa pair sharing one power of two.
fn split_complex((re, im): (f64, f64)) -> ([f32; 2], i32) {
    let (_, exponent) = split(re.abs().max(im.abs()));
    let scale = 2f64.powi(-exponent);
    ([(re * scale) as f32, (im * scale) as f32], exponent)
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct OrbitPoint {
    z: [f32; 2],
    mantissa: [f32; 2],
    exponent: i32,
    _padding: u32,
}

impl OrbitPoint {
    fn new(z: (f64, f64)) -> Self {
        let (mantissa, exponent) = split_complex(z);
        Self {
            z: [z.0 as f32, z.1 as f32],
            mantissa,
            exponent,
            _padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Step {
    a: [f32; 2],
    b: [f32; 2],
    a_mantissa: [f32; 2],
    b_mantissa: [f32; 2],
    a_exponent: i32,
    b_exponent: i32,
    radius_sqr: f32,
    radius_mantissa: f32,
    radius_exponent: i32,
    _padding: u32,
}

impl Step {
    fn new(step: &crate::bla::Step) -> Self {
        let (a_mantissa, a_exponent) = split_complex(step.a);
        let (b_mantissa, b_exponent) = split_complex(step.b);
        let (radius_mantissa, radius_exponent) = split(step.radius_sqr.sqrt());
        Self {
            a: [step.a.0 as f32, step.a.1 as f32],
            b: [step.b.0 as f32, step.b.1 as f32],
            a_mantissa,
            b_mantissa,
            a_exponent,
            b_exponent,
            radius_sqr: step.radius_sqr as f32,
            radius_mantissa,
            radius_exponent,
            _padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct Sample {
    pub(super) iterations: u32,
    norm_sqr: f32,
    normal: [f32; 2],
}

pub(super) struct Buffers {
    pub(super) params: wgpu::Buffer,
    pub(super) samples: wgpu::Buffer,
    pub(super) pixels: wgpu::Buffer,
    pub(super) unfinished: wgpu::Buffer,
    pub(super) readback: wgpu::Buffer,
    pub(super) unfinished_readback: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
}

impl Buffers {
    /// Buffers for bands of up to `pixels` pixels.
    pub(super) fn new(gpu: &GpuRenderer, orbit: &ReferenceOrbit, pixels: usize) -> Self {
        let device = &gpu.device;
        let points: Vec<OrbitPoint> = orbit.points().iter().map(|&z| OrbitPoint::new(z)).collect();
        let mut steps = Vec::new();
        let mut levels = Vec::new();
        for level in orbit.bla().levels() {
            levels.push([steps.len() as u32, level.len() as u32]);
            steps.extend(level.iter().map(Step::new));
        }
        if steps.is_empty() {
            steps.push(Step::zeroed());
        }

        let init = |label, contents: &[u8]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: Usage::STORAGE,
            })
        };
        let buffer = |label, size: usize, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage,
                mapped_at_creation: false,
            })
        };
        let orbit = init("orbit", bytemuck::cast_slice(&points));
        let bla = init("bla", bytemuck::cast_slice(&steps));
        let bla_levels = init("bla levels", bytemuck::cast_slice(&levels));
        let params = buffer(
            "params",
            size_of::<Params>(),
            Usage::UNIFORM | Usage::COPY_DST,
        );
        let states = buffer("states", pixels * STATE_SIZE, Usage::STORAGE);
        let sample_size = pixels * size_of::<Sample>();
        let samples = buffer("samples", sample_size, Usage::STORAGE | Usage::COPY_SRC);
        let rgba = buffer("pixels", pixels * 4, Usage::STORAGE | Usage::COPY_SRC);
        let unfinished = buffer(
            "unfinished",
            4,
            Usage::STORAGE | Usage::COPY_SRC | Usage::COPY_DST,
        );
        let readback = buffer("readback", sample_size, Usage::MAP_READ | Usage::COPY_DST);
        let unfinished_readback =
            buffer("unfinished readback", 4, Usage::MAP_READ | Usage::COPY_DST);

        let bindings = [
            &params,
            &orbit,
            &bla,
            &bla_levels,
            &states,
            &samples,
            &unfinished,
            &rgba,
        ];
        let entries: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(binding, buffer)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: buffer.as_entire_binding(),
            })
            .collect();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &gpu.bind_group_layout,
            entries: &entries,
        });
        Self {
            params,
            samples,
            pixels: rgba,
            unfinished,
            readback,
            unfinished_readback,
            bind_group,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keeps_values_past_f32_range() {
        for x in [3e-200, -7.5e150, 1.0, 0.1] {
            let (mantissa, exponent) = split(x);
            assert!((0.5..1.0).contains(&mantissa.abs()), "{x}: {mantissa}");
            let back = mantissa as f64 * 2f64.powi(exponent);
            assert!((back / x - 1.0).abs() < 1e-7, "{x} came back as {back}");
        }
        assert_eq!(split(0.0), (0.0, 0));
        assert_eq!(split(f64::INFINITY), (f32::INFINITY, 0));
    }
}
