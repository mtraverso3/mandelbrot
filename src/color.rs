use image::Rgb;

pub(crate) const PALETTE_SIZE: usize = 16;

const CLASSIC: [[u8; 3]; PALETTE_SIZE] = [
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

const FIRE: [[u8; 3]; PALETTE_SIZE] = [
    [20, 0, 0],
    [60, 4, 0],
    [110, 10, 0],
    [160, 25, 0],
    [205, 50, 0],
    [235, 90, 5],
    [250, 135, 20],
    [255, 180, 50],
    [255, 220, 110],
    [255, 245, 190],
    [250, 210, 120],
    [235, 160, 50],
    [200, 100, 15],
    [150, 50, 5],
    [95, 20, 0],
    [50, 5, 0],
];

const OCEAN: [[u8; 3]; PALETTE_SIZE] = [
    [2, 10, 30],
    [4, 24, 58],
    [6, 42, 88],
    [8, 64, 116],
    [10, 90, 140],
    [18, 118, 160],
    [34, 148, 176],
    [64, 178, 190],
    [110, 206, 204],
    [170, 230, 220],
    [120, 200, 210],
    [70, 160, 190],
    [40, 120, 165],
    [22, 82, 130],
    [12, 50, 92],
    [6, 26, 56],
];

const MONO: [[u8; 3]; PALETTE_SIZE] = [
    [14, 14, 14],
    [23, 23, 23],
    [47, 47, 47],
    [84, 84, 84],
    [127, 127, 127],
    [170, 170, 170],
    [207, 207, 207],
    [231, 231, 231],
    [240, 240, 240],
    [231, 231, 231],
    [207, 207, 207],
    [170, 170, 170],
    [127, 127, 127],
    [84, 84, 84],
    [47, 47, 47],
    [23, 23, 23],
];

const SUNSET: [[u8; 3]; PALETTE_SIZE] = [
    [16, 8, 40],
    [40, 14, 72],
    [72, 20, 100],
    [110, 28, 120],
    [150, 36, 128],
    [192, 52, 122],
    [226, 80, 108],
    [246, 118, 94],
    [252, 160, 90],
    [254, 204, 120],
    [255, 236, 180],
    [236, 178, 140],
    [196, 120, 130],
    [140, 70, 120],
    [80, 36, 90],
    [36, 16, 58],
];

pub const INTERIOR: Rgb<u8> = Rgb([255, 255, 255]);

pub(crate) const OUTLINE_WIDTH: f64 = 0.5;

const LIGHT_HEIGHT: f64 = 1.0;
const AMBIENT_LIGHT: f64 = 0.3;
const BRIGHTNESS_BOOST: f64 = 1.3;

/// Colors that escapes cycle through, by their position in the color bands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Palette {
    #[default]
    Classic,
    Fire,
    Ocean,
    Mono,
    Sunset,
}

impl Palette {
    pub const ALL: [Palette; 5] = [
        Palette::Classic,
        Palette::Fire,
        Palette::Ocean,
        Palette::Mono,
        Palette::Sunset,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Palette::Classic => "classic",
            Palette::Fire => "fire",
            Palette::Ocean => "ocean",
            Palette::Mono => "mono",
            Palette::Sunset => "sunset",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|palette| palette.name() == name)
    }

    pub(crate) fn colors(self) -> &'static [[u8; 3]; PALETTE_SIZE] {
        match self {
            Palette::Classic => &CLASSIC,
            Palette::Fire => &FIRE,
            Palette::Ocean => &OCEAN,
            Palette::Mono => &MONO,
            Palette::Sunset => &SUNSET,
        }
    }

    pub(crate) fn color(self, position: f64) -> Rgb<u8> {
        let colors = self.colors();
        let position = position.rem_euclid(PALETTE_SIZE as f64);
        let index = position as usize % PALETTE_SIZE;
        let from = colors[index];
        let to = colors[(index + 1) % PALETTE_SIZE];
        let t = position.fract();
        Rgb(std::array::from_fn(|i| {
            (from[i] as f64 * (1.0 - t) + to[i] as f64 * t) as u8
        }))
    }
}

/// What escapes too close to the set to resolve fade into, if anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Outline {
    #[default]
    Off,
    Dark,
    /// The interior's color, as if the filaments were drawn in
    Light,
}

impl Outline {
    pub const ALL: [Outline; 3] = [Outline::Off, Outline::Dark, Outline::Light];

    pub fn name(self) -> &'static str {
        match self {
            Outline::Off => "off",
            Outline::Dark => "dark",
            Outline::Light => "light",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|outline| outline.name() == name)
    }

    pub(crate) fn color(self) -> Option<Rgb<u8>> {
        match self {
            Outline::Off => None,
            Outline::Dark => Some(Rgb([0, 0, 0])),
            Outline::Light => Some(INTERIOR),
        }
    }
}

pub fn shade(base: Rgb<u8>, normal: (f64, f64), light: (f64, f64)) -> Rgb<u8> {
    let t = normal.0 * light.0 + normal.1 * light.1 + LIGHT_HEIGHT;
    let t = t / (1.0 + LIGHT_HEIGHT);
    let light_factor = (t * (1.0 - AMBIENT_LIGHT) + AMBIENT_LIGHT).clamp(0.0, 1.0);
    Rgb(base
        .0
        .map(|channel| ((channel as f64 * light_factor * BRIGHTNESS_BOOST) as u32).min(255) as u8))
}

/// Fades `rgb` into `edge` as `pixels`, an escape's estimated distance from the set in
/// pixels, drops below [`OUTLINE_WIDTH`].
pub fn outline(rgb: Rgb<u8>, edge: Rgb<u8>, pixels: f64) -> Rgb<u8> {
    let t = (pixels / OUTLINE_WIDTH).clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    Rgb(std::array::from_fn(|i| {
        (edge[i] as f64 + (rgb[i] as f64 - edge[i] as f64) * t) as u8
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for palette in Palette::ALL {
            assert_eq!(Palette::from_name(palette.name()), Some(palette));
        }
        assert_eq!(Palette::from_name("plaid"), None);
    }

    #[test]
    fn outline_names_round_trip() {
        for outline in Outline::ALL {
            assert_eq!(Outline::from_name(outline.name()), Some(outline));
        }
        assert_eq!(Outline::from_name("thick"), None);
    }

    #[test]
    fn outline_fades_into_the_edge_near_the_set() {
        let rgb = Rgb([200, 100, 50]);
        let edge = Rgb([0, 0, 0]);
        assert_eq!(outline(rgb, edge, 0.0), edge);
        assert_eq!(outline(rgb, edge, OUTLINE_WIDTH), rgb);
        assert_eq!(outline(rgb, edge, 1e300), rgb);
        let half = outline(rgb, edge, OUTLINE_WIDTH / 2.0);
        assert_eq!(half, Rgb([100, 50, 25]));
    }

    #[test]
    fn palettes_cycle_through_their_colors() {
        for palette in Palette::ALL {
            let colors = palette.colors();
            for (i, color) in colors.iter().enumerate() {
                assert_eq!(palette.color(i as f64).0, *color);
                assert_eq!(palette.color((i + PALETTE_SIZE) as f64).0, *color);
            }
            assert_eq!(
                palette.color(-0.5),
                palette.color(PALETTE_SIZE as f64 - 0.5)
            );
        }
    }
}
