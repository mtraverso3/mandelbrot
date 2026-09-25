const PALETTE_SIZE: u32 = 16u;
// Iteration counts are split at this power of two so their palette position stays exact in f32
const ITERATION_BLOCK_BITS: u32 = 12u;
const LIGHT_HEIGHT: f32 = 1.0;
const AMBIENT_LIGHT: f32 = 0.3;
const BRIGHTNESS_BOOST: f32 = 1.3;
const WHITE: u32 = 0xffffffffu;

var<private> PALETTE: array<vec3<f32>, 16> = array(
    vec3(66.0, 30.0, 15.0),
    vec3(25.0, 7.0, 26.0),
    vec3(9.0, 1.0, 47.0),
    vec3(4.0, 4.0, 73.0),
    vec3(0.0, 7.0, 100.0),
    vec3(12.0, 44.0, 138.0),
    vec3(24.0, 82.0, 177.0),
    vec3(57.0, 125.0, 209.0),
    vec3(134.0, 181.0, 229.0),
    vec3(211.0, 236.0, 248.0),
    vec3(241.0, 233.0, 191.0),
    vec3(248.0, 201.0, 95.0),
    vec3(255.0, 170.0, 0.0),
    vec3(204.0, 128.0, 0.0),
    vec3(153.0, 87.0, 0.0),
    vec3(106.0, 52.0, 3.0),
);

@compute @workgroup_size(8, 8)
fn color(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= params.width || id.y >= params.rows {
        return;
    }
    let index = id.y * params.width + id.x;
    let sample = samples[index];
    if sample.iterations == INTERIOR {
        pixels[index] = WHITE;
        return;
    }
    var rgb = palette(palette_position(sample));
    if params.normal_shading != 0u {
        rgb = shade(rgb, sample.normal);
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
    let start = PALETTE[index];
    let end = PALETTE[(index + 1u) % PALETTE_SIZE];
    return floor(mix(start, end, fract(position)));
}

fn shade(rgb: vec3<f32>, normal: vec2<f32>) -> vec3<f32> {
    let t = (dot(normal, params.light) + LIGHT_HEIGHT) / (1.0 + LIGHT_HEIGHT);
    let light = clamp(t * (1.0 - AMBIENT_LIGHT) + AMBIENT_LIGHT, 0.0, 1.0);
    return min(floor(rgb * light * BRIGHTNESS_BOOST), vec3(255.0));
}
