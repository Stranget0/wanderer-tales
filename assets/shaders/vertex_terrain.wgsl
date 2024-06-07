#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

#import wanderer_tales::utils_noise::{value_noise_2d}

const EPSILON:f32 = 0.1;

fn displace(pos: vec2<f32>) -> vec3<f32> {
        let data = value_noise_2d(pos / 100.0) ;
        return vec3(data.x* 100, data.yz);
}

fn displace_dt(pos: vec2<f32>, v: f32) -> vec2<f32> {
        let v_x = displace(pos + vec2(EPSILON, 0.0)).x - v;
        let v_y = displace(pos + vec2(0.0, EPSILON)).x - v;
        return vec2(v_x, v_y) / EPSILON;
}

fn compute_normal(derivative: vec2<f32>) -> vec3<f32> {
    return (vec3(-derivative.x, 1.0, -derivative.y));
}

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if weight == 0.0 {
                        continue;
        }
        vertex.position += weight * morph(vertex.index, bevy_pbr,:: morph,:: position_offset, i);
        #ifdef VERTEX_NORMALS
        vertex.normal += weight * morph(vertex.index, bevy_pbr,:: morph,:: normal_offset, i);
        #endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph(vertex.index, bevy_pbr,:: morph,:: tangent_offset, i), 0.0);
        #endif
    }
    return vertex;
}
#endif

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var model = mesh_functions::get_model_matrix(vertex_no_morph.instance_index);
#endif


#ifdef VERTEX_POSITIONS
		out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4(vertex.position, 1.0));
        let position = out.world_position.xyz;
        let displaced_data = displace(out.world_position.xz);
		out.world_position.y = displaced_data.x;
        let displaced_position = out.world_position.xyz;
		out.position = position_world_to_clip(displaced_position);
#endif

#ifdef VERTEX_TANGENTS
		out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_NORMALS
        let dt2 = displaced_data.yz;
        let dt_normal = compute_normal(dt2);
        let test_normal = dt_normal;
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(model, test_normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        test_normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif


#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

    return out;
}
