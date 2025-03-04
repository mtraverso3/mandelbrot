# Mandelbrot CLI

A command-line tool written in Rust to generate Mandelbrot set images. Choose from preset locations or specify custom coordinates.

## Example Outputs

|               Mandelbrot               |             Minibrot              |          Spirals          |
|:--------------------------------------:|:---------------------------------:|:-------------------------:|
| ![Mandelbrot](examples/mandelbrot.png) | ![Minibrot](examples/mini-mandelbrot.png) | ![Spirals](examples/spirals.png) |


## Features
- Generate Mandelbrot set visualizations
- Preset locations or custom coordinates
- Adjustable zoom
- Optional image antialiasing
- Timing information

## Installation
1. Ensure Rust is installed ([rustup.rs](https://rustup.rs/))
2. Clone this repository
3. Build with `cargo build --release`
4. (alternatively) Run directly with `cargo run --release -- <ARGS>`
5. The binary will be located at `target/release/mandelbrot`

## Usage
```
Mandelbrot Set Generator CLI

Usage: mandelbrot [OPTIONS] <COMMAND>

Commands:
  preset  Use a predefined location
  custom  Use custom coordinates
  help    Print this message or the help of the given subcommand(s)

Options:
  -o, --output <OUTPUT>  Output file path [default: output.png]
  -r, --resize           Resize output to half size (Anti-aliasing effect)
  -v, --verbose          Show timing information
  -h, --help             Print help
  -V, --version          Print version
```

### Examples
```bash
# Default preset (mandelbrot)
mandelbrot-cli preset

# Preset with options
mandelbrot-cli preset -l spiral -z 2.0 -o spiral.png -r -v

# Custom coordinates, passing in x, y, and zoom
mandelbrot-cli custom -x -0.75 -y 0.0 -z 1.0 -o custom.png
```


## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE.txt) file for details.