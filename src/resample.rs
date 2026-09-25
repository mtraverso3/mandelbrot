use image::RgbImage;
use rayon::prelude::*;
use std::f32::consts::PI;

const LANCZOS_SUPPORT: f32 = 3.0;

/// Reproduces `image::imageops::resize` bit for bit.
pub fn resize_lanczos3(img: &RgbImage, width: u32, height: u32) -> RgbImage {
    if img.width() == 0 || img.height() == 0 || (width, height) == img.dimensions() {
        let mut out = RgbImage::new(width, height);
        if (width, height) == img.dimensions() {
            out.copy_from_slice(img);
        }
        return out;
    }
    let vertical = vertical_pass(img, height);
    horizontal_pass(&vertical, img.width() as usize, width)
}

struct Taps {
    start: usize,
    weights: Vec<f32>,
}

fn sinc(t: f32) -> f32 {
    let a = t * PI;
    if t == 0.0 { 1.0 } else { a.sin() / a }
}

fn lanczos3(x: f32) -> f32 {
    if x.abs() < LANCZOS_SUPPORT {
        sinc(x) * sinc(x / LANCZOS_SUPPORT)
    } else {
        0.0
    }
}

fn taps(source_len: u32, target_len: u32) -> Vec<Taps> {
    let ratio = source_len as f32 / target_len as f32;
    let scale = ratio.max(1.0);
    let support = LANCZOS_SUPPORT * scale;

    (0..target_len)
        .map(|out| {
            let center = (out as f32 + 0.5) * ratio;
            let left = ((center - support).floor() as i64).clamp(0, source_len as i64 - 1);
            let right = ((center + support).ceil() as i64).clamp(left + 1, source_len as i64);
            let center = center - 0.5;

            let mut weights: Vec<f32> = (left..right)
                .map(|i| lanczos3((i as f32 - center) / scale))
                .collect();
            let sum = weights.iter().fold(0.0, |acc, w| acc + w);
            weights.iter_mut().for_each(|w| *w /= sum);
            Taps {
                start: left as usize,
                weights,
            }
        })
        .collect()
}

fn vertical_pass(img: &RgbImage, height: u32) -> Vec<f32> {
    let row_len = img.width() as usize * 3;
    let source: &[u8] = img;
    let mut out = vec![0.0f32; row_len * height as usize];

    out.par_chunks_mut(row_len)
        .zip(taps(img.height(), height))
        .for_each(|(row, taps)| {
            for (i, &w) in taps.weights.iter().enumerate() {
                let src = &source[(taps.start + i) * row_len..][..row_len];
                for (acc, &value) in row.iter_mut().zip(src) {
                    *acc += value as f32 * w;
                }
            }
        });
    out
}

fn horizontal_pass(source: &[f32], source_width: usize, width: u32) -> RgbImage {
    let taps = taps(source_width as u32, width);
    let height = source.len() / (source_width * 3);
    let mut out = RgbImage::new(width, height as u32);

    out.par_chunks_mut(width as usize * 3)
        .zip(source.par_chunks(source_width * 3))
        .for_each(|(row, src)| {
            let (pixels, _) = row.as_chunks_mut::<3>();
            for (pixel, taps) in pixels.iter_mut().zip(&taps) {
                let mut acc = [0.0f32; 3];
                for (i, &w) in taps.weights.iter().enumerate() {
                    let offset = (taps.start + i) * 3;
                    for c in 0..3 {
                        acc[c] += src[offset + c] * w;
                    }
                }
                for c in 0..3 {
                    pixel[c] = acc[c].clamp(0.0, 255.0).round() as u8;
                }
            }
        });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::imageops::{FilterType, resize};

    fn noise(width: u32, height: u32) -> RgbImage {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        RgbImage::from_fn(width, height, |_, _| {
            let mut next = || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 56) as u8
            };
            image::Rgb([next(), next(), next()])
        })
    }

    #[test]
    fn matches_image_crate_resize() {
        for (w, h, nw, nh) in [
            (64, 48, 32, 24),
            (101, 77, 50, 38),
            (40, 30, 80, 60),
            (7, 5, 3, 2),
            (9, 9, 9, 4),
        ] {
            let img = noise(w, h);
            let expected = resize(&img, nw, nh, FilterType::Lanczos3);
            assert_eq!(
                resize_lanczos3(&img, nw, nh),
                expected,
                "{w}x{h} -> {nw}x{nh}"
            );
        }
    }

    #[test]
    fn handles_empty_and_identity_sizes() {
        let img = noise(6, 4);
        assert_eq!(resize_lanczos3(&img, 6, 4), img);
        assert_eq!(
            resize_lanczos3(&RgbImage::new(0, 0), 3, 2).dimensions(),
            (3, 2)
        );
    }
}
