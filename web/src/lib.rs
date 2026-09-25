#[cfg(target_arch = "wasm32")]
mod gpu;

use mandelbrot::{Coordinate, MAX_ZOOM, PRESETS, RenderOptions, Renderer, Shading, Viewport};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    // Bands of one view share its reference orbit and color bands
    static LAST: RefCell<Option<Renderer>> = const { RefCell::new(None) };
}

fn viewport(x: &str, y: &str, zoom: f64) -> Result<Viewport, JsError> {
    Ok(Viewport {
        center_x: x.parse()?,
        center_y: y.parse()?,
        zoom,
    })
}

fn options(width: u32, height: u32, max_iterations: u32, normal_shading: bool) -> RenderOptions {
    RenderOptions {
        width,
        height,
        max_iterations: max_iterations as usize,
        shading: if normal_shading {
            Shading::Normal
        } else {
            Shading::Flat
        },
    }
}

fn to_strings(view: &Viewport) -> Vec<String> {
    vec![view.center_x.to_string(), view.center_y.to_string()]
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = renderRows)]
pub fn render_rows_rgba(
    center_x: &str,
    center_y: &str,
    zoom: f64,
    width: u32,
    height: u32,
    max_iterations: u32,
    normal_shading: bool,
    first_row: u32,
    row_count: u32,
) -> Result<Vec<u8>, JsError> {
    let view = viewport(center_x, center_y, zoom)?;
    let opts = options(width, height, max_iterations, normal_shading);
    LAST.with_borrow_mut(|last| {
        let renderer = Renderer::reusing(&view, &opts, last.as_ref());
        let row_count = row_count.min(height.saturating_sub(first_row));
        let mut rgb = vec![0; width as usize * row_count as usize * 3];
        renderer.render_rows(first_row, &mut rgb);
        *last = Some(renderer);

        let (pixels, _) = rgb.as_chunks::<3>();
        Ok(pixels
            .iter()
            .flat_map(|&[r, g, b]| [r, g, b, 255])
            .collect())
    })
}

#[wasm_bindgen(js_name = presetNames)]
pub fn preset_names() -> Vec<String> {
    PRESETS.iter().map(|(name, ..)| name.to_string()).collect()
}

/// `[center_x, center_y, zoom]`
#[wasm_bindgen(js_name = presetView)]
pub fn preset_view(name: &str) -> Option<Vec<String>> {
    mandelbrot::preset(name).map(|view| {
        let mut parts = to_strings(&view);
        parts.push(view.zoom.to_string());
        parts
    })
}

/// `[center_x, center_y]`
#[wasm_bindgen]
pub fn pan(
    center_x: &str,
    center_y: &str,
    zoom: f64,
    dx: f64,
    dy: f64,
) -> Result<Vec<String>, JsError> {
    Ok(to_strings(&viewport(center_x, center_y, zoom)?.pan(dx, dy)))
}

/// `[center_x, center_y, zoom]`
#[wasm_bindgen(js_name = zoomAt)]
pub fn zoom_at(
    center_x: &str,
    center_y: &str,
    zoom: f64,
    dx: f64,
    dy: f64,
    factor: f64,
) -> Result<Vec<String>, JsError> {
    let view = viewport(center_x, center_y, zoom)?.zoom_at(dx, dy, factor);
    let mut parts = to_strings(&view);
    parts.push(view.zoom.to_string());
    Ok(parts)
}

#[wasm_bindgen]
pub fn difference(a: &str, b: &str) -> Result<f64, JsError> {
    Ok(a.parse::<Coordinate>()?.difference(&b.parse()?))
}

#[wasm_bindgen(js_name = normalizeCoordinate)]
pub fn normalize_coordinate(value: &str) -> Option<String> {
    value.parse::<Coordinate>().ok().map(|c| c.to_string())
}

#[wasm_bindgen(js_name = autoIterations)]
pub fn auto_iterations(
    center_x: &str,
    center_y: &str,
    zoom: f64,
    width: u32,
    height: u32,
) -> Result<u32, JsError> {
    let view = viewport(center_x, center_y, zoom)?;
    Ok(mandelbrot::auto_iterations(&view, width, height) as u32)
}

#[wasm_bindgen(js_name = maxZoom)]
pub fn max_zoom() -> f64 {
    MAX_ZOOM
}
