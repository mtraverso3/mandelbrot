use mandelbrot1::generate_mandelbrot_image;

use image_ascii::TextGenerator;

mod mandelbrot1;

fn mandelbrot_locations(name: &str) -> (f64, f64, f64) {
    match name {
        "mini-mandelbrot" => (-1.249559196, 0.030466443, 1.73e6),
        "spiral" => (-1.2494989, 0.0303330, 4.437000e4),
        "quad-spiral" => (-4.621603e-1, -5.823998e-1, 2.633507e7),
        _ => (-0.75, 0.0, 1.0),
    }
}

fn main() {
    let start = std::time::Instant::now();

    let (x, y, zoom) = mandelbrot_locations("mini-mandelbrot");
    let img = generate_mandelbrot_image(x, y, zoom);

    let duration_generation = start.elapsed();
    println!(
        "Mandelbrot image generated in: {:.3?}, saving...",
        duration_generation
    );
    let start2 = std::time::Instant::now();

    img.save("output.png").unwrap();

    let width = img.width();
    let height = img.height();
    image::DynamicImage::ImageRgb8(img)
        .resize(width / 2, height / 2, image::imageops::FilterType::Lanczos3)
        .to_rgb8()
        .save("output_resized.png")
        .unwrap();


    let duration_img = start2.elapsed();
    println!("Mandelbrot image saved in: {:.3?}", duration_img);

    let duration = start.elapsed();
    println!("Time elapsed overall is: {:.3?}", duration);
}
