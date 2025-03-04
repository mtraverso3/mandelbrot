use mandelbrot::generate_mandelbrot_image;
use clap::{Parser, Args, Subcommand};
use std::path::PathBuf;

mod mandelbrot;

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
    /// Mandelbrot location to generate (mandelbrot, mini-mandelbrot, spiral, quad-spiral)
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

fn mandelbrot_locations(name: &str) -> Result<(f64, f64, f64), &'static str> {
    Ok(match name {
        "mini-mandelbrot" => (-1.249559196, 0.030466443, 1.73e6),
        "spiral" => (-1.2494989, 0.0303330, 4.437000e4),
        "quad-spiral" => (-4.621603e-1, -5.823998e-1, 2.633507e7),
        "mandelbrot" => (-0.75, 0.0, 1.0),
        _ => return Err("Invalid preset location"),
    })
}

fn main() {
    let cli = Cli::parse();
    let start = std::time::Instant::now();

    // Get coordinates and zoom based on command
    let (x, y, zoom) = match cli.command {
        Commands::Preset(args) => {
            let (base_x, base_y, base_zoom) = match mandelbrot_locations(&args.location) {
                Ok((x, y, z)) => (x, y, z),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };
            
            (base_x, base_y, base_zoom * args.zoom)
        }
        Commands::Custom(args) => (args.x, args.y, args.zoom),
    };

    // Generate the image
    let img = generate_mandelbrot_image(x, y, zoom, true);

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
        let width = img.width();
        let height = img.height();

        let resized = image::DynamicImage::ImageRgb8(img)
            .resize(
                width / 2,
                height / 2,
                image::imageops::FilterType::Lanczos3
            )
            .to_rgb8();

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