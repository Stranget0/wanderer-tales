#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

#import wanderer_tales::utils_noise::{value_noise_2d, ValueDt2}
#import wanderer_tales::utils_common::{morph_vertex}

const EPSILON:f32 = 0.1;

fn displace(pos: vec2<f32>) -> ValueDt2 {
        let data = value_noise_2d(pos / 100.0) ;
        return ValueDt2(data.value* 100, data.derivative);
}

fn displace_dt(pos: vec2<f32>, v: f32) -> vec2<f32> {
        let v_x = displace(pos + vec2(EPSILON, 0.0)).value - v;
        let v_y = displace(pos + vec2(0.0, EPSILON)).value - v;
        return vec2(v_x, v_y) / EPSILON;
}

fn compute_normal(derivative: vec2<f32>) -> vec3<f32> {
    return (vec3(-derivative.x, 1.0, -derivative.y));
}


@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var world_from_local = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
#endif

#ifdef VERTEX_NORMALS
        let dt_normal = compute_normal(displaced_data.derivative);
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(world_from_local, dt_normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        dt_normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
		let position = out.world_position.xyz;
		let displaced_data = displace(out.world_position.xz);
		out.world_position.y = displaced_data.value;
		let displaced_position = out.world_position.xyz;
		out.position = position_world_to_clip(displaced_position);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, world_from_local[3]);
#endif

    return out;
}