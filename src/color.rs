use image::Rgb;

const PALETTE: [[u8; 3]; 16] = [
    [66, 30, 15],
    [25, 7, 26],
    [9, 1, 47],
    [4, 4, 73],
    [0, 7, 100],
    [12, 44, 138],
    [24, 82, 177],
    [57, 125, 209],
    [134, 181, 229],
    [211, 236, 248],
    [241, 233, 191],
    [248, 201, 95],
    [255, 170, 0],
    [204, 128, 0],
    [153, 87, 0],
    [106, 52, 3],
];

pub const INTERIOR: Rgb<u8> = Rgb([255, 255, 255]);

const LIGHT_HEIGHT: f64 = 1.0;
const AMBIENT_LIGHT: f64 = 0.3;
const BRIGHTNESS_BOOST: f64 = 1.3;

pub fn palette(position: f64) -> Rgb<u8> {
    let index = position as usize;
    let from = PALETTE[index % PALETTE.len()];
    let to = PALETTE[(index + 1) % PALETTE.len()];
    let t = position % 1.0;
    Rgb(std::array::from_fn(|i| {
        (from[i] as f64 * (1.0 - t) + to[i] as f64 * t) as u8
    }))
}

/// Lambert-style shading from the direction of the distance-estimate normal.
pub fn shade(base: Rgb<u8>, normal: (f64, f64), light: (f64, f64)) -> Rgb<u8> {
    let t = normal.0 * light.0 + normal.1 * light.1 + LIGHT_HEIGHT;
    let t = t / (1.0 + LIGHT_HEIGHT);
    let light_factor = (t * (1.0 - AMBIENT_LIGHT) + AMBIENT_LIGHT).clamp(0.0, 1.0);
    Rgb(base
        .0
        .map(|channel| ((channel as f64 * light_factor * BRIGHTNESS_BOOST) as u32).min(255) as u8))
}
