//! Renders on the GPU with f32 perturbation around a reference orbit computed on the CPU.

use crate::ReferenceOrbit;
use crate::color::INTERIOR;
use crate::render::{RenderOptions, Renderer, Shading, Viewport};
use bytemuck::{Pod, Zeroable};
use image::RgbImage;
use rayon::prelude::*;
use std::fmt;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// Past this, pixel offsets approach the smallest normal f32.
pub const GPU_MAX_ZOOM: f64 = 1e30;
const WORKGROUP_SIZE: u32 = 8;
const BAND_PIXELS: usize = 1 << 20;
const INTERIOR_SAMPLE: u32 = u32::MAX;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    width: u32,
    first_row: u32,
    rows: u32,
    max_iterations: u32,
    last: u32,
    track_derivative: u32,
    pixel_size: f32,
    left: f32,
    top: f32,
    _padding: [u32; 3],
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
    pipeline: wgpu::ComputePipeline,
    adapter_name: String,
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
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("perturbation"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Self {
            device,
            queue,
            pipeline,
            adapter_name: adapter.get_info().name,
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
        let samples = self.samples(&renderer, BAND_PIXELS)?;
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

    /// Renders in bands of rows so buffers stay small and each dispatch stays short.
    fn samples(&self, renderer: &Renderer, band_pixels: usize) -> Result<Vec<Sample>, GpuError> {
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
        let points: Vec<[f32; 2]> = orbit
            .points()
            .iter()
            .map(|&(r, i)| [r as f32, i as f32])
            .collect();

        let band_rows = (band_pixels / width).clamp(1, height);
        let band_size = (band_rows * width * size_of::<Sample>()) as u64;
        let orbit_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("orbit"),
                contents: bytemuck::cast_slice(&points),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let params_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("params"),
            size: size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("samples"),
            size: band_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: band_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: orbit_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output.as_entire_binding(),
                },
            ],
        });

        let mut samples = Vec::with_capacity(width * height);
        for first_row in (0..height).step_by(band_rows) {
            let rows = band_rows.min(height - first_row);
            let params = Params {
                width: width as u32,
                first_row: first_row as u32,
                rows: rows as u32,
                max_iterations: opts.max_iterations.min(u32::MAX as usize - 1) as u32,
                last: (points.len() - 1) as u32,
                track_derivative: (opts.shading == Shading::Normal) as u32,
                pixel_size: renderer.pixel_size() as f32,
                left: width as f32 / 2.0,
                top: height as f32 / 2.0,
                _padding: [0; 3],
            };
            self.queue
                .write_buffer(&params_buffer, 0, bytemuck::bytes_of(&params));

            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(
                    (width as u32).div_ceil(WORKGROUP_SIZE),
                    (rows as u32).div_ceil(WORKGROUP_SIZE),
                    1,
                );
            }
            let size = (rows * width * size_of::<Sample>()) as u64;
            encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, size);
            self.queue.submit([encoder.finish()]);

            let slice = readback.slice(..size);
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
            samples.extend_from_slice(bytemuck::cast_slice(&view));
            drop(view);
            readback.unmap();
        }
        Ok(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset;

    fn escapes(gpu: &GpuRenderer, renderer: &Renderer, band_pixels: usize) -> Vec<Option<usize>> {
        gpu.samples(renderer, band_pixels)
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

    /// f32 rounding reshuffles iteration counts in chaotic texture, but must never turn an
    /// escaping pixel into an interior one or back.
    fn assert_close_to_exact(view: Viewport, max_iterations: usize, max_mismatch: f64) {
        let gpu = GpuRenderer::new().unwrap();
        let opts = RenderOptions {
            width: 96,
            height: 64,
            max_iterations,
            shading: Shading::Normal,
        };
        let renderer = Renderer::new(&view, &opts);
        let actual = escapes(&gpu, &renderer, BAND_PIXELS);
        let expected = exact_escapes(&renderer);
        assert!(
            expected.iter().any(|e| *e != expected[0]),
            "grid should not be uniform"
        );
        for (i, (a, e)) in actual.iter().zip(&expected).enumerate() {
            assert_eq!(a.is_some(), e.is_some(), "pixel {i}: {a:?} vs {e:?}");
        }
        let mismatches = actual.iter().zip(&expected).filter(|(a, e)| a != e).count();
        eprintln!(
            "zoom {:e}: {mismatches}/{} differ",
            view.zoom,
            expected.len()
        );
        assert!(
            mismatches as f64 <= max_mismatch * expected.len() as f64,
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
    fn close_to_exact_at_deepest_zoom() {
        let view = Viewport {
            center_x: "0".parse().unwrap(),
            center_y: "1".parse().unwrap(),
            zoom: GPU_MAX_ZOOM,
        };
        assert_close_to_exact(view, 3000, 0.01);
    }

    #[test]
    fn bands_match_a_single_dispatch() {
        let gpu = GpuRenderer::new().unwrap();
        let opts = RenderOptions {
            width: 37,
            height: 23,
            max_iterations: 400,
            shading: Shading::Normal,
        };
        let renderer = Renderer::new(&Viewport::from_f64(-0.7453, 0.1127, 150.0), &opts);
        assert_eq!(
            escapes(&gpu, &renderer, 37 * 5),
            escapes(&gpu, &renderer, BAND_PIXELS)
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
