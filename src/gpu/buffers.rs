//! Host-side mirrors of the shader's structs, and the buffers one render binds.

use super::{BULB_CHECK_ZOOM, GpuRenderer};
use crate::ReferenceOrbit;
use crate::render::{Renderer, Shading};
use bytemuck::{Pod, Zeroable};
use wgpu::BufferUsages as Usage;
use wgpu::util::DeviceExt;

/// Size of `State` in the shader.
const STATE_SIZE: usize = 56;
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
    _padding: [u32; 3],
}

impl Params {
    /// Parameters for the first slice of the whole image; bands set their own rows.
    pub(super) fn new(renderer: &Renderer, orbit: &ReferenceOrbit, slice_steps: u32) -> Self {
        let opts = renderer.options();
        let view = renderer.view();
        let (band_scale, band_phase) = renderer.color_bands();
        let light = renderer.light();
        Self {
            width: opts.width,
            first_row: 0,
            rows: opts.height,
            max_iterations: opts.max_iterations.min(u32::MAX as usize - 1) as u32,
            last: (orbit.points().len() - 1) as u32,
            pixel_size: renderer.pixel_size() as f32,
            left: opts.width as f32 / 2.0,
            top: opts.height as f32 / 2.0,
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
            _padding: [0; 3],
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Step {
    a: [f32; 2],
    b: [f32; 2],
    radius_sqr: f32,
    _padding: f32,
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
        let points: Vec<[f32; 2]> = orbit
            .points()
            .iter()
            .map(|&(r, i)| [r as f32, i as f32])
            .collect();
        let mut steps = Vec::new();
        let mut levels = Vec::new();
        for level in orbit.bla().levels() {
            levels.push([steps.len() as u32, level.len() as u32]);
            steps.extend(level.iter().map(|step| Step {
                a: [step.a.0 as f32, step.a.1 as f32],
                b: [step.b.0 as f32, step.b.1 as f32],
                radius_sqr: step.radius_sqr as f32,
                _padding: 0.0,
            }));
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
