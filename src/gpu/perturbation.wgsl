struct Params {
    width: u32,
    first_row: u32,
    rows: u32,
    max_iterations: u32,
    last: u32,
    track_derivative: u32,
    pixel_size: f32,
    left: f32,
    top: f32,
}

struct Sample {
    iterations: u32,
    norm_sqr: f32,
    normal: vec2<f32>,
}

const ESCAPE_RADIUS_SQR: f32 = 10000.0;
const INTERIOR: u32 = 0xffffffffu;
const DERIVATIVE_LIMIT: f32 = 4294967296.0;
const DERIVATIVE_LIMIT_BITS: i32 = 32;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> orbit: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> samples: array<Sample>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.width || id.y >= params.rows {
        return;
    }
    let pixel = vec2(f32(id.x) - params.left, f32(params.first_row + id.y) - params.top);
    samples[id.y * params.width + id.x] = escape(pixel * params.pixel_size);
}

fn escape(dc: vec2<f32>) -> Sample {
    var delta = vec2(0.0);
    // Only the derivative's direction is used, so it is kept as mantissa * 2^exponent to stay
    // within f32 range
    var derivative = vec2(1.0, 0.0);
    var exponent = 0;
    var m = 0u;
    for (var n = 0u; n < params.max_iterations; n++) {
        let z = orbit[m] + delta;
        let norm_sqr = dot(z, z);
        if norm_sqr > ESCAPE_RADIUS_SQR {
            return Sample(n, norm_sqr, normal(z, derivative));
        }
        if params.track_derivative != 0u {
            derivative = 2.0 * mul(derivative, z) + vec2(exp2_neg(exponent), 0.0);
            if max(abs(derivative.x), abs(derivative.y)) > DERIVATIVE_LIMIT {
                derivative /= DERIVATIVE_LIMIT;
                exponent += DERIVATIVE_LIMIT_BITS;
            }
        }
        if m == params.last || norm_sqr < dot(delta, delta) {
            delta = z;
            m = 0u;
        }
        delta = 2.0 * mul(orbit[m], delta) + mul(delta, delta) + dc;
        m++;
    }
    return Sample(INTERIOR, 0.0, vec2(0.0));
}

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn exp2_neg(exponent: i32) -> f32 {
    if exponent > 126 {
        return 0.0;
    }
    return ldexp(1.0, -exponent);
}

fn normal(z: vec2<f32>, derivative: vec2<f32>) -> vec2<f32> {
    let u = mul(z, vec2(derivative.x, -derivative.y));
    return u / length(u);
}
