use crate::{options, viewport};
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

    /// Calls `on_rows(firstRow, rgba)` for each band as it finishes. Resolves to whether the
    /// render finished, rather than being cancelled by a later one.
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
        on_rows: js_sys::Function,
    ) -> Result<js_sys::Promise, JsError> {
        let view = viewport(center_x, center_y, zoom)?;
        let opts = options(width, height, max_iterations, normal_shading);
        self.cancel();
        let inner = self.inner.clone();
        let generation = inner.generation.get();
        Ok(wasm_bindgen_futures::future_to_promise(async move {
            let renderer = Renderer::reusing(&view, &opts, inner.last.borrow().as_ref());
            let finished = inner
                .gpu
                .render_rows(
                    &renderer,
                    || inner.generation.get() != generation,
                    |first_row, rgba| {
                        let rgba = js_sys::Uint8Array::from(rgba);
                        let _ = on_rows.call2(&JsValue::NULL, &first_row.into(), &rgba);
                    },
                )
                .await
                .map_err(|e| JsError::new(&e.to_string()))?;
            *inner.last.borrow_mut() = Some(renderer);
            Ok(finished.into())
        }))
    }
}
