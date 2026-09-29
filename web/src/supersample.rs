//! Averages supersampled renders in linear light, keeping thin bright filaments bright.

pub fn downsample(pixels: &[u8], channels: usize, width: usize, factor: usize) -> Vec<u8> {
    let source_row = width * factor * channels;
    if source_row == 0 {
        return Vec::new();
    }
    if factor == 1 {
        return pixels
            .chunks_exact(channels)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect();
    }
    let to_linear: [f32; 256] = std::array::from_fn(|i| srgb_to_linear(i as f32 / 255.0));
    let scale = 1.0 / (factor * factor) as f32;
    let mut out = Vec::with_capacity(pixels.len() / source_row / factor * width * 4);
    let mut sums = vec![[0.0f32; 3]; width];
    for block in pixels.chunks_exact(source_row * factor) {
        sums.fill([0.0; 3]);
        for row in block.chunks_exact(source_row) {
            for (x, pixel) in row.chunks_exact(channels).enumerate() {
                let sum = &mut sums[x / factor];
                for c in 0..3 {
                    sum[c] += to_linear[pixel[c] as usize];
                }
            }
        }
        for sum in &sums {
            out.extend(sum.map(|v| linear_to_srgb(v * scale)));
            out.push(255);
        }
    }
    out
}

/// Averages blocks of rows as they complete, from bands that need not align with them.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct BlockRows {
    width: usize,
    factor: usize,
    channels: usize,
    pending: Vec<u8>,
    next_row: u32,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl BlockRows {
    pub fn new(width: usize, factor: usize, channels: usize) -> Self {
        Self {
            width,
            factor,
            channels,
            pending: Vec::new(),
            next_row: 0,
        }
    }

    pub fn push(&mut self, rows: &[u8]) -> Option<(u32, Vec<u8>)> {
        self.pending.extend_from_slice(rows);
        let block = self.width * self.factor * self.factor * self.channels;
        let blocks = self.pending.len().checked_div(block)?;
        if blocks == 0 {
            return None;
        }
        let first_row = self.next_row;
        let pixels = downsample(
            &self.pending[..blocks * block],
            self.channels,
            self.width,
            self.factor,
        );
        self.pending.drain(..blocks * block);
        self.next_row += blocks as u32;
        Some((first_row, pixels))
    }
}

fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> u8 {
    let v = if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_blocks_keep_their_color() {
        for value in 0..=255u8 {
            let pixels = [value, value / 2, 255 - value].repeat(9);
            assert_eq!(
                downsample(&pixels, 3, 1, 3),
                [value, value / 2, 255 - value, 255]
            );
        }
    }

    #[test]
    fn averages_in_linear_light() {
        let pixels = [0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255];
        assert_eq!(downsample(&pixels, 4, 1, 1), pixels);
        let checker = [
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
        ];
        assert_eq!(downsample(&checker, 4, 1, 2), [188, 188, 188, 255]);
    }

    #[test]
    fn averages_each_block_separately() {
        let row = [0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255];
        let pixels = row.repeat(2);
        assert_eq!(
            downsample(&pixels, 3, 2, 2),
            [0, 0, 0, 255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn bands_split_anywhere_match_one_downsample() {
        let (width, factor, rows) = (5, 3, 4);
        let pixels: Vec<u8> = (0..width * factor * rows * factor * 4)
            .map(|i| (i * 37 % 251) as u8)
            .collect();
        let whole = downsample(&pixels, 4, width, factor);

        let mut blocks = BlockRows::new(width, factor, 4);
        let mut out = Vec::new();
        let source_row = width * factor * 4;
        for band in pixels.chunks(source_row * 2) {
            if let Some((first_row, rows)) = blocks.push(band) {
                assert_eq!(first_row as usize * width * 4, out.len());
                out.extend(rows);
            }
        }
        assert_eq!(out, whole);
    }
}
