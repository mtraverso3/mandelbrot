mod bla;
mod color;
mod coordinate;
#[cfg(feature = "gpu")]
mod gpu;
mod perturbation;
mod render;
mod resample;

pub use color::{Outline, Palette};
pub use coordinate::{Coordinate, ParseCoordinateError};
#[cfg(feature = "gpu")]
pub use gpu::{GpuError, GpuRenderer};
pub use perturbation::ReferenceOrbit;
pub use render::{
    ColorBands, MAX_AUTO_ITERATIONS, MAX_ZOOM, PERTURBATION_ZOOM, RenderOptions, Renderer, Shading,
    Viewport, auto_iterations, render, render_rows,
};
pub use resample::resize_lanczos3;

pub fn downsample(img: &image::RgbImage) -> image::RgbImage {
    resize_lanczos3(img, img.width() / 2, img.height() / 2)
}

pub const PRESETS: [(&str, &str, &str, f64); 10] = [
    ("mandelbrot", "-0.75", "0", 1.0),
    ("mini-mandelbrot", "-1.249559196", "0.030466443", 1.73e6),
    ("spirals", "-1.2494989", "0.0303330", 7.437e7),
    ("quad-spiral", "-0.4621603", "-0.5823998", 2.633507e7),
    ("elephant-valley", "0.2855", "0.0114", 400.0),
    ("seahorse-spiral", "-0.7436438870", "0.1318259042", 5e4),
    ("triple-spiral", "-0.0886", "0.6547", 3000.0),
    ("double-spiral", "-0.774680610627", "-0.137416885604", 1e6),
    ("julia-island", "-1.768778833", "-0.001738996", 3e6),
    ("tendrils", "-0.1011", "0.9563", 1000.0),
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
