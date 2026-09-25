// Declarations shared by the iterate and color passes, which are compiled as one module

struct Params {
    width: u32,
    first_row: u32,
    rows: u32,
    max_iterations: u32,
    last: u32,
    pixel_size: f32,
    left: f32,
    top: f32,
    slice_steps: u32,
    first_slice: u32,
    bla_levels: u32,
    max_skip_radius_sqr: f32,
    check_bulbs: u32,
    normal_shading: u32,
    light: vec2<f32>,
    inv_band_scale: f32,
    band_cycle: f32,
    band_phase: f32,
    pixel_mantissa: f32,
    pixel_exponent: i32,
    // The main cardioid's and period-2 bulb's equations expanded around the view's center,
    // so each pixel only adds small terms in its offset: see `in_main_cardioid_or_bulb`
    cardioid: vec4<f32>,
    bulb: vec4<f32>,
}

// Past f32 range, values are also kept as mantissa * 2^exponent
struct OrbitPoint {
    z: vec2<f32>,
    mantissa: vec2<f32>,
    exponent: i32,
}

struct Step {
    a: vec2<f32>,
    b: vec2<f32>,
    a_mantissa: vec2<f32>,
    b_mantissa: vec2<f32>,
    a_exponent: i32,
    b_exponent: i32,
    radius_sqr: f32,
    radius_mantissa: f32,
    radius_exponent: i32,
}

// Only the derivative's direction is used, so it is kept as mantissa * 2^exponent to stay
// within f32 range. So are delta, while delta_exponent is negative, and the contraction.
struct State {
    delta: vec2<f32>,
    derivative: vec2<f32>,
    checkpoint_reference: vec2<f32>,
    checkpoint_delta: vec2<f32>,
    contraction: vec2<f32>,
    exponent: i32,
    delta_exponent: i32,
    contraction_exponent: i32,
    m: u32,
    n: u32,
    next_checkpoint: u32,
    done: u32,
}

struct Sample {
    iterations: u32,
    norm_sqr: f32,
    normal: vec2<f32>,
}

const INTERIOR: u32 = 0xffffffffu;
// Neither escaped nor known to be interior within the iteration limit
const UNDECIDED: u32 = 0xfffffffeu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> orbit: array<OrbitPoint>;
@group(0) @binding(2) var<storage, read> bla: array<Step>;
// (offset into bla, length) of each level; level k advances 2^k iterations
@group(0) @binding(3) var<storage, read> bla_levels: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read_write> states: array<State>;
@group(0) @binding(5) var<storage, read_write> samples: array<Sample>;
@group(0) @binding(6) var<storage, read_write> unfinished: atomic<u32>;
// RGBA8
@group(0) @binding(7) var<storage, read_write> pixels: array<u32>;

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}
