mod mandelbrot;

pub use mandelbrot::{RenderOptions, Shading, Viewport, downsample, render};

pub const PRESETS: [(&str, Viewport); 4] = [
    ("mandelbrot", Viewport { center_x: -0.75, center_y: 0.0, zoom: 1.0 }),
    ("mini-mandelbrot", Viewport { center_x: -1.249559196, center_y: 0.030466443, zoom: 1.73e6 }),
    ("spirals", Viewport { center_x: -1.2494989, center_y: 0.0303330, zoom: 7.437000e7 }),
    ("quad-spiral", Viewport { center_x: -4.621603e-1, center_y: -5.823998e-1, zoom: 2.633507e7 }),
];

pub fn preset(name: &str) -> Option<Viewport> {
    PRESETS.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
}
