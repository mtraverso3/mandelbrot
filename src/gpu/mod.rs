//! Renders on the GPU with f32 perturbation around a reference orbit computed on the CPU.

use crate::ReferenceOrbit;
use crate::color::INTERIOR;
use crate::render::{PERTURBATION_ZOOM, RenderOptions, Renderer, Shading, Viewport};
use bytemuck::{Pod, Zeroable};
use image::RgbImage;
use rayon::prelude::*;
use std::fmt;
use std::sync::{Arc, OnceLock};
use wgpu::util::DeviceExt;

/// Past this, pixel offsets approach the smallest normal f32.
pub const GPU_MAX_ZOOM: f64 = 1e30;
const WORKGROUP_SIZE: u32 = 8;
const INTERIOR_SAMPLE: u32 = u32::MAX;
/// Size of `State` in the shader.
const STATE_SIZE: usize = 56;

/// How a render is split up, so buffers stay small and each dispatch stays well within the
/// time the OS allows before it resets the GPU.
#[derive(Clone, Copy)]
struct Chunking {
    band_pixels: usize,
    slice_steps: u32,
}

const CHUNKING: Chunking = Chunking {
    band_pixels: 1 << 20,
    slice_steps: 4096,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    width: u32,
    first_row: u32,
    rows: u32,
    max_iterations: u32,
    last: u32,
    pixel_size: f32,
    left: f32,
    top: f32,
    slice_steps: u32,
    first_slice: u32,
    bla_levels: u32,
    max_skip_radius_sqr: f32,
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
struct Sample {
    iterations: u32,
    norm_sqr: f32,
    normal: [f32; 2],
}

#[derive(Debug)]
pub enum GpuError {
    Adapter(wgpu::RequestAdapterError),
    Device(wgpu::RequestDeviceError),
    Poll(wgpu::PollError),
    Readback(wgpu::BufferAsyncError),
    ZoomTooDeep(f64),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(e) => write!(f, "no GPU available: {e}"),
            Self::Device(e) => write!(f, "could not open the GPU: {e}"),
            Self::Poll(e) => write!(f, "GPU render failed: {e}"),
            Self::Readback(e) => write!(f, "could not read the GPU render back: {e}"),
            Self::ZoomTooDeep(zoom) => write!(
                f,
                "zoom {zoom:e} is past the GPU's deepest zoom of {GPU_MAX_ZOOM:e}"
            ),
        }
    }
}

impl std::error::Error for GpuError {}

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    /// Built on first use, indexed by [`Variant::index`].
    pipelines: [OnceLock<wgpu::ComputePipeline>; 4],
    adapter_name: String,
}

/// Features compiled in or out of the shader, like the CPU renderer's const generics.
#[derive(Clone, Copy)]
struct Variant {
    skip: bool,
    track_derivative: bool,
}

impl Variant {
    fn index(self) -> usize {
        self.skip as usize * 2 + self.track_derivative as usize
    }
}

impl GpuRenderer {
    pub fn new() -> Result<Self, GpuError> {
        pollster::block_on(Self::request())
    }

    async fn request() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .map_err(GpuError::Adapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(GpuError::Device)?;
        let module = device.create_shader_module(wgpu::include_wgsl!("perturbation.wgsl"));
        let storage = |read_only| wgpu::BufferBindingType::Storage { read_only };
        let types = [
            wgpu::BufferBindingType::Uniform,
            storage(true),
            storage(true),
            storage(true),
            storage(false),
            storage(false),
            storage(false),
        ];
        let entries: Vec<_> = types
            .into_iter()
            .enumerate()
            .map(|(binding, ty)| wgpu::BindGroupLayoutEntry {
                binding: binding as u32,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        Ok(Self {
            device,
            queue,
            module,
            bind_group_layout,
            layout,
            pipelines: Default::default(),
            adapter_name: adapter.get_info().name,
        })
    }

    fn pipeline(&self, variant: Variant) -> &wgpu::ComputePipeline {
        self.pipelines[variant.index()].get_or_init(|| {
            let constants = [
                ("SKIP", variant.skip as u8 as f64),
                ("TRACK_DERIVATIVE", variant.track_derivative as u8 as f64),
            ];
            self.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("perturbation"),
                    layout: Some(&self.layout),
                    module: &self.module,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &constants,
                        ..Default::default()
                    },
                    cache: None,
                })
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn render(&self, view: &Viewport, opts: &RenderOptions) -> Result<RgbImage, GpuError> {
        if view.zoom > GPU_MAX_ZOOM {
            return Err(GpuError::ZoomTooDeep(view.zoom));
        }
        let renderer = Renderer::new(view, opts);
        let samples = self.samples(&renderer, CHUNKING)?;
        let mut img = RgbImage::new(opts.width, opts.height);
        img.par_chunks_mut(3)
            .zip(samples.par_iter())
            .for_each(|(pixel, sample)| {
                let color = match sample.iterations {
                    INTERIOR_SAMPLE => INTERIOR,
                    iterations => renderer.paint(
                        iterations as usize,
                        sample.norm_sqr as f64,
                        (sample.normal[0] as f64, sample.normal[1] as f64),
                    ),
                };
                pixel.copy_from_slice(&color.0);
            });
        Ok(img)
    }

    fn samples(&self, renderer: &Renderer, chunking: Chunking) -> Result<Vec<Sample>, GpuError> {
        let opts = renderer.options();
        let (width, height) = (opts.width as usize, opts.height as usize);
        if width == 0 || height == 0 {
            return Ok(Vec::new());
        }
        let orbit = match renderer.reference_orbit() {
            Some(orbit) => orbit.clone(),
            None => Arc::new(ReferenceOrbit::compute(
                renderer.view(),
                opts.max_iterations,
                height as f64 / width as f64,
            )),
        };
        let pipeline = self.pipeline(Variant {
            // As on the CPU, skipped blocks are only long enough to pay for their lookups once
            // perturbation is needed
            skip: renderer.view().zoom >= PERTURBATION_ZOOM,
            track_derivative: opts.shading == Shading::Normal,
        });
        let band_rows = (chunking.band_pixels / width).clamp(1, height);
        let buffers = Buffers::new(self, &orbit, band_rows * width);

        let mut samples = Vec::with_capacity(width * height);
        for first_row in (0..height).step_by(band_rows) {
            let rows = band_rows.min(height - first_row);
            let mut params = Params {
                width: width as u32,
                first_row: first_row as u32,
                rows: rows as u32,
                max_iterations: opts.max_iterations.min(u32::MAX as usize - 1) as u32,
                last: (orbit.points().len() - 1) as u32,
                pixel_size: renderer.pixel_size() as f32,
                left: width as f32 / 2.0,
                top: height as f32 / 2.0,
                slice_steps: chunking.slice_steps,
                first_slice: 1,
                bla_levels: orbit.bla().levels().len() as u32,
                max_skip_radius_sqr: max_skip_radius_sqr(&orbit),
            };
            let size = (rows * width * size_of::<Sample>()) as u64;
            while self.slice(pipeline, &buffers, &params, size)? > 0 {
                params.first_slice = 0;
            }
            self.read(&buffers.readback, size, |bytes| {
                samples.extend_from_slice(bytemuck::cast_slice(bytes))
            })?;
        }
        Ok(samples)
    }

    /// Runs up to `slice_steps` steps on every unfinished pixel of the band, returning how
    /// many pixels are still unfinished. The samples are copied out every time, so the last
    /// slice needs no further round trip to the GPU.
    fn slice(
        &self,
        pipeline: &wgpu::ComputePipeline,
        buffers: &Buffers,
        params: &Params,
        samples_size: u64,
    ) -> Result<u32, GpuError> {
        self.queue
            .write_buffer(&buffers.params, 0, bytemuck::bytes_of(params));
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.clear_buffer(&buffers.unfinished, 0, None);
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &buffers.bind_group, &[]);
            pass.dispatch_workgroups(
                params.width.div_ceil(WORKGROUP_SIZE),
                params.rows.div_ceil(WORKGROUP_SIZE),
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&buffers.unfinished, 0, &buffers.unfinished_readback, 0, 4);
        encoder.copy_buffer_to_buffer(&buffers.samples, 0, &buffers.readback, 0, samples_size);
        self.queue.submit([encoder.finish()]);
        self.read(&buffers.unfinished_readback, 4, |bytes| {
            bytemuck::pod_read_unaligned(bytes)
        })
    }

    fn read<T>(
        &self,
        buffer: &wgpu::Buffer,
        size: u64,
        f: impl FnOnce(&[u8]) -> T,
    ) -> Result<T, GpuError> {
        let slice = buffer.slice(..size);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(GpuError::Poll)?;
        receiver
            .recv()
            .expect("map callback runs during poll")
            .map_err(GpuError::Readback)?;
        let view = slice.get_mapped_range().expect("mapped above");
        let value = f(&view);
        drop(view);
        buffer.unmap();
        Ok(value)
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

struct Buffers {
    params: wgpu::Buffer,
    samples: wgpu::Buffer,
    unfinished: wgpu::Buffer,
    readback: wgpu::Buffer,
    unfinished_readback: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Buffers {
    fn new(gpu: &GpuRenderer, orbit: &ReferenceOrbit, pixels: usize) -> Self {
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
                usage: wgpu::BufferUsages::STORAGE,
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
        use wgpu::BufferUsages as Usage;
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
    use crate::preset;

    fn escapes(gpu: &GpuRenderer, renderer: &Renderer, chunking: Chunking) -> Vec<Option<usize>> {
        gpu.samples(renderer, chunking)
            .unwrap()
            .iter()
            .map(|s| (s.iterations != INTERIOR_SAMPLE).then_some(s.iterations as usize))
            .collect()
    }

    fn exact_escapes(renderer: &Renderer) -> Vec<Option<usize>> {
        let opts = renderer.options();
        let orbit = ReferenceOrbit::compute(renderer.view(), opts.max_iterations, 1.0);
        let pixel = renderer.pixel_size();
        let (left, top) = (opts.width as f64 / 2.0, opts.height as f64 / 2.0);
        (0..opts.height as usize * opts.width as usize)
            .map(|i| {
                let x = (i % opts.width as usize) as f64 - left;
                let y = (i / opts.width as usize) as f64 - top;
                orbit.escape_unskipped(x * pixel, y * pixel, opts.max_iterations)
            })
            .collect()
    }

    /// f32 rounding reshuffles iteration counts in chaotic texture, and can flip pixels that
    /// escape right at the iteration limit, but must not change what is inside the set.
    fn assert_close_to_exact(view: Viewport, max_iterations: usize, max_mismatch: f64) {
        let gpu = GpuRenderer::new().unwrap();
        let opts = RenderOptions {
            width: 96,
            height: 64,
            max_iterations,
            shading: Shading::Normal,
        };
        let renderer = Renderer::new(&view, &opts);
        let actual = escapes(&gpu, &renderer, CHUNKING);
        let expected = exact_escapes(&renderer);
        assert!(
            expected.iter().any(|e| *e != expected[0]),
            "grid should not be uniform"
        );
        let pairs = || actual.iter().zip(&expected);
        let flipped = pairs().filter(|(a, e)| a.is_some() != e.is_some()).count();
        let mismatches = pairs().filter(|(a, e)| a != e).count();
        eprintln!(
            "zoom {:e}: {mismatches}/{} differ, {flipped} flipped",
            view.zoom,
            expected.len()
        );
        let pixels = expected.len() as f64;
        assert!(flipped as f64 <= 0.001 * pixels, "{flipped} pixels flipped");
        assert!(
            mismatches as f64 <= max_mismatch * pixels,
            "{mismatches} pixels differ"
        );
    }

    #[test]
    fn close_to_exact_on_full_view() {
        assert_close_to_exact(preset("mandelbrot").unwrap(), 1500, 0.01);
    }

    #[test]
    fn close_to_exact_in_chaotic_spirals() {
        assert_close_to_exact(preset("spirals").unwrap(), 6000, 0.25);
    }

    #[test]
    fn close_to_exact_when_reference_escapes_early() {
        assert_close_to_exact(Viewport::from_f64(0.2501, 0.0, 1e3), 2000, 0.01);
    }

    #[test]
    fn close_to_exact_around_a_minibrot() {
        assert_close_to_exact(preset("mini-mandelbrot").unwrap(), 24000, 0.25);
    }

    fn deep_seahorse() -> Viewport {
        Viewport {
            center_x: "-1.24949889563508492587065068503213228909045011806661"
                .parse()
                .unwrap(),
            center_y: "0.03033300303590165779311010118330780526875599532123"
                .parse()
                .unwrap(),
            zoom: 4.7374e16,
        }
    }

    #[test]
    fn close_to_exact_when_skipping_at_deep_zoom() {
        assert_close_to_exact(deep_seahorse(), 24000, 0.15);
    }

    #[test]
    fn close_to_exact_at_deepest_zoom() {
        let view = Viewport {
            center_x: "0".parse().unwrap(),
            center_y: "1".parse().unwrap(),
            zoom: GPU_MAX_ZOOM,
        };
        assert_close_to_exact(view, 3000, 0.01);
    }

    #[test]
    fn bands_and_slices_match_a_single_dispatch() {
        let gpu = GpuRenderer::new().unwrap();
        let opts = RenderOptions {
            width: 37,
            height: 23,
            max_iterations: 400,
            shading: Shading::Normal,
        };
        let renderer = Renderer::new(&Viewport::from_f64(-0.7453, 0.1127, 150.0), &opts);
        let whole = Chunking {
            band_pixels: usize::MAX,
            slice_steps: u32::MAX,
        };
        let split = Chunking {
            band_pixels: 37 * 5,
            slice_steps: 7,
        };
        let (whole, split) = (gpu.samples(&renderer, whole), gpu.samples(&renderer, split));
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&whole.unwrap()),
            bytemuck::cast_slice::<_, u8>(&split.unwrap())
        );
    }

    #[test]
    fn rejects_zoom_past_f32_range() {
        let gpu = GpuRenderer::new().unwrap();
        let view = Viewport::from_f64(0.0, 1.0, GPU_MAX_ZOOM * 10.0);
        let result = gpu.render(&view, &RenderOptions::default());
        assert!(matches!(result, Err(GpuError::ZoomTooDeep(_))));
    }

    #[test]
    fn shading_does_not_change_escapes() {
        let gpu = GpuRenderer::new().unwrap();
        for view in [preset("spirals").unwrap(), deep_seahorse()] {
            let [normal, flat] = [Shading::Normal, Shading::Flat].map(|shading| {
                let opts = RenderOptions {
                    width: 64,
                    height: 48,
                    max_iterations: 4000,
                    shading,
                };
                escapes(&gpu, &Renderer::new(&view, &opts), CHUNKING)
            });
            assert_eq!(normal, flat, "zoom {:e}", view.zoom);
        }
    }

    #[test]
    fn render_produces_requested_dimensions() {
        let gpu = GpuRenderer::new().unwrap();
        for (width, height) in [(21, 9), (0, 5)] {
            let opts = RenderOptions {
                width,
                height,
                max_iterations: 50,
                shading: Shading::Normal,
            };
            let img = gpu.render(&Viewport::from_f64(-0.75, 0.0, 1.0), &opts);
            assert_eq!(img.unwrap().dimensions(), (width, height));
        }
    }
}
