override SKIP: bool = false;
override TRACK_DERIVATIVE: bool = false;
override DEEP: bool = false;
override DETECT_INTERIOR: bool = false;

const ESCAPE_RADIUS_SQR: f32 = 10000.0;
const DERIVATIVE_LIMIT: f32 = 65536.0;
const CYCLE_CHECK_START: u32 = 16u;
// Larger than any orbit point, so no state matches before the first checkpoint
const NO_CHECKPOINT: f32 = 1e30;
// As on the CPU: the product of 2z along an orbit only shrinks this far inside the set, and
// is capped around 1e100 so a contraction late in the orbit still shows
const INTERIOR_CONTRACTION: f32 = 1e-6;
const CONTRACTION_CAP_EXPONENT: i32 = 332;

@compute @workgroup_size(8, 8)
fn iterate(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.width || id.y >= params.rows {
        return;
    }
    let index = id.y * params.width + id.x;
    let pixel = vec2(f32(id.x) - params.left, f32(params.first_row + id.y) - params.top);
    let dc = pixel * params.pixel_size;
    var state: State;
    if params.first_slice != 0u {
        state = State(
            vec2(0.0),
            vec2(1.0, 0.0),
            vec2(NO_CHECKPOINT),
            vec2(0.0),
            vec2(1.0, 0.0),
            0,
            select(0, params.pixel_exponent, DEEP),
            0,
            0u,
            0u,
            CYCLE_CHECK_START,
            0u,
        );
        if params.check_bulbs != 0u && in_main_cardioid_or_bulb(params.center + dc) {
            samples[index] = Sample(INTERIOR, 0.0, vec2(0.0));
            state.done = 1u;
            states[index] = state;
            return;
        }
    } else {
        state = states[index];
        if state.done != 0u {
            return;
        }
    }
    for (var step = 0u; step < params.slice_steps && state.n < params.max_iterations; step++) {
        if DEEP && state.delta_exponent < 0 {
            state = deep_step(state, index, pixel);
            if state.done != 0u {
                break;
            }
            continue;
        }
        var level = -1;
        let delta_sqr = dot(state.delta, state.delta);
        if SKIP && delta_sqr < params.max_skip_radius_sqr {
            level = skip_level(state.m, delta_sqr, params.max_iterations - state.n);
        }
        if level >= 0 {
            let range = bla_levels[level];
            let skip = bla[range.x + ((state.m - 1u) >> u32(level))];
            state.delta = mul(skip.a, state.delta) + mul(skip.b, dc);
            if DETECT_INTERIOR && contract(&state, skip.a, 0) {
                state = finish_interior(state, index);
                break;
            }
            if TRACK_DERIVATIVE {
                // Normalized first so the product with a large A stays finite
                state = normalize_derivative(state);
                state.derivative = mul(skip.a, state.derivative) + skip.b * exp2_neg(state.exponent);
            }
            state.m += 1u << u32(level);
            state.n += 1u << u32(level);
            continue;
        }

        let reference = orbit[state.m].z;
        let z = reference + state.delta;
        let norm_sqr = dot(z, z);
        if norm_sqr > ESCAPE_RADIUS_SQR {
            samples[index] = Sample(state.n, norm_sqr, normal(z, state.derivative));
            state.done = 1u;
            break;
        }
        if DETECT_INTERIOR && state.n > 0u && contract(&state, 2.0 * z, 0) {
            state = finish_interior(state, index);
            break;
        }
        // An exactly repeated state means the iteration is periodic and can never escape
        if all(reference == state.checkpoint_reference) && all(state.delta == state.checkpoint_delta) {
            state = finish_interior(state, index);
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
        state.delta = 2.0 * mul(orbit[state.m].z, state.delta) + mul(state.delta, state.delta) + dc;
        state.m++;
        state.n++;
    }

    if state.done == 0u {
        if state.n >= params.max_iterations {
            samples[index] = Sample(UNDECIDED, 0.0, vec2(0.0));
            state.done = 1u;
        } else {
            atomicAdd(&unfinished, 1u);
        }
    }
    states[index] = state;
}

fn finish_interior(state_in: State, index: u32) -> State {
    var state = state_in;
    samples[index] = Sample(INTERIOR, 0.0, vec2(0.0));
    state.done = 1u;
    return state;
}

/// Multiplies a factor into the contraction, returning whether the orbit is now known to be
/// interior.
fn contract(state: ptr<function, State>, factor: vec2<f32>, factor_exponent: i32) -> bool {
    let product = fx_normalize(Fx(
        mul((*state).contraction, factor),
        (*state).contraction_exponent + factor_exponent,
    ));
    (*state).contraction = product.mantissa;
    (*state).contraction_exponent = min(product.exponent, CONTRACTION_CAP_EXPONENT);
    let size = length(product.mantissa);
    if size == 0.0 || product.exponent < -40 {
        return true;
    }
    return product.exponent < 1 && ldexp(size, product.exponent) < INTERIOR_CONTRACTION;
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

fn in_main_cardioid_or_bulb(c: vec2<f32>) -> bool {
    let y2 = c.y * c.y;
    let shifted = c.x - 0.25;
    let q = shifted * shifted + y2;
    let in_cardioid = q * (q + shifted) <= 0.25 * y2;
    let in_period2_bulb = (c.x + 1.0) * (c.x + 1.0) + y2 <= 0.0625;
    return in_cardioid || in_period2_bulb;
}
