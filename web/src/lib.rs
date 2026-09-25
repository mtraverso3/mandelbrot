use mandelbrot::{Coordinate, MAX_ZOOM, PRESETS, RenderOptions, Renderer, Shading, Viewport};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    // Every band of a view shares its reference orbit and color bands, so each worker keeps
    // the last renderer to build the next one from.
    static LAST: RefCell<Option<Renderer>> = const { RefCell::new(None) };
}

fn viewport(x: &str, y: &str, zoom: f64) -> Result<Viewport, JsError> {
    Ok(Viewport {
        center_x: x.parse()?,
        center_y: y.parse()?,
        zoom,
    })
}

fn to_strings(view: &Viewport) -> Vec<String> {
    vec![view.center_x.to_string(), view.center_y.to_string()]
}

/// Renders a band of rows of a `width` x `height` view as RGBA, ready for `ImageData`.
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
    let opts = RenderOptions {
        width,
        height,
        max_iterations: max_iterations as usize,
        shading: if normal_shading {
            Shading::Normal
        } else {
            Shading::Flat
        },
    };
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

/// The preset's view as `[center_x, center_y, zoom]`.
#[wasm_bindgen(js_name = presetView)]
pub fn preset_view(name: &str) -> Option<Vec<String>> {
    mandelbrot::preset(name).map(|view| {
        let mut parts = to_strings(&view);
        parts.push(view.zoom.to_string());
        parts
    })
}

/// The center moved by `(dx, dy)`, as `[center_x, center_y]`.
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

/// The view zoomed by `factor` around the point `(dx, dy)` from the center, as
/// `[center_x, center_y, zoom]`.
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

/// `a - b` as the nearest `f64`.
#[wasm_bindgen]
pub fn difference(a: &str, b: &str) -> Result<f64, JsError> {
    Ok(a.parse::<Coordinate>()?.difference(&b.parse()?))
}

/// The canonical form of a decimal coordinate, or `undefined` if it is not one.
#[wasm_bindgen(js_name = normalizeCoordinate)]
pub fn normalize_coordinate(value: &str) -> Option<String> {
    value.parse::<Coordinate>().ok().map(|c| c.to_string())
}

#[wasm_bindgen(js_name = maxZoom)]
pub fn max_zoom() -> f64 {
    MAX_ZOOM
}
