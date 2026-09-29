const PALETTE_SIZE: u32 = 16u;
// Iteration counts are split at this power of two so their palette position stays exact in f32
const ITERATION_BLOCK_BITS: u32 = 12u;
const LIGHT_HEIGHT: f32 = 1.0;
const AMBIENT_LIGHT: f32 = 0.3;
const BRIGHTNESS_BOOST: f32 = 1.3;
const OUTLINE_WIDTH: f32 = 0.5;
const WHITE: u32 = 0xffffffffu;

@compute @workgroup_size(8, 8)
fn color(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.width || id.y >= params.rows {
        return;
    }
    let index = id.y * params.width + id.x;
    let sample = samples[index];
    if sample.iterations >= UNDECIDED {
        pixels[index] = WHITE;
        return;
    }
    var rgb = palette(palette_position(sample));
    if params.normal_shading != 0u {
        rgb = shade(rgb, sample.normal);
    }
    if params.outline != 0u {
        let t = smoothstep(0.0, OUTLINE_WIDTH, sample.distance);
        rgb = floor(mix(params.outline_color.rgb, rgb, t));
    }
    let channels = vec3<u32>(rgb);
    pixels[index] = channels.x | (channels.y << 8u) | (channels.z << 16u) | 0xff000000u;
}

/// Smooth iterations / band scale + phase, reduced modulo the palette size.
fn palette_position(sample: Sample) -> f32 {
    let nu = log2(0.5 * log2(sample.norm_sqr));
    let block_mask = (1u << ITERATION_BLOCK_BITS) - 1u;
    let blocks = f32(sample.iterations >> ITERATION_BLOCK_BITS) * params.band_cycle;
    let rest = f32(sample.iterations & block_mask) + 1.0 - nu;
    let position = blocks + rest * params.inv_band_scale + params.band_phase;
    return position - f32(PALETTE_SIZE) * floor(position / f32(PALETTE_SIZE));
}

fn palette(position: f32) -> vec3<f32> {
    let index = u32(position) % PALETTE_SIZE;
    let start = params.palette[index].rgb;
    let end = params.palette[(index + 1u) % PALETTE_SIZE].rgb;
    return floor(mix(start, end, fract(position)));
}

fn shade(rgb: vec3<f32>, normal: vec2<f32>) -> vec3<f32> {
    let t = (dot(normal, params.light) + LIGHT_HEIGHT) / (1.0 + LIGHT_HEIGHT);
    let light = clamp(t * (1.0 - AMBIENT_LIGHT) + AMBIENT_LIGHT, 0.0, 1.0);
    return min(floor(rgb * light * BRIGHTNESS_BOOST), vec3(255.0));
}
