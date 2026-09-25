use clap::Parser;
use image::{ImageFormat, Rgb, RgbImage};
use mandelbrot::{RenderOptions, Shading, Viewport, downsample, preset, render};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Benchmarks rendering across presets, shading modes, resolutions and thread counts
#[derive(Parser, Debug)]
struct Cli {
    /// Directory for results.json, summary.md, rendered images and diffs
    #[arg(long, default_value = "target/bench")]
    out: PathBuf,

    /// Minimum timed samples per scenario, after one warmup run
    #[arg(long, default_value_t = 5)]
    samples: usize,

    /// Keep sampling fast scenarios until this many seconds have been measured
    #[arg(long, default_value_t = 1.0)]
    min_time: f64,

    /// Only run scenarios whose name contains this string
    #[arg(long)]
    filter: Option<String>,

    /// Previous output directory to compare timings and images against
    #[arg(long)]
    baseline: Option<PathBuf>,

    /// Smallest relative change reported as a regression or improvement
    #[arg(long, default_value_t = 0.05)]
    threshold: f64,

    #[arg(long, hide = true)]
    bench: bool,
}

const BENCH_SIZE: (u32, u32) = (1024, 820);
const MAX_SAMPLES: usize = 50;
const NOISE_SIGMAS: f64 = 3.0;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Machine {
    cpu: String,
    logical_cores: usize,
    os: String,
    arch: String,
}

impl Machine {
    fn detect() -> Self {
        let cpu = fs::read_to_string("/proc/cpuinfo")
            .ok()
            .and_then(|info| {
                info.lines()
                    .find(|line| line.starts_with("model name"))
                    .and_then(|line| line.split_once(':'))
                    .map(|(_, model)| model.trim().to_string())
            })
            .unwrap_or_else(|| "unknown CPU".into());
        Self {
            cpu,
            logical_cores: std::thread::available_parallelism().map_or(0, |n| n.get()),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
        }
    }
}

impl std::fmt::Display for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}, {} logical CPUs ({}/{})",
            self.cpu, self.logical_cores, self.os, self.arch
        )
    }
}

struct Sampling {
    min_samples: usize,
    max_samples: usize,
    min_time: Duration,
}

enum Workload {
    Render {
        view: Viewport,
        opts: RenderOptions,
        threads: Option<usize>,
    },
    EncodePng,
    Downsample,
}

struct Scenario {
    name: String,
    workload: Workload,
    max_samples: usize,
}

fn render_scenario(location: &str, shading: Shading) -> Scenario {
    let (width, height) = BENCH_SIZE;
    let mode = match shading {
        Shading::Flat => "flat",
        Shading::Normal => "normal",
    };
    Scenario {
        name: format!("render/{location}/{mode}"),
        workload: Workload::Render {
            view: preset(location).expect("unknown preset"),
            opts: RenderOptions {
                width,
                height,
                shading,
                ..RenderOptions::default()
            },
            threads: None,
        },
        max_samples: usize::MAX,
    }
}

fn scenarios() -> Vec<Scenario> {
    let mut list = vec![
        render_scenario("mandelbrot", Shading::Normal),
        render_scenario("mandelbrot", Shading::Flat),
        render_scenario("mini-mandelbrot", Shading::Normal),
        render_scenario("mini-mandelbrot", Shading::Flat),
        render_scenario("spirals", Shading::Normal),
        render_scenario("quad-spiral", Shading::Normal),
    ];

    let mut single_thread = render_scenario("mandelbrot", Shading::Normal);
    single_thread.name.push_str("/1-thread");
    if let Workload::Render { threads, .. } = &mut single_thread.workload {
        *threads = Some(1);
    }
    list.push(single_thread);

    list.push(Scenario {
        name: "render/mandelbrot/normal/full-res".into(),
        workload: Workload::Render {
            view: preset("mandelbrot").unwrap(),
            opts: RenderOptions::default(),
            threads: None,
        },
        max_samples: 3,
    });
    list.push(Scenario {
        name: "encode/png/full-res".into(),
        workload: Workload::EncodePng,
        max_samples: usize::MAX,
    });
    list.push(Scenario {
        name: "downsample/lanczos3/full-res".into(),
        workload: Workload::Downsample,
        max_samples: usize::MAX,
    });
    list
}

#[derive(Serialize, Deserialize, Clone)]
struct BenchResult {
    name: String,
    unit: String,
    value: f64,
    range: String,
    extra: String,
}

struct Stats {
    median: f64,
    min: f64,
    max: f64,
    stddev: f64,
}

impl Stats {
    fn from(samples: &[Duration]) -> Self {
        let mut ms: Vec<f64> = samples.iter().map(|d| d.as_secs_f64() * 1e3).collect();
        ms.sort_by(f64::total_cmp);
        let n = ms.len();
        let median = if n.is_multiple_of(2) {
            (ms[n / 2 - 1] + ms[n / 2]) / 2.0
        } else {
            ms[n / 2]
        };
        let mean = ms.iter().sum::<f64>() / n as f64;
        let variance = ms.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        Self {
            median,
            min: ms[0],
            max: ms[n - 1],
            stddev: variance.sqrt(),
        }
    }
}

fn time<T>(sampling: &Sampling, mut f: impl FnMut() -> T) -> (Vec<Duration>, T) {
    let mut output = f();
    let mut durations = Vec::new();
    let mut total = Duration::ZERO;
    while durations.len() < sampling.min_samples
        || (total < sampling.min_time && durations.len() < sampling.max_samples)
    {
        let start = Instant::now();
        output = std::hint::black_box(f());
        let elapsed = start.elapsed();
        durations.push(elapsed);
        total += elapsed;
    }
    (durations, output)
}

fn encode_png(img: &RgbImage) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .expect("png encode");
    buf.into_inner()
}

struct Run {
    result: BenchResult,
    image: Option<RgbImage>,
}

fn run(scenario: &Scenario, cli: &Cli, machine: &Machine, full_res: &mut Option<RgbImage>) -> Run {
    let max_samples = scenario.max_samples.min(MAX_SAMPLES);
    let sampling = &Sampling {
        min_samples: cli.samples.clamp(1, max_samples),
        max_samples,
        min_time: Duration::from_secs_f64(cli.min_time),
    };
    let mut source_image = || {
        full_res
            .get_or_insert_with(|| {
                render(&preset("mandelbrot").unwrap(), &RenderOptions::default())
            })
            .clone()
    };

    let (durations, image, detail) = match &scenario.workload {
        Workload::Render {
            view,
            opts,
            threads,
        } => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads.unwrap_or(0))
                .build()
                .expect("thread pool");
            let (durations, img) = pool.install(|| time(sampling, || render(view, opts)));
            if *opts == RenderOptions::default() && preset("mandelbrot").as_ref() == Some(view) {
                full_res.get_or_insert_with(|| img.clone());
            }
            let detail = format!(
                "{}x{}, {} iterations, {} threads",
                opts.width,
                opts.height,
                opts.max_iterations,
                pool.current_num_threads()
            );
            (durations, Some(img), detail)
        }
        Workload::EncodePng => {
            let img = source_image();
            let (durations, bytes) = time(sampling, || encode_png(&img));
            (
                durations,
                None,
                format!(
                    "{}x{}, {} KiB",
                    img.width(),
                    img.height(),
                    bytes.len() / 1024
                ),
            )
        }
        Workload::Downsample => {
            let img = source_image();
            let (durations, small) = time(sampling, || downsample(&img));
            let detail = format!(
                "{}x{} -> {}x{}",
                img.width(),
                img.height(),
                small.width(),
                small.height()
            );
            (durations, None, detail)
        }
    };

    let stats = Stats::from(&durations);
    Run {
        result: BenchResult {
            name: scenario.name.clone(),
            unit: "ms".into(),
            value: round2(stats.median),
            range: format!("± {:.2}", stats.stddev),
            extra: format!(
                "{detail}; min {:.2} ms, max {:.2} ms, {} samples; {}",
                stats.min,
                stats.max,
                durations.len(),
                machine.cpu
            ),
        },
        image,
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn image_file(name: &str) -> String {
    format!("{}.png", name.replace('/', "_"))
}

struct ImageDiff {
    differing_pixels: u64,
    total_pixels: u64,
    max_channel_delta: u8,
    mean_abs_delta: f64,
}

fn diff_images(new: &RgbImage, old: &RgbImage) -> Option<(ImageDiff, RgbImage)> {
    if new.dimensions() != old.dimensions() {
        return None;
    }
    let mut highlight = RgbImage::new(new.width(), new.height());
    let mut differing_pixels = 0;
    let mut max_channel_delta = 0;
    let mut delta_sum = 0u64;

    for ((a, b), out) in new.pixels().zip(old.pixels()).zip(highlight.pixels_mut()) {
        let deltas = [0, 1, 2].map(|i| a[i].abs_diff(b[i]));
        let pixel_max = *deltas.iter().max().unwrap();
        delta_sum += deltas.iter().map(|&d| d as u64).sum::<u64>();
        max_channel_delta = max_channel_delta.max(pixel_max);
        *out = if pixel_max > 0 {
            differing_pixels += 1;
            Rgb([255, 0, 0])
        } else {
            let dimmed_luma = ((a[0] as u32 * 3 + a[1] as u32 * 6 + a[2] as u32) / 30) as u8;
            Rgb([dimmed_luma; 3])
        };
    }

    let total_pixels = new.width() as u64 * new.height() as u64;
    let diff = ImageDiff {
        differing_pixels,
        total_pixels,
        max_channel_delta,
        mean_abs_delta: delta_sum as f64 / (total_pixels * 3) as f64,
    };
    Some((diff, highlight))
}

struct Baseline {
    results: HashMap<String, BenchResult>,
    images: PathBuf,
    machine: Option<Machine>,
}

impl Baseline {
    fn load(dir: &Path) -> Option<Self> {
        let json = fs::read_to_string(dir.join("results.json")).ok()?;
        let results: Vec<BenchResult> = serde_json::from_str(&json).ok()?;
        let machine = fs::read_to_string(dir.join("machine.json"))
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok());
        Some(Self {
            results: results.into_iter().map(|r| (r.name.clone(), r)).collect(),
            images: dir.join("images"),
            machine,
        })
    }

    fn hardware_note(&self, machine: &Machine) -> String {
        match &self.machine {
            Some(m) if m == machine => "Baseline ran on the same hardware.".into(),
            Some(m) => format!(
                "⚠️ Baseline ran on different hardware ({m}); timing deltas are marked ❔ and are not comparable."
            ),
            None => "⚠️ Baseline hardware is unknown; timing deltas are marked ❔ and may not be comparable.".into(),
        }
    }
}

fn relative_noise(result: &BenchResult) -> f64 {
    let stddev: f64 = result
        .range
        .trim_start_matches('±')
        .trim()
        .parse()
        .unwrap_or(0.0);
    stddev / result.value
}

fn timing_cell(
    result: &BenchResult,
    baseline: Option<&Baseline>,
    comparable: bool,
    threshold: f64,
) -> String {
    let Some(old) = baseline.and_then(|b| b.results.get(&result.name)) else {
        return "–".into();
    };
    let change = result.value / old.value - 1.0;
    let noise = NOISE_SIGMAS * (relative_noise(result) + relative_noise(old));
    let significant = change.abs() > threshold.max(noise);
    let marker = match (comparable, significant) {
        (false, _) => "❔",
        (true, false) => "⚪",
        (true, true) if change > 0.0 => "🔴",
        (true, true) => "🟢",
    };
    format!("{marker} {:+.1}% (was {:.2})", change * 100.0, old.value)
}

fn image_cell(
    name: &str,
    image: &RgbImage,
    baseline: Option<&Baseline>,
    diff_dir: &Path,
) -> String {
    let Some(baseline) = baseline else {
        return "–".into();
    };
    let Ok(old) = image::open(baseline.images.join(image_file(name))) else {
        return "no baseline".into();
    };
    let Some((diff, highlight)) = diff_images(image, &old.to_rgb8()) else {
        return "⚠️ size changed".into();
    };
    if diff.differing_pixels == 0 {
        return "✅ identical".into();
    }
    fs::create_dir_all(diff_dir).expect("create diff dir");
    highlight
        .save(diff_dir.join(image_file(name)))
        .expect("save diff");
    format!(
        "⚠️ {:.3}% px differ, max Δ {}, mean Δ {:.3}",
        diff.differing_pixels as f64 / diff.total_pixels as f64 * 100.0,
        diff.max_channel_delta,
        diff.mean_abs_delta
    )
}

fn main() {
    let cli = Cli::parse();
    let images_dir = cli.out.join("images");
    let diff_dir = cli.out.join("diff");
    let _ = fs::remove_dir_all(&diff_dir);
    fs::create_dir_all(&images_dir).expect("create output dir");

    let machine = Machine::detect();
    let baseline = cli.baseline.as_deref().and_then(|dir| {
        let loaded = Baseline::load(dir);
        if loaded.is_none() {
            eprintln!("warning: no usable baseline in {}", dir.display());
        }
        loaded
    });
    let comparable = baseline
        .as_ref()
        .is_some_and(|b| b.machine.as_ref() == Some(&machine));

    let mut full_res = None;
    let mut results = Vec::new();
    let mut table = String::from(
        "| Scenario | Median (ms) | Spread | vs baseline | Image | Details |\n|---|--:|--:|---|---|---|\n",
    );

    for scenario in scenarios() {
        if cli
            .filter
            .as_ref()
            .is_some_and(|f| !scenario.name.contains(f.as_str()))
        {
            continue;
        }
        eprint!("{:<40}", scenario.name);
        let Run { result, image } = run(&scenario, &cli, &machine, &mut full_res);
        eprintln!("{:>10.2} ms {}", result.value, result.range);

        let image_status = match &image {
            Some(img) => {
                img.save(images_dir.join(image_file(&result.name)))
                    .expect("save image");
                image_cell(&result.name, img, baseline.as_ref(), &diff_dir)
            }
            None => "–".into(),
        };
        writeln!(
            table,
            "| `{}` | {:.2} | {} | {} | {} | {} |",
            result.name,
            result.value,
            result.range,
            timing_cell(&result, baseline.as_ref(), comparable, cli.threshold),
            image_status,
            result.extra
        )
        .unwrap();
        results.push(result);
    }

    let mut summary = format!(
        "## Mandelbrot benchmarks\n\nRan on {machine}. Median of at least {} samples per scenario; \
         changes count only beyond {:.0}% and {NOISE_SIGMAS}σ of combined noise.\n\n",
        cli.samples,
        cli.threshold * 100.0
    );
    if let Some(baseline) = &baseline {
        writeln!(summary, "{}\n", baseline.hardware_note(&machine)).unwrap();
    }
    summary.push_str(&table);
    fs::write(cli.out.join("summary.md"), &summary).expect("write summary");
    fs::write(
        cli.out.join("machine.json"),
        serde_json::to_string_pretty(&machine).unwrap(),
    )
    .expect("write machine info");
    fs::write(
        cli.out.join("results.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .expect("write results");
    eprintln!("\nResults written to {}", cli.out.display());
}
