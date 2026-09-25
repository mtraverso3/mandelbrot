//! Renders on the GPU with f32 perturbation around a reference orbit computed on the CPU.

mod buffers;
#[cfg(test)]
mod tests;

use crate::ReferenceOrbit;
use crate::render::{
    MAX_AUTO_ITERATIONS, PERTURBATION_ZOOM, PROBE_COLUMNS, RenderOptions, Renderer, Shading,
    UNDECIDED_FRACTION, Viewport, probe_rows,
};
use buffers::{Buffers, Params, Sample};
use std::fmt;
use std::sync::{Arc, OnceLock};

/// Past this, pixel offsets approach the smallest normal f32, so they start out with a
/// separate exponent.
const DEEP_ZOOM: f64 = 1e30;
const WORKGROUP_SIZE: u32 = 8;
/// The sample of a pixel that is neither escaped nor known to be interior.
const UNDECIDED: u32 = u32::MAX - 1;

const SHADER: &str = concat!(
    include_str!("shaders/bindings.wgsl"),
    include_str!("shaders/iterate.wgsl"),
    include_str!("shaders/deep.wgsl"),
    include_str!("shaders/color.wgsl"),
);

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

#[derive(Debug)]
pub enum GpuError {
    Adapter(wgpu::RequestAdapterError),
    Device(wgpu::RequestDeviceError),
    Poll(wgpu::PollError),
    Readback(wgpu::BufferAsyncError),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(e) => write!(f, "no GPU available: {e}"),
            Self::Device(e) => write!(f, "could not open the GPU: {e}"),
            Self::Poll(e) => write!(f, "GPU render failed: {e}"),
            Self::Readback(e) => write!(f, "could not read the GPU render back: {e}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// Features compiled in or out of the iterate pass, like the CPU renderer's const generics.
#[derive(Clone, Copy)]
struct Variant {
    skip: bool,
    track_derivative: bool,
    deep: bool,
    detect_interior: bool,
}

impl Variant {
    fn new(renderer: &Renderer) -> Self {
        let zoom = renderer.view().zoom;
        Self {
            // As on the CPU, skipped blocks are only long enough to pay for their lookups
            // once perturbation is needed
            skip: zoom >= PERTURBATION_ZOOM,
            track_derivative: renderer.options().shading == Shading::Normal,
            deep: zoom > DEEP_ZOOM,
            detect_interior: false,
        }
    }

    fn index(self) -> usize {
        [
            self.deep,
            self.detect_interior,
            self.skip,
            self.track_derivative,
        ]
        .into_iter()
        .fold(0, |index, flag| index * 2 + flag as usize)
    }
}

/// What a render reads back for each pixel.
#[derive(Clone, Copy, PartialEq)]
enum Output {
    Rgba,
    /// Raw escapes, for probing and for comparing against exact iteration in tests
    Samples,
}

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    /// Built on first use, indexed by [`Variant::index`].
    iterate: [OnceLock<wgpu::ComputePipeline>; 16],
    color: wgpu::ComputePipeline,
    adapter_name: String,
}

impl GpuRenderer {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Result<Self, GpuError> {
        pollster::block_on(Self::new_async())
    }

    pub async fn new_async() -> Result<Self, GpuError> {
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
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mandelbrot"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let storage = |read_only| wgpu::BufferBindingType::Storage { read_only };
        let types = [
            wgpu::BufferBindingType::Uniform,
            storage(true),
            storage(true),
            storage(true),
            storage(false),
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
        let color = compute_pipeline(&device, &layout, &module, "color", &[]);
        Ok(Self {
            device,
            queue,
            module,
            bind_group_layout,
            layout,
            iterate: Default::default(),
            color,
            adapter_name: adapter.get_info().name,
        })
    }

    fn iterate_pipeline(&self, variant: Variant) -> &wgpu::ComputePipeline {
        self.iterate[variant.index()].get_or_init(|| {
            let constants = [
                ("SKIP", variant.skip as u8 as f64),
                ("TRACK_DERIVATIVE", variant.track_derivative as u8 as f64),
                ("DEEP", variant.deep as u8 as f64),
                ("DETECT_INTERIOR", variant.detect_interior as u8 as f64),
            ];
            compute_pipeline(
                &self.device,
                &self.layout,
                &self.module,
                "iterate",
                &constants,
            )
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn auto_iterations(
        &self,
        view: &Viewport,
        width: u32,
        height: u32,
    ) -> Result<usize, GpuError> {
        let limit = pollster::block_on(self.auto_iterations_async(view, width, height, || false))?;
        Ok(limit.expect("never cancelled"))
    }

    /// [`crate::auto_iterations`], probing on the GPU. Returns `Ok(None)` if `cancelled`
    /// returned true first.
    pub async fn auto_iterations_async(
        &self,
        view: &Viewport,
        width: u32,
        height: u32,
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<usize>, GpuError> {
        let rows = probe_rows(width, height);
        let mut limit = RenderOptions::default().max_iterations;
        while limit * 4 <= MAX_AUTO_ITERATIONS {
            let opts = RenderOptions {
                width: PROBE_COLUMNS as u32,
                height: rows as u32,
                max_iterations: limit,
                shading: Shading::Flat,
            };
            let renderer = Renderer::unprobed(view, &opts);
            let variant = Variant {
                detect_interior: true,
                ..Variant::new(&renderer)
            };
            let mut undecided = 0;
            let finished = self
                .run(
                    &renderer,
                    variant,
                    CHUNKING,
                    Output::Samples,
                    &cancelled,
                    &mut |_, bytes| {
                        let samples = bytemuck::pod_collect_to_vec::<u8, Sample>(bytes);
                        undecided += samples.iter().filter(|s| s.iterations == UNDECIDED).count();
                    },
                )
                .await?;
            if !finished {
                return Ok(None);
            }
            if undecided as f64 <= UNDECIDED_FRACTION * (PROBE_COLUMNS * rows) as f64 {
                break;
            }
            limit *= 4;
        }
        Ok(Some(limit))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn render(
        &self,
        view: &Viewport,
        opts: &RenderOptions,
    ) -> Result<image::RgbImage, GpuError> {
        use rayon::prelude::*;
        let renderer = Renderer::new(view, opts);
        let mut img = image::RgbImage::new(opts.width, opts.height);
        let row_len = opts.width as usize * 3;
        pollster::block_on(self.render_rows(
            &renderer,
            || false,
            |first_row, rgba| {
                let start = first_row as usize * row_len;
                let rgb: &mut [u8] = &mut img;
                rgb[start..start + rgba.len() / 4 * 3]
                    .par_chunks_mut(3)
                    .zip(rgba.par_chunks(4))
                    .for_each(|(rgb, rgba)| rgb.copy_from_slice(&rgba[..3]));
            },
        ))?;
        Ok(img)
    }

    /// Renders the view `renderer` was set up for, passing RGBA rows to `on_rows` a band at a
    /// time from the top. Returns `Ok(false)` if `cancelled` returned true first.
    pub async fn render_rows(
        &self,
        renderer: &Renderer,
        cancelled: impl Fn() -> bool,
        mut on_rows: impl FnMut(u32, &[u8]),
    ) -> Result<bool, GpuError> {
        self.run(
            renderer,
            Variant::new(renderer),
            CHUNKING,
            Output::Rgba,
            &cancelled,
            &mut |first_row, rows| on_rows(first_row, rows),
        )
        .await
    }

    async fn run(
        &self,
        renderer: &Renderer,
        variant: Variant,
        chunking: Chunking,
        output: Output,
        cancelled: &dyn Fn() -> bool,
        on_rows: &mut dyn FnMut(u32, &[u8]),
    ) -> Result<bool, GpuError> {
        let view = renderer.view();
        let opts = renderer.options();
        let (width, height) = (opts.width as usize, opts.height as usize);
        if width == 0 || height == 0 {
            return Ok(true);
        }
        let orbit = match renderer.reference_orbit() {
            Some(orbit) => orbit.clone(),
            None => Arc::new(ReferenceOrbit::compute(
                view,
                opts.max_iterations,
                height as f64 / width as f64,
            )),
        };
        let iterate = self.iterate_pipeline(variant);
        let band_rows = (chunking.band_pixels / width).clamp(1, height);
        let buffers = Buffers::new(self, &orbit, band_rows * width);
        let mut params = Params::new(renderer, &orbit, chunking.slice_steps);

        for first_row in (0..height).step_by(band_rows) {
            params.first_row = first_row as u32;
            params.rows = band_rows.min(height - first_row) as u32;
            params.first_slice = 1;
            loop {
                if cancelled() {
                    return Ok(false);
                }
                if self.slice(iterate, &buffers, &params, output).await? == 0 {
                    break;
                }
                params.first_slice = 0;
            }
            let pixel_size = match output {
                Output::Rgba => 4,
                Output::Samples => size_of::<Sample>(),
            };
            let size = (params.rows * params.width) as u64 * pixel_size as u64;
            self.read(&buffers.readback, size, |bytes| {
                on_rows(params.first_row, bytes)
            })
            .await?;
        }
        Ok(true)
    }

    /// Runs up to `slice_steps` steps on every unfinished pixel of the band, returning how
    /// many pixels are still unfinished. The output is colored and copied out every time, so
    /// the last slice needs no further round trip to the GPU.
    async fn slice(
        &self,
        iterate: &wgpu::ComputePipeline,
        buffers: &Buffers,
        params: &Params,
        output: Output,
    ) -> Result<u32, GpuError> {
        self.queue
            .write_buffer(&buffers.params, 0, bytemuck::bytes_of(params));
        let mut encoder = self.device.create_command_encoder(&Default::default());
        encoder.clear_buffer(&buffers.unfinished, 0, None);
        let workgroups = (
            params.width.div_ceil(WORKGROUP_SIZE),
            params.rows.div_ceil(WORKGROUP_SIZE),
        );
        let mut passes = vec![iterate];
        if output == Output::Rgba {
            passes.push(&self.color);
        }
        for pipeline in passes {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &buffers.bind_group, &[]);
            pass.dispatch_workgroups(workgroups.0, workgroups.1, 1);
        }
        let (source, pixel_size) = match output {
            Output::Rgba => (&buffers.pixels, 4),
            Output::Samples => (&buffers.samples, size_of::<Sample>() as u64),
        };
        let size = (params.rows * params.width) as u64 * pixel_size;
        encoder.copy_buffer_to_buffer(&buffers.unfinished, 0, &buffers.unfinished_readback, 0, 4);
        encoder.copy_buffer_to_buffer(source, 0, &buffers.readback, 0, size);
        self.queue.submit([encoder.finish()]);
        self.read(&buffers.unfinished_readback, 4, |bytes| {
            bytemuck::pod_read_unaligned(bytes)
        })
        .await
    }

    async fn read<T>(
        &self,
        buffer: &wgpu::Buffer,
        size: u64,
        f: impl FnOnce(&[u8]) -> T,
    ) -> Result<T, GpuError> {
        let slice = buffer.slice(..size);
        let (sender, receiver) = futures_channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        // Natively the callback runs during the poll; on the web the browser runs it later
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(GpuError::Poll)?;
        receiver
            .await
            .expect("map callback always runs")
            .map_err(GpuError::Readback)?;
        let view = slice.get_mapped_range().expect("mapped above");
        let value = f(&view);
        drop(view);
        buffer.unmap();
        Ok(value)
    }
}

fn compute_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    entry_point: &str,
    constants: &[(&str, f64)],
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry_point),
        layout: Some(layout),
        module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions {
            constants,
            ..Default::default()
        },
        cache: None,
    })
}
