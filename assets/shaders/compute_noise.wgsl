#import wanderer_tales::value_noise_2d

@group(0) @binding(2)
var<storage> input: array<vec2<i32>>;

@group(0) @binding(3)
var<storage, read_write> output: array<f32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    output[id.x] = value_noise_2d(vec2<f32>(input[id.x])).value;

}
