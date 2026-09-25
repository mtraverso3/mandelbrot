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
- Configurable resolution, iteration count and shading
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
  -o, --output <OUTPUT>          Output file path [default: output.png]
  -r, --resize                   Also save a half-size, anti-aliased copy next to the output
  -v, --verbose                  Show timing information
      --width <WIDTH>            Image width in pixels [default: 4096]
      --height <HEIGHT>          Image height in pixels [default: 3280]
      --iterations <ITERATIONS>  Maximum iterations per pixel [default: 1500]
      --shading <SHADING>        Shading mode [default: normal] [possible values: flat, normal]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples
```bash
# Default preset (mandelbrot)
mandelbrot preset

# Preset with options
mandelbrot preset -l spirals -z 2.0 -o spirals.png -r -v

# Custom coordinates, passing in x, y, and zoom
mandelbrot custom -x -0.75 -y 0.0 -z 1.0 -o custom.png

# Quick, low-resolution preview with flat shading
mandelbrot preset -l quad-spiral --width 800 --height 640 --shading flat
```


## Benchmarks
```bash
# Run all scenarios, writing results, summary and images to target/bench
cargo bench --bench render

# Fewer samples, subset of scenarios, compared against a previous run
cargo bench --bench render -- --samples 2 --filter spirals --baseline path/to/previous
```

Each run records the machine it ran on. Comparisons against a baseline from different hardware
are marked ❔, and changes only count when they exceed both `--threshold` and the measured noise.

The `Benchmark` workflow builds the bench binary for both the change and its base commit (the PR
base, or the previous commit on `master`) and runs them back to back on the same runner. The job
summary shows timing deltas and image diffs, and the `bench-results` artifact holds the rendered
images. Pushes to `master` publish timing history to the `gh-pages` branch
(charts at `https://<owner>.github.io/mandelbrot/dev/bench/` once GitHub Pages is enabled).


## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE.txt) file for details.