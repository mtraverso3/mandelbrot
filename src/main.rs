use clap::builder::PossibleValuesParser;
use clap::{Args, Parser, Subcommand};
use image::{ImageFormat, ImageResult, RgbImage};
use mandelbrot::{PRESETS, RenderOptions, Shading, Viewport, downsample, preset, render};
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

    /// Maximum iterations per pixel
    #[arg(long, global = true, default_value_t = RenderOptions::default().max_iterations)]
    iterations: usize,

    /// Shading mode
    #[arg(long, global = true, value_enum, default_value_t = RenderOptions::default().shading)]
    shading: Shading,
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
    #[arg(short, long, default_value = "mandelbrot", value_parser = PossibleValuesParser::new(PRESETS.map(|(name, _)| name)))]
    location: String,

    /// Zoom multiplier applied to the preset
    #[arg(short, long, default_value_t = 1.0, value_parser = positive)]
    zoom: f64,
}

#[derive(Args, Debug)]
struct CustomArgs {
    /// Real coordinate of the image center
    #[arg(short, allow_negative_numbers = true)]
    x: f64,

    /// Imaginary coordinate of the image center
    #[arg(short, allow_negative_numbers = true)]
    y: f64,

    /// Zoom factor, where 1 shows the whole set
    #[arg(short, value_parser = positive)]
    zoom: f64,
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let start = Instant::now();

    let view = match cli.command {
        Commands::Preset(args) => {
            let base = preset(&args.location).expect("validated by clap");
            Viewport {
                zoom: base.zoom * args.zoom,
                ..base
            }
        }
        Commands::Custom(args) => Viewport {
            center_x: args.x,
            center_y: args.y,
            zoom: args.zoom,
        },
    };
    let opts = RenderOptions {
        width: cli.width,
        height: cli.height,
        max_iterations: cli.iterations,
        shading: cli.shading,
    };

    let img = render(&view, &opts);
    if cli.verbose {
        println!(
            "Rendered {}x{} in {:.3?}",
            img.width(),
            img.height(),
            start.elapsed()
        );
    }

    let save_start = Instant::now();
    if let Err(e) = save_outputs(&img, &cli.output, cli.resize) {
        eprintln!("Error: failed to save image: {e}");
        return ExitCode::FAILURE;
    }

    if cli.verbose {
        println!("Saved in {:.3?}", save_start.elapsed());
        println!("Total: {:.3?}", start.elapsed());
    }
    ExitCode::SUCCESS
}
