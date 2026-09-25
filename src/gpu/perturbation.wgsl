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
}

struct Step {
    a: vec2<f32>,
    b: vec2<f32>,
    radius_sqr: f32,
}

// Only the derivative's direction is used, so it is kept as mantissa * 2^exponent to stay
// within f32 range
struct State {
    delta: vec2<f32>,
    derivative: vec2<f32>,
    checkpoint_reference: vec2<f32>,
    checkpoint_delta: vec2<f32>,
    exponent: i32,
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

override SKIP: bool;
override TRACK_DERIVATIVE: bool;

const ESCAPE_RADIUS_SQR: f32 = 10000.0;
const INTERIOR: u32 = 0xffffffffu;
const DERIVATIVE_LIMIT: f32 = 65536.0;
const CYCLE_CHECK_START: u32 = 16u;
// Larger than any orbit point, so no state matches before the first checkpoint
const NO_CHECKPOINT: f32 = 1e30;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> orbit: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> bla: array<Step>;
// (offset into bla, length) of each level; level k advances 2^k iterations
@group(0) @binding(3) var<storage, read> bla_levels: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read_write> states: array<State>;
@group(0) @binding(5) var<storage, read_write> samples: array<Sample>;
@group(0) @binding(6) var<storage, read_write> unfinished: atomic<u32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.width || id.y >= params.rows {
        return;
    }
    let index = id.y * params.width + id.x;
    var state: State;
    if params.first_slice != 0u {
        state = State(
            vec2(0.0),
            vec2(1.0, 0.0),
            vec2(NO_CHECKPOINT),
            vec2(0.0),
            0,
            0u,
            0u,
            CYCLE_CHECK_START,
            0u,
        );
    } else {
        state = states[index];
        if state.done != 0u {
            return;
        }
    }
    let pixel = vec2(f32(id.x) - params.left, f32(params.first_row + id.y) - params.top);
    let dc = pixel * params.pixel_size;

    for (var step = 0u; step < params.slice_steps && state.n < params.max_iterations; step++) {
        var level = -1;
        let delta_sqr = dot(state.delta, state.delta);
        if SKIP && delta_sqr < params.max_skip_radius_sqr {
            level = skip_level(state.m, delta_sqr, params.max_iterations - state.n);
        }
        if level >= 0 {
            let range = bla_levels[level];
            let skip = bla[range.x + ((state.m - 1u) >> u32(level))];
            state.delta = mul(skip.a, state.delta) + mul(skip.b, dc);
            if TRACK_DERIVATIVE {
                // Normalized first so the product with a large A stays finite
                state = normalize_derivative(state);
                state.derivative = mul(skip.a, state.derivative) + skip.b * exp2_neg(state.exponent);
            }
            state.m += 1u << u32(level);
            state.n += 1u << u32(level);
            continue;
        }

        let reference = orbit[state.m];
        let z = reference + state.delta;
        let norm_sqr = dot(z, z);
        if norm_sqr > ESCAPE_RADIUS_SQR {
            samples[index] = Sample(state.n, norm_sqr, normal(z, state.derivative));
            state.done = 1u;
            break;
        }
        // An exactly repeated state means the iteration is periodic and can never escape
        if all(reference == state.checkpoint_reference) && all(state.delta == state.checkpoint_delta) {
            state.n = params.max_iterations;
            break;
        }
        if state.n >= state.next_checkpoint {
            state.checkpoint_reference = reference;
            state.checkpoint_delta = state.delta;
            state.next_checkpoint *= 2u;
        }
        if TRACK_DERIVATIVE {
            state.derivative = 2.0 * mul(state.derivative, z) + vec2(exp2_neg(state.exponent), 0.0);
            if max(abs(state.derivative.x), abs(state.derivative.y)) > DERIVATIVE_LIMIT {
                state = normalize_derivative(state);
            }
        }
        if state.m == params.last || norm_sqr < dot(state.delta, state.delta) {
            state.delta = z;
            state.m = 0u;
        }
        state.delta = 2.0 * mul(orbit[state.m], state.delta) + mul(state.delta, state.delta) + dc;
        state.m++;
        state.n++;
    }

    if state.done == 0u {
        if state.n >= params.max_iterations {
            samples[index] = Sample(INTERIOR, 0.0, vec2(0.0));
            state.done = 1u;
        } else {
            atomicAdd(&unfinished, 1u);
        }
    }
    states[index] = state;
}

/// The highest level of the table that can skip from reference index `m`, or -1.
fn skip_level(m: u32, delta_sqr: f32, budget: u32) -> i32 {
    if m == 0u {
        return -1;
    }
    let index = m - 1u;
    let aligned = min(countTrailingZeros(index), params.bla_levels - 1u);
    var found = -1;
    // Level 0 saves nothing over a plain step, and is implied by level 1 since a merged block
    // is never valid further out than its first half
    for (var level = 1u; level <= aligned; level++) {
        let range = bla_levels[level];
        let j = index >> level;
        if j >= range.y || (1u << level) > budget || delta_sqr >= bla[range.x + j].radius_sqr {
            break;
        }
        found = i32(level);
    }
    return found;
}

fn normalize_derivative(state: State) -> State {
    var normalized = state;
    let largest = frexp(max(abs(state.derivative.x), abs(state.derivative.y)));
    normalized.derivative = ldexp(state.derivative, vec2(-largest.exp));
    normalized.exponent += largest.exp;
    return normalized;
}

fn mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

fn exp2_neg(exponent: i32) -> f32 {
    if exponent > 126 {
        return 0.0;
    }
    return ldexp(1.0, min(-exponent, 127));
}

fn normal(z: vec2<f32>, derivative: vec2<f32>) -> vec2<f32> {
    let u = mul(z, vec2(derivative.x, -derivative.y));
    return u / length(u);
}
