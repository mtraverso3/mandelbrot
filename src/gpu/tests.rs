use super::*;
use crate::{RenderOptions, Viewport, preset, render};

fn samples(gpu: &GpuRenderer, renderer: &Renderer, chunking: Chunking) -> Vec<Sample> {
    let mut samples = Vec::new();
    let finished = pollster::block_on(gpu.run(
        renderer,
        chunking,
        Output::Samples,
        &|| false,
        &mut |_, bytes| samples.extend(bytemuck::pod_collect_to_vec::<u8, Sample>(bytes)),
    ));
    assert!(finished.unwrap());
    samples
}

fn escapes(gpu: &GpuRenderer, renderer: &Renderer, chunking: Chunking) -> Vec<Option<usize>> {
    samples(gpu, renderer, chunking)
        .iter()
        .map(|s| (s.iterations != u32::MAX).then_some(s.iterations as usize))
        .collect()
}

fn exact_escapes(renderer: &Renderer) -> Vec<Option<usize>> {
    let opts = renderer.options();
    let orbit = ReferenceOrbit::compute(renderer.view(), opts.max_iterations, 1.0);
    let pixel = renderer.pixel_size();
    let (left, top) = (opts.width as f64 / 2.0, opts.height as f64 / 2.0);
    (0..opts.height as usize * opts.width as usize)
        .map(|i| {
            let x = (i % opts.width as usize) as f64 - left;
            let y = (i / opts.width as usize) as f64 - top;
            orbit.escape_unskipped(x * pixel, y * pixel, opts.max_iterations)
        })
        .collect()
}

/// f32 rounding reshuffles iteration counts in chaotic texture, and can flip pixels that
/// escape right at the iteration limit, but must not change what is inside the set.
fn assert_close_to_exact(view: Viewport, max_iterations: usize, max_mismatch: f64) {
    let gpu = GpuRenderer::new().unwrap();
    let opts = RenderOptions {
        width: 96,
        height: 64,
        max_iterations,
        shading: Shading::Normal,
    };
    let renderer = Renderer::new(&view, &opts);
    let actual = escapes(&gpu, &renderer, CHUNKING);
    let expected = exact_escapes(&renderer);
    assert!(
        expected.iter().any(|e| *e != expected[0]),
        "grid should not be uniform"
    );
    let pairs = || actual.iter().zip(&expected);
    let flipped = pairs().filter(|(a, e)| a.is_some() != e.is_some()).count();
    let mismatches = pairs().filter(|(a, e)| a != e).count();
    eprintln!(
        "zoom {:e}: {mismatches}/{} differ, {flipped} flipped",
        view.zoom,
        expected.len()
    );
    let pixels = expected.len() as f64;
    assert!(flipped as f64 <= 0.001 * pixels, "{flipped} pixels flipped");
    assert!(
        mismatches as f64 <= max_mismatch * pixels,
        "{mismatches} pixels differ"
    );
}

#[test]
fn close_to_exact_on_full_view() {
    assert_close_to_exact(preset("mandelbrot").unwrap(), 1500, 0.01);
}

#[test]
fn close_to_exact_in_chaotic_spirals() {
    assert_close_to_exact(preset("spirals").unwrap(), 6000, 0.25);
}

#[test]
fn close_to_exact_when_reference_escapes_early() {
    assert_close_to_exact(Viewport::from_f64(0.2501, 0.0, 1e3), 2000, 0.01);
}

#[test]
fn close_to_exact_around_a_minibrot() {
    assert_close_to_exact(preset("mini-mandelbrot").unwrap(), 24000, 0.25);
}

fn deep_seahorse() -> Viewport {
    Viewport {
        center_x: "-1.24949889563508492587065068503213228909045011806661"
            .parse()
            .unwrap(),
        center_y: "0.03033300303590165779311010118330780526875599532123"
            .parse()
            .unwrap(),
        zoom: 4.7374e16,
    }
}

#[test]
fn close_to_exact_when_skipping_at_deep_zoom() {
    assert_close_to_exact(deep_seahorse(), 24000, 0.15);
}

#[test]
fn close_to_exact_at_deepest_zoom() {
    let view = Viewport {
        center_x: "0".parse().unwrap(),
        center_y: "1".parse().unwrap(),
        zoom: GPU_MAX_ZOOM,
    };
    assert_close_to_exact(view, 3000, 0.01);
}

#[test]
fn bands_and_slices_match_a_single_dispatch() {
    let gpu = GpuRenderer::new().unwrap();
    let opts = RenderOptions {
        width: 37,
        height: 23,
        max_iterations: 400,
        shading: Shading::Normal,
    };
    let renderer = Renderer::new(&Viewport::from_f64(-0.7453, 0.1127, 150.0), &opts);
    let whole = Chunking {
        band_pixels: usize::MAX,
        slice_steps: u32::MAX,
    };
    let split = Chunking {
        band_pixels: 37 * 5,
        slice_steps: 7,
    };
    let (whole, split) = (
        samples(&gpu, &renderer, whole),
        samples(&gpu, &renderer, split),
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&whole),
        bytemuck::cast_slice::<_, u8>(&split)
    );
}

#[test]
fn rejects_zoom_past_f32_range() {
    let gpu = GpuRenderer::new().unwrap();
    let view = Viewport::from_f64(0.0, 1.0, GPU_MAX_ZOOM * 10.0);
    let result = gpu.render(&view, &RenderOptions::default());
    assert!(matches!(result, Err(GpuError::ZoomTooDeep(_))));
}

#[test]
fn shading_does_not_change_escapes() {
    let gpu = GpuRenderer::new().unwrap();
    for view in [preset("spirals").unwrap(), deep_seahorse()] {
        let [normal, flat] = [Shading::Normal, Shading::Flat].map(|shading| {
            let opts = RenderOptions {
                width: 64,
                height: 48,
                max_iterations: 4000,
                shading,
            };
            escapes(&gpu, &Renderer::new(&view, &opts), CHUNKING)
        });
        assert_eq!(normal, flat, "zoom {:e}", view.zoom);
    }
}

#[test]
fn render_produces_requested_dimensions() {
    let gpu = GpuRenderer::new().unwrap();
    for (width, height) in [(21, 9), (0, 5)] {
        let opts = RenderOptions {
            width,
            height,
            max_iterations: 50,
            shading: Shading::Normal,
        };
        let img = gpu.render(&Viewport::from_f64(-0.75, 0.0, 1.0), &opts);
        assert_eq!(img.unwrap().dimensions(), (width, height));
    }
}

/// Outside chaotic texture, where f32 reshuffles iteration counts, colors match the CPU's to
/// within rounding.
#[test]
fn colors_match_the_cpu() {
    let gpu = GpuRenderer::new().unwrap();
    for (view, shading) in [
        (preset("mandelbrot").unwrap(), Shading::Normal),
        (preset("mandelbrot").unwrap(), Shading::Flat),
        (Viewport::from_f64(0.2501, 0.0, 1e3), Shading::Normal),
    ] {
        let opts = RenderOptions {
            width: 96,
            height: 64,
            max_iterations: 1500,
            shading,
        };
        let (cpu, gpu) = (render(&view, &opts), gpu.render(&view, &opts).unwrap());
        let differing = cpu
            .pixels()
            .zip(gpu.pixels())
            .filter(|(a, b)| a.0.iter().zip(b.0).any(|(x, y)| x.abs_diff(y) > 2))
            .count();
        let pixels = (opts.width * opts.height) as usize;
        assert!(
            differing * 100 <= pixels,
            "{differing}/{pixels} pixels differ at zoom {:e}, {shading:?}",
            view.zoom
        );
    }
}
