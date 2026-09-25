use clap::{Args, Parser, Subcommand};
use mandelbrot::{RenderOptions, Viewport, downsample, preset, render};
use std::path::PathBuf;

/// Mandelbrot Set Generator CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output file path
    #[arg(short, long, default_value = "output.png")]
    output: PathBuf,

    /// Resize output to half size (Anti-aliasing effect)
    #[arg(short, long, default_value_t = false)]
    resize: bool,

    /// Show timing information
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
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
    /// Mandelbrot location to generate (mandelbrot, mini-mandelbrot, spirals, quad-spiral)
    #[arg(short, long, default_value = "mandelbrot")]
    location: String,

    /// Zoom factor multiplier
    #[arg(short, long, default_value_t = 1.0)]
    zoom: f64,
}

#[derive(Args, Debug)]
struct CustomArgs {
    /// X coordinate
    #[arg(short)]
    x: f64,

    /// Y coordinate
    #[arg(short)]
    y: f64,

    /// Zoom factor
    #[arg(short)]
    zoom: f64,
}

fn main() {
    let cli = Cli::parse();
    let start = std::time::Instant::now();

    let view = match cli.command {
        Commands::Preset(args) => {
            let Some(base) = preset(&args.location) else {
                eprintln!("Error: Invalid preset location");
                std::process::exit(1);
            };
            Viewport { zoom: base.zoom * args.zoom, ..base }
        }
        Commands::Custom(args) => Viewport { center_x: args.x, center_y: args.y, zoom: args.zoom },
    };

    let img = render(&view, &RenderOptions::default());

    if cli.verbose {
        let duration_generation = start.elapsed();
        println!(
            "Mandelbrot image generated in: {:.3?}, saving...",
            duration_generation
        );
    }

    let start2 = std::time::Instant::now();

    // Save original image
    img.save(&cli.output).expect("Failed to save output image");

    // Resize/anti-aliasing
    if cli.resize {
        let resized = downsample(&img);
        let mut resized_path = cli.output.clone();
        let stem = cli.output.file_stem().unwrap().to_str().unwrap();
        let ext = cli.output.extension().unwrap().to_str().unwrap();
        resized_path.set_file_name(format!("{}_resized.{}", stem, ext));

        resized.save(&resized_path).expect("Failed to save resized image");
    }
    
    if cli.verbose {
        let duration_img = start2.elapsed();
        println!("Mandelbrot image saved in: {:.3?}", duration_img);
        let duration = start.elapsed();
        println!("Time elapsed overall is: {:.3?}", duration);
    }
}