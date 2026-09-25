use clap::builder::PossibleValuesParser;
use clap::{Args, Parser, Subcommand};
use image::{ImageFormat, ImageResult, RgbImage};
use mandelbrot::{
    Coordinate, MAX_ZOOM, PRESETS, RenderOptions, Shading, Viewport, auto_iterations, downsample,
    preset, render,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// Mandelbrot Set Generator CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output file path
    #[arg(short, long, global = true, default_value = "output.png")]
    output: PathBuf,

    /// Also save a half-size, anti-aliased copy next to the output
    #[arg(short, long, global = true)]
    resize: bool,

    /// Show timing information
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Image width in pixels
    #[arg(long, global = true, default_value_t = RenderOptions::default().width, value_parser = clap::value_parser!(u32).range(1..))]
    width: u32,

    /// Image height in pixels
    #[arg(long, global = true, default_value_t = RenderOptions::default().height, value_parser = clap::value_parser!(u32).range(1..))]
    height: u32,

    /// Maximum iterations per pixel, or "auto" to pick one from the view
    #[arg(long, global = true, default_value = "1500", value_parser = iterations)]
    iterations: Iterations,

    /// Shading mode
    #[arg(long, global = true, value_enum, default_value_t = RenderOptions::default().shading)]
    shading: Shading,

    /// Render on the GPU (requires the `gpu` feature)
    #[arg(long, global = true)]
    gpu: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Use a predefined location
    Preset(PresetArgs),
    /// Use custom coordinates
    Custom(CustomArgs),
}

#[derive(Args, Debug)]
struct PresetArgs {
    /// Location to render
    #[arg(short, long, default_value = "mandelbrot", value_parser = PossibleValuesParser::new(PRESETS.map(|(name, ..)| name)))]
    location: String,

    /// Zoom multiplier applied to the preset
    #[arg(short, long, default_value_t = 1.0, value_parser = positive)]
    zoom: f64,
}

#[derive(Args, Debug)]
struct CustomArgs {
    /// Real coordinate of the image center, to any number of decimal places
    #[arg(short, allow_negative_numbers = true)]
    x: Coordinate,

    /// Imaginary coordinate of the image center, to any number of decimal places
    #[arg(short, allow_negative_numbers = true)]
    y: Coordinate,

    /// Zoom factor, where 1 shows the whole set (up to 1e250)
    #[arg(short, value_parser = positive)]
    zoom: f64,
}

#[derive(Clone, Copy, Debug)]
enum Iterations {
    Auto,
    Fixed(usize),
}

fn iterations(value: &str) -> Result<Iterations, String> {
    match value {
        "auto" => Ok(Iterations::Auto),
        _ => match value.parse::<usize>() {
            Ok(0) => Err("must be at least 1".into()),
            Ok(n) => Ok(Iterations::Fixed(n)),
            Err(_) => Err("expected a number or \"auto\"".into()),
        },
    }
}

fn positive(value: &str) -> Result<f64, String> {
    match value.parse::<f64>() {
        Ok(v) if v > 0.0 && v.is_finite() => Ok(v),
        Ok(_) => Err("must be a positive number".into()),
        Err(e) => Err(e.to_string()),
    }
}

fn resized_path(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default().to_string_lossy();
    let name = match output.extension() {
        Some(ext) => format!("{stem}_resized.{}", ext.to_string_lossy()),
        None => format!("{stem}_resized"),
    };
    output.with_file_name(name)
}

fn save(img: &RgbImage, path: &Path) -> ImageResult<()> {
    let format = ImageFormat::from_path(path).unwrap_or(ImageFormat::Png);
    img.save_with_format(path, format)
}

fn save_outputs(img: &RgbImage, output: &Path, resize: bool) -> ImageResult<()> {
    save(img, output)?;
    if resize {
        save(&downsample(img), &resized_path(output))?;
    }
    Ok(())
}

/// Where the image is computed, so the rest of the CLI need not know about the `gpu` feature.
enum Backend {
    Cpu,
    #[cfg(feature = "gpu")]
    Gpu(Box<mandelbrot::GpuRenderer>),
}

impl Backend {
    fn new(gpu: bool, verbose: bool) -> Result<Self, String> {
        if !gpu {
            return Ok(Self::Cpu);
        }
        #[cfg(feature = "gpu")]
        {
            let gpu = mandelbrot::GpuRenderer::new().map_err(|e| e.to_string())?;
            if verbose {
                println!("Rendering on {}", gpu.adapter_name());
            }
            Ok(Self::Gpu(Box::new(gpu)))
        }
        #[cfg(not(feature = "gpu"))]
        {
            let _ = verbose;
            Err("this build has no GPU support; rebuild with `--features gpu`".into())
        }
    }

    fn auto_iterations(&self, view: &Viewport, width: u32, height: u32) -> Result<usize, String> {
        match self {
            Self::Cpu => Ok(auto_iterations(view, width, height)),
            #[cfg(feature = "gpu")]
            Self::Gpu(gpu) => gpu
                .auto_iterations(view, width, height)
                .map_err(|e| e.to_string()),
        }
    }

    fn render(&self, view: &Viewport, opts: &RenderOptions) -> Result<RgbImage, String> {
        match self {
            Self::Cpu => Ok(render(view, opts)),
            #[cfg(feature = "gpu")]
            Self::Gpu(gpu) => gpu.render(view, opts).map_err(|e| e.to_string()),
        }
    }
}

fn run(cli: &Cli, view: &Viewport, start: Instant) -> Result<(), String> {
    let backend = Backend::new(cli.gpu, cli.verbose)?;
    let opts = RenderOptions {
        width: cli.width,
        height: cli.height,
        max_iterations: match cli.iterations {
            Iterations::Fixed(n) => n,
            Iterations::Auto => backend.auto_iterations(view, cli.width, cli.height)?,
        },
        shading: cli.shading,
    };
    let img = backend.render(view, &opts)?;
    if cli.verbose {
        println!(
            "Rendered {}x{} with {} iterations in {:.3?}",
            img.width(),
            img.height(),
            opts.max_iterations,
            start.elapsed()
        );
    }

    let save_start = Instant::now();
    save_outputs(&img, &cli.output, cli.resize)
        .map_err(|e| format!("failed to save image: {e}"))?;
    if cli.verbose {
        println!("Saved in {:.3?}", save_start.elapsed());
        println!("Total: {:.3?}", start.elapsed());
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let start = Instant::now();

    let view = match &cli.command {
        Commands::Preset(args) => {
            let base = preset(&args.location).expect("validated by clap");
            Viewport {
                zoom: base.zoom * args.zoom,
                ..base
            }
        }
        Commands::Custom(args) => Viewport {
            center_x: args.x.clone(),
            center_y: args.y.clone(),
            zoom: args.zoom,
        },
    };
    if view.zoom > MAX_ZOOM {
        eprintln!(
            "Error: zoom {:e} is past the deepest supported zoom of {MAX_ZOOM:e}",
            view.zoom
        );
        return ExitCode::FAILURE;
    }
    match run(&cli, &view, start) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
