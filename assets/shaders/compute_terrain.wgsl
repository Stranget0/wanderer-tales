#import wanderer_tales::noise::{
perlin_noise_2d,
Value2Dt2,
Value2Dt1,
mul_dt2_f,
dt2_length,
add_dt1_f,
mul_dt1_f,
div_dt1_dt1,
dt1_length,
div_dt1_f,
dt2_to_dt1,
add_dt1_dt1
}





const LAYERS_COUNT = 10;

struct LayerWeight {
    size: f32,
    amplitude: f32,
    erosion: f32,
    seed: u32,
}

struct Layer {
    sample: Value2Dt1,
    erosion: f32,
}


@group(0) @binding(0)
var<storage, read> positions: array<vec2<f32>>;

@group(0) @binding(1)
var<uniform> weights: array<LayerWeight, LAYERS_COUNT>;

@group(0) @binding(2)
var<storage, read_write> result: array<Value2Dt1>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let position = positions[global_id.x];
    result[global_id.x] = sample_many_base(position, LAYERS_COUNT).sample;
}

fn sample_many_base(
    pos: vec2<f32>,
) -> Layer {
    var erosion_factor = 0.0;
    var terrain = Value2Dt1(0.0, vec2<f32>(0.0, 0.0));

    for (var i: u32 = 0; i < LAYERS_COUNT; i = i + 1) {
        let weight: LayerWeight = weights[i];
        let layer: Layer = sample_erosion_base(pos, weight, erosion_factor);

        terrain = add_dt1_dt1(terrain, layer.sample);
        erosion_factor = layer.erosion;
    }

    return Layer(terrain, erosion_factor);
}

fn sample_erosion_base(
    pos: vec2<f32>,
    weight: LayerWeight,
    erosion_factor: f32,
) -> Layer {
    let layer: Value2Dt2 = sample_perlin_2d(pos, weight.size, weight.amplitude, weight.seed);
    let layer_steepiness: Value2Dt1 = dt2_length(layer);

    let pre_erosion_factor: Value2Dt1 = add_dt1_f(layer_steepiness, erosion_factor);
    let v: Value2Dt1 = add_dt1_f(mul_dt1_f(pre_erosion_factor, weight.erosion), 1.0);

    let layer_sample: Value2Dt2 = div_dt1_dt1(dt2_to_dt1(layer), v);
    let layer_erosion: f32 = dt1_length(div_dt1_f(dt2_to_dt1(layer), v.value));

    return Layer(layer_sample, layer_erosion);
}

fn sample_perlin_2d(pos: vec2<f32>, size: f32, amplitude: f32, seed: u32) -> Value2Dt2 {
    let v = perlin_noise_2d(pos, 1.0 / size, seed) / 2.0 + 0.5;
    return mul_dt2_f(v, amplitude);
}

