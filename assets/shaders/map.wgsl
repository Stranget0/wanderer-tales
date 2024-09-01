struct MapPosition {
    position: vec3<f32>,
}

struct MapPointData {
    height: f32,
}

@group(0) @binding(0) var<storage, read_write> input: array<MapPosition>;
@group(0) @binding(1) var<storage, read_write> output: array<MapPointData>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
     output[global_id.x] = MapPointData(input[global_id.x].position.x);
}
