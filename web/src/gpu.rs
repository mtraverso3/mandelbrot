use crate::{bands, options, supersample, viewport};
use mandelbrot::{GpuRenderer, Renderer};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Renders through WebGPU. Starting a render cancels the one before it.
#[wasm_bindgen]
pub struct GpuViewer {
    inner: Rc<Inner>,
}

struct Inner {
    gpu: GpuRenderer,
    generation: Cell<u32>,
    // A preview and its full render share a reference orbit and color bands
    last: RefCell<Option<Renderer>>,
}

#[wasm_bindgen]
impl GpuViewer {
    /// Rejects when the browser has no usable WebGPU adapter.
    pub async fn create() -> Result<GpuViewer, JsError> {
        let gpu = GpuRenderer::new_async()
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self {
            inner: Rc::new(Inner {
                gpu,
                generation: Cell::new(0),
                last: RefCell::new(None),
            }),
        })
    }

    #[wasm_bindgen(getter, js_name = adapterName)]
    pub fn adapter_name(&self) -> String {
        self.inner.gpu.adapter_name().to_string()
    }

    pub fn cancel(&self) {
        let generation = &self.inner.generation;
        generation.set(generation.get().wrapping_add(1));
    }

    /// Resolves to the iteration limit the view needs, or `undefined` if a later call
    /// cancelled it.
    #[wasm_bindgen(js_name = autoIterations)]
    pub fn auto_iterations(
        &self,
        center_x: &str,
        center_y: &str,
        zoom: f64,
        width: u32,
        height: u32,
    ) -> Result<js_sys::Promise, JsError> {
        let view = viewport(center_x, center_y, zoom)?;
        self.cancel();
        let inner = self.inner.clone();
        let generation = inner.generation.get();
        Ok(wasm_bindgen_futures::future_to_promise(async move {
            let limit = inner
                .gpu
                .auto_iterations_async(&view, width, height, || {
                    inner.generation.get() != generation
                })
                .await
                .map_err(|e| JsError::new(&e.to_string()))?;
            Ok(limit.map_or(JsValue::UNDEFINED, |limit| (limit as u32).into()))
        }))
    }

    /// Calls `on_rows(firstRow, rgba, [bandScale, bandPhase])` for each band as it finishes.
    /// Resolves to whether the render finished, rather than being cancelled by a later one.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        center_x: &str,
        center_y: &str,
        zoom: f64,
        width: u32,
        height: u32,
        max_iterations: u32,
        normal_shading: bool,
        palette: &str,
        band_scale: f64,
        band_phase: f64,
        on_rows: js_sys::Function,
    ) -> Result<js_sys::Promise, JsError> {
        let bands = bands(band_scale, band_phase);
        let view = viewport(center_x, center_y, zoom)?;
        let opts = options(width, height, max_iterations, normal_shading, palette);
        self.cancel();
        let inner = self.inner.clone();
        let generation = inner.generation.get();
        Ok(wasm_bindgen_futures::future_to_promise(async move {
            let renderer = Renderer::continuing(&view, &opts, inner.last.borrow().as_ref(), bands);
            let used = renderer.color_bands();
            let used = js_sys::Float64Array::from(&[used.scale, used.phase][..]);
            let finished = inner
                .gpu
                .render_rows(
                    &renderer,
                    || inner.generation.get() != generation,
                    |first_row, rgba| {
                        let rgba = js_sys::Uint8Array::from(rgba);
                        let _ = on_rows.call3(&JsValue::NULL, &first_row.into(), &rgba, &used);
                    },
                )
                .await
                .map_err(|e| JsError::new(&e.to_string()))?;
            *inner.last.borrow_mut() = Some(renderer);
            Ok(finished.into())
        }))
    }

    /// Re-renders `pixels` from `samples`×`samples` points each, calling
    /// `on_pixels(first, rgba)` with their averaged colors as bands finish.
    #[allow(clippy::too_many_arguments)]
    pub fn refine(
        &self,
        center_x: &str,
        center_y: &str,
        zoom: f64,
        width: u32,
        height: u32,
        samples: u32,
        max_iterations: u32,
        normal_shading: bool,
        palette: &str,
        band_scale: f64,
        band_phase: f64,
        pixels: Vec<u32>,
        round: u32,
        on_pixels: js_sys::Function,
    ) -> Result<js_sys::Promise, JsError> {
        let bands = bands(band_scale, band_phase);
        let view = viewport(center_x, center_y, zoom)?;
        let opts = options(width, height, max_iterations, normal_shading, palette);
        let samples = samples.max(1);
        let per_pixel = (samples * samples) as usize;
        self.cancel();
        let inner = self.inner.clone();
        let generation = inner.generation.get();
        Ok(wasm_bindgen_futures::future_to_promise(async move {
            let renderer = Renderer::continuing(&view, &opts, inner.last.borrow().as_ref(), bands);
            let mut pending = Vec::new();
            let mut done = 0u32;
            let finished = inner
                .gpu
                .render_points(
                    &renderer,
                    supersample::points(&pixels, width, samples, round),
                    || inner.generation.get() != generation,
                    |_, rgba| {
                        pending.extend_from_slice(rgba);
                        let whole = pending.len() / (4 * per_pixel) * 4 * per_pixel;
                        let colors = supersample::average(&pending[..whole], 4, per_pixel);
                        pending.drain(..whole);
                        let count = (colors.len() / 4) as u32;
                        let colors = js_sys::Uint8Array::from(&colors[..]);
                        let _ = on_pixels.call2(&JsValue::NULL, &done.into(), &colors);
                        done += count;
                    },
                )
                .await
                .map_err(|e| JsError::new(&e.to_string()))?;
            *inner.last.borrow_mut() = Some(renderer);
            Ok(finished.into())
        }))
    }
}
