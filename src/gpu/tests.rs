use super::*;
use crate::{RenderOptions, Viewport, preset, render};

fn samples(
    gpu: &GpuRenderer,
    renderer: &Renderer,
    variant: Variant,
    chunking: Chunking,
) -> Vec<Sample> {
    let mut samples = Vec::new();
    let finished = pollster::block_on(gpu.run(
        renderer,
        variant,
        chunking,
        Output::Samples,
        &|| false,
        &mut |_, bytes| samples.extend(bytemuck::pod_collect_to_vec::<u8, Sample>(bytes)),
    ));
    assert!(finished.unwrap());
    samples
}

fn escapes(gpu: &GpuRenderer, renderer: &Renderer, variant: Variant) -> Vec<Option<usize>> {
    samples(gpu, renderer, variant, CHUNKING)
        .iter()
        .map(|s| (s.iterations < UNDECIDED).then_some(s.iterations as usize))
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
    assert_variant_close_to_exact(view, max_iterations, max_mismatch, false);
}

/// The same with offsets starting out with a separate exponent, whatever the zoom.
fn assert_deep_close_to_exact(view: Viewport, max_iterations: usize, max_mismatch: f64) {
    assert_variant_close_to_exact(view, max_iterations, max_mismatch, true);
}

fn assert_variant_close_to_exact(
    view: Viewport,
    max_iterations: usize,
    max_mismatch: f64,
    force_deep: bool,
) {
    let gpu = GpuRenderer::new().unwrap();
    let opts = RenderOptions {
        width: 96,
        height: 64,
        max_iterations,
        shading: Shading::Normal,
    };
    let renderer = Renderer::new(&view, &opts);
    let mut variant = Variant::new(&renderer);
    variant.deep |= force_deep;
    let actual = escapes(&gpu, &renderer, variant);
    let expected = exact_escapes(&renderer);
    assert!(
        expected.iter().any(|e| *e != expected[0]),
        "grid should not be uniform"
    );
    let pairs = || actual.iter().zip(&expected);
    let flipped = pairs().filter(|(a, e)| a.is_some() != e.is_some()).count();
    let mismatches = pairs().filter(|(a, e)| a != e).count();
    eprintln!(
        "zoom {:e}{}: {mismatches}/{} differ, {flipped} flipped",
        view.zoom,
        if variant.deep { " (deep)" } else { "" },
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
fn close_to_exact_at_the_edge_of_f32_range() {
    assert_close_to_exact(misiurewicz_i(DEEP_ZOOM), 3000, 0.01);
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
    let variant = Variant::new(&renderer);
    let (whole, split) = (
        samples(&gpu, &renderer, variant, whole),
        samples(&gpu, &renderer, variant, split),
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&whole),
        bytemuck::cast_slice::<_, u8>(&split)
    );
}

fn misiurewicz_i(zoom: f64) -> Viewport {
    Viewport {
        center_x: "0".parse().unwrap(),
        center_y: "1".parse().unwrap(),
        zoom,
    }
}

// The real nucleus of the period 3 minibrot, where the reference orbit falls to about 1e-80
// every third iteration
const PERIOD_3_NUCLEUS: &str =
    "-1.7548776662466927600495088963585286918946066177727931439892839706460806551280810";

#[test]
fn close_to_exact_past_f32_range() {
    for zoom in [1e40, 1e100, 1e200, crate::MAX_ZOOM] {
        assert_close_to_exact(misiurewicz_i(zoom), 3000, 0.01);
    }
}

#[test]
fn deep_path_close_to_exact_at_shallow_zoom() {
    assert_deep_close_to_exact(preset("mandelbrot").unwrap(), 1500, 0.01);
    assert_deep_close_to_exact(preset("spirals").unwrap(), 6000, 0.25);
    assert_deep_close_to_exact(Viewport::from_f64(0.2501, 0.0, 1e3), 2000, 0.01);
    assert_deep_close_to_exact(misiurewicz_i(1e20), 3000, 0.01);
    let nucleus = Viewport {
        center_x: PERIOD_3_NUCLEUS.parse().unwrap(),
        center_y: "0".parse().unwrap(),
        zoom: 40.0,
    };
    assert_deep_close_to_exact(nucleus, 3000, 0.01);
}

#[test]
fn deep_view_inside_a_minibrot_stays_interior() {
    let gpu = GpuRenderer::new().unwrap();
    let view = Viewport {
        center_x: PERIOD_3_NUCLEUS.parse().unwrap(),
        center_y: format!("0.{}1", "0".repeat(44)).parse().unwrap(),
        zoom: 1e40,
    };
    let opts = RenderOptions {
        width: 32,
        height: 24,
        max_iterations: 3000,
        shading: Shading::Normal,
    };
    let renderer = Renderer::new(&view, &opts);
    assert!(exact_escapes(&renderer).iter().all(Option::is_none));
    let escaped = escapes(&gpu, &renderer, Variant::new(&renderer));
    assert!(escaped.iter().all(Option::is_none), "{escaped:?}");
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
            let renderer = Renderer::new(&view, &opts);
            escapes(&gpu, &renderer, Variant::new(&renderer))
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

#[test]
fn auto_iterations_match_the_cpu() {
    let gpu = GpuRenderer::new().unwrap();
    let nucleus = |zoom| Viewport {
        center_x: PERIOD_3_NUCLEUS.parse().unwrap(),
        center_y: "0".parse().unwrap(),
        zoom,
    };
    for view in [
        preset("mandelbrot").unwrap(),
        preset("spirals").unwrap(),
        nucleus(40.0),
        nucleus(1e12),
        misiurewicz_i(1e40),
        deep_seahorse(),
    ] {
        for (width, height) in [(400, 300), (1920, 1080)] {
            assert_eq!(
                gpu.auto_iterations(&view, width, height).unwrap(),
                crate::auto_iterations(&view, width, height),
                "zoom {:e} at {width}x{height}",
                view.zoom
            );
        }
    }
}
