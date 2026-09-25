use mandelbrot::{PRESETS, RenderOptions, Shading, Viewport, render_rows};
use wasm_bindgen::prelude::*;

/// Renders a band of rows of a `width` x `height` view as RGBA, ready for `ImageData`.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen(js_name = renderRows)]
pub fn render_rows_rgba(
    center_x: f64,
    center_y: f64,
    zoom: f64,
    width: u32,
    height: u32,
    max_iterations: u32,
    normal_shading: bool,
    first_row: u32,
    row_count: u32,
) -> Vec<u8> {
    let view = Viewport::from_f64(center_x, center_y, zoom);
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
    let row_count = row_count.min(height.saturating_sub(first_row));
    let mut rgb = vec![0; width as usize * row_count as usize * 3];
    render_rows(&view, &opts, first_row, &mut rgb);

    let (pixels, _) = rgb.as_chunks::<3>();
    pixels
        .iter()
        .flat_map(|&[r, g, b]| [r, g, b, 255])
        .collect()
}

#[wasm_bindgen(js_name = presetNames)]
pub fn preset_names() -> Vec<String> {
    PRESETS.iter().map(|(name, ..)| name.to_string()).collect()
}

/// The preset's view as `[center_x, center_y, zoom]`.
#[wasm_bindgen(js_name = presetView)]
pub fn preset_view(name: &str) -> Option<Vec<f64>> {
    mandelbrot::preset(name).map(|v| vec![v.center_x.to_f64(), v.center_y.to_f64(), v.zoom])
}
