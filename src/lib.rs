mod bla;
mod color;
mod coordinate;
mod perturbation;
mod render;
mod resample;

pub use coordinate::{Coordinate, ParseCoordinateError};
pub use perturbation::ReferenceOrbit;
pub use render::{
    MAX_ZOOM, PERTURBATION_ZOOM, RenderOptions, Renderer, Shading, Viewport, render, render_rows,
};
pub use resample::resize_lanczos3;

pub fn downsample(img: &image::RgbImage) -> image::RgbImage {
    resize_lanczos3(img, img.width() / 2, img.height() / 2)
}

pub const PRESETS: [(&str, &str, &str, f64); 4] = [
    ("mandelbrot", "-0.75", "0", 1.0),
    ("mini-mandelbrot", "-1.249559196", "0.030466443", 1.73e6),
    ("spirals", "-1.2494989", "0.0303330", 7.437e7),
    ("quad-spiral", "-0.4621603", "-0.5823998", 2.633507e7),
];

pub fn preset(name: &str) -> Option<Viewport> {
    PRESETS
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, x, y, zoom)| Viewport {
            center_x: x.parse().expect("valid preset"),
            center_y: y.parse().expect("valid preset"),
            zoom: *zoom,
        })
}
