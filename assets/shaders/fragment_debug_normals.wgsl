#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.world_normal, 1.0);
}
