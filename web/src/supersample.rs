//! Adaptive supersampling: only pixels that differ sharply from a neighbor are re-rendered
//! from a grid of points, averaged in linear light.

/// Summed channel difference between neighbors above which both are refined.
const CONTRAST: u32 = 64;

pub fn refine_pixels(pixels: &[u8], width: u32, height: u32) -> Vec<u32> {
    let (w, h) = (width as usize, height as usize);
    if pixels.len() < w * h * 4 {
        return Vec::new();
    }
    let contrast = |a: usize, b: usize| {
        let (a, b) = (&pixels[a * 4..a * 4 + 3], &pixels[b * 4..b * 4 + 3]);
        a.iter()
            .zip(b)
            .map(|(&a, &b)| a.abs_diff(b) as u32)
            .sum::<u32>()
            > CONTRAST
    };
    let mut refine = vec![false; w * h];
    let mut compare = |a: usize, b: usize| {
        if contrast(a, b) {
            refine[a] = true;
            refine[b] = true;
        }
    };
    for i in 0..w * h {
        if (i + 1) % w != 0 {
            compare(i, i + 1);
        }
        if i + w < w * h {
            compare(i, i + w);
        }
    }
    (0..refine.len() as u32)
        .filter(|&i| refine[i as usize])
        .collect()
}

pub fn points(
    pixels: &[u32],
    width: u32,
    samples: u32,
) -> impl ExactSizeIterator<Item = [f32; 2]> + '_ {
    let per_pixel = (samples * samples) as usize;
    let step = 1.0 / samples as f32;
    (0..pixels.len() * per_pixel).map(move |k| {
        let pixel = pixels[k / per_pixel];
        let point = (k % per_pixel) as u32;
        let offset = |i: u32| (i as f32 + 0.5) * step - 0.5;
        [
            (pixel % width) as f32 + offset(point % samples),
            (pixel / width) as f32 + offset(point / samples),
        ]
    })
}

pub fn average(colors: &[u8], channels: usize, count: usize) -> Vec<u8> {
    let to_linear: [f32; 256] = std::array::from_fn(|i| srgb_to_linear(i as f32 / 255.0));
    let scale = 1.0 / count as f32;
    colors
        .chunks_exact(channels * count)
        .flat_map(|run| {
            let mut sum = [0.0f32; 3];
            for pixel in run.chunks_exact(channels) {
                for c in 0..3 {
                    sum[c] += to_linear[pixel[c] as usize];
                }
            }
            let [r, g, b] = sum.map(|v| linear_to_srgb(v * scale));
            [r, g, b, 255]
        })
        .collect()
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

    fn image(width: u32, height: u32, color: impl Fn(u32, u32) -> u8) -> Vec<u8> {
        (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .flat_map(|(x, y)| {
                let v = color(x, y);
                [v, v, v, 255]
            })
            .collect()
    }

    #[test]
    fn smooth_images_need_no_refining() {
        let gradient = image(50, 40, |x, y| (x * 2 + y) as u8);
        assert!(refine_pixels(&gradient, 50, 40).is_empty());
    }

    #[test]
    fn refines_both_sides_of_edges() {
        let edge = image(5, 3, |x, _| if x < 2 { 0 } else { 255 });
        assert_eq!(refine_pixels(&edge, 5, 3), [1, 2, 6, 7, 11, 12]);
        let edge = image(3, 4, |_, y| if y < 3 { 0 } else { 255 });
        assert_eq!(refine_pixels(&edge, 3, 4), [6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn rows_do_not_wrap_around() {
        let stripes = image(4, 3, |x, _| if x == 3 { 255 } else { 0 });
        assert_eq!(refine_pixels(&stripes, 4, 3), [2, 3, 6, 7, 10, 11]);
    }

    #[test]
    fn points_cover_each_pixel_evenly() {
        let grid: Vec<_> = points(&[0, 7], 5, 2).collect();
        assert_eq!(
            grid,
            [
                [-0.25, -0.25],
                [0.25, -0.25],
                [-0.25, 0.25],
                [0.25, 0.25],
                [1.75, 0.75],
                [2.25, 0.75],
                [1.75, 1.25],
                [2.25, 1.25],
            ]
        );
        let centered: Vec<_> = points(&[7], 5, 1).collect();
        assert_eq!(centered, [[2.0, 1.0]]);
    }

    #[test]
    fn uniform_runs_keep_their_color() {
        for value in 0..=255u8 {
            let colors = [value, value / 2, 255 - value].repeat(9);
            assert_eq!(average(&colors, 3, 9), [value, value / 2, 255 - value, 255]);
        }
    }

    #[test]
    fn averages_in_linear_light() {
        let colors = [
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
        ];
        assert_eq!(average(&colors, 4, 4), [188, 188, 188, 255]);
        assert_eq!(
            average(&colors, 4, 2),
            [188, 188, 188, 255, 188, 188, 188, 255]
        );
    }
}
