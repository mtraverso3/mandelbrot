// Past zoom 1e30, pixel offsets start below the smallest normal f32. Until they grow past
// 2^PLAIN_EXPONENT they are iterated as mantissa * 2^exponent, against the orbit and the
// skip table kept the same way; from then on plain f32 is exact enough.

const PLAIN_EXPONENT: i32 = -64;

struct Fx {
    mantissa: vec2<f32>,
    exponent: i32,
}

fn fx_normalize(x: Fx) -> Fx {
    let largest = max(abs(x.mantissa.x), abs(x.mantissa.y));
    if largest == 0.0 {
        return x;
    }
    let shift = frexp(largest).exp;
    return Fx(ldexp(x.mantissa, vec2(-shift)), x.exponent + shift);
}

fn fx_add(a: Fx, b: Fx) -> Fx {
    // A zero's exponent says nothing, so it must not decide the alignment
    if all(a.mantissa == vec2(0.0)) {
        return b;
    }
    if all(b.mantissa == vec2(0.0)) {
        return a;
    }
    let exponent = max(a.exponent, b.exponent);
    let mantissa = ldexp(a.mantissa, vec2(a.exponent - exponent)) + ldexp(b.mantissa, vec2(b.exponent - exponent));
    return Fx(mantissa, exponent);
}

fn fx_mul(a: vec2<f32>, a_exponent: i32, b: Fx) -> Fx {
    return Fx(mul(a, b.mantissa), a_exponent + b.exponent);
}

/// |a| < |b| for normalized values.
fn fx_smaller(a: Fx, b_size: f32, b_exponent: i32) -> bool {
    let a_size = length(a.mantissa);
    if b_size == 0.0 {
        return false;
    }
    if a_size == 0.0 {
        return true;
    }
    let shift = a.exponent - b_exponent;
    if shift > 64 {
        return false;
    }
    if shift < -64 {
        return true;
    }
    return ldexp(a_size, shift) < b_size;
}

fn deep_step(state_in: State, index: u32, pixel: vec2<f32>) -> State {
    var state = state_in;
    let delta = Fx(state.delta, state.delta_exponent);
    let dc = Fx(pixel * params.pixel_mantissa, params.pixel_exponent);
    var next: Fx;

    var level = -1;
    if SKIP {
        level = deep_skip_level(state.m, delta, params.max_iterations - state.n);
    }
    if level >= 0 {
        let range = bla_levels[level];
        let skip = bla[range.x + ((state.m - 1u) >> u32(level))];
        next = fx_add(fx_mul(skip.a_mantissa, skip.a_exponent, delta), fx_mul(skip.b_mantissa, skip.b_exponent, dc));
        if DETECT_INTERIOR && contract(&state, skip.a_mantissa, skip.a_exponent) {
            return finish_interior(state, index);
        }
        if TRACK_DERIVATIVE {
            let derivative = Fx(state.derivative, state.exponent);
            let skipped = fx_normalize(fx_add(
                fx_mul(skip.a_mantissa, skip.a_exponent, derivative),
                Fx(skip.b_mantissa, skip.b_exponent),
            ));
            state.derivative = skipped.mantissa;
            state.exponent = skipped.exponent;
        }
        state.m += 1u << u32(level);
        state.n += 1u << u32(level);
    } else {
        let point = orbit[state.m];
        let z = point.z + ldexp(delta.mantissa, vec2(delta.exponent));
        let norm_sqr = dot(z, z);
        if norm_sqr > ESCAPE_RADIUS_SQR {
            samples[index] = Sample(state.n, norm_sqr, normal(z, state.derivative));
            state.done = 1u;
            return state;
        }
        if DETECT_INTERIOR && state.n > 0u && contract(&state, 2.0 * z, 0) {
            return finish_interior(state, index);
        }
        if TRACK_DERIVATIVE {
            state.derivative = 2.0 * mul(state.derivative, z) + vec2(exp2_neg(state.exponent), 0.0);
            if max(abs(state.derivative.x), abs(state.derivative.y)) > DERIVATIVE_LIMIT {
                state = normalize_derivative(state);
            }
        }

        var offset = delta;
        let full = fx_normalize(fx_add(Fx(point.mantissa, point.exponent), delta));
        if state.m == params.last || fx_smaller(full, length(delta.mantissa), delta.exponent) {
            offset = full;
            state.m = 0u;
        }
        let reference = orbit[state.m];
        let twice = fx_mul(2.0 * reference.mantissa, reference.exponent, offset);
        let square = Fx(mul(offset.mantissa, offset.mantissa), 2 * offset.exponent);
        next = fx_add(fx_add(twice, square), dc);
        state.m++;
        state.n++;
    }

    let normalized = fx_normalize(next);
    if normalized.exponent > PLAIN_EXPONENT {
        state.delta = ldexp(normalized.mantissa, vec2(normalized.exponent));
        state.delta_exponent = 0;
    } else {
        state.delta = normalized.mantissa;
        state.delta_exponent = normalized.exponent;
    }
    return state;
}

/// `skip_level` for an offset kept as mantissa and exponent.
fn deep_skip_level(m: u32, delta: Fx, budget: u32) -> i32 {
    if m == 0u {
        return -1;
    }
    let index = m - 1u;
    let aligned = min(countTrailingZeros(index), params.bla_levels - 1u);
    var found = -1;
    for (var level = 1u; level <= aligned; level++) {
        let range = bla_levels[level];
        let j = index >> level;
        if j >= range.y || (1u << level) > budget {
            break;
        }
        let step = bla[range.x + j];
        if !fx_smaller(delta, step.radius_mantissa, step.radius_exponent) {
            break;
        }
        found = i32(level);
    }
    return found;
}
