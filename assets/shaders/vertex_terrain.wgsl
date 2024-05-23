#import bevy_pbr::{
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

#import noisy_bevy::{simplex_noise_2d_seeded}
#import wanderer_tales::common::morph_vertex;
#import wanderer_tales::my_noise::{displace, displace_dt, compute_normal};

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
		out.world_position.y = displace(out.world_position.xz);
        let displaced_position = out.world_position.xyz;
		out.position = position_world_to_clip(out.world_position.xyz);
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
		var neighbour_1 = position + vec3(1.0,0.0, 0.0);
		neighbour_1.y = displace(neighbour_1.xz);
		var neighbour_2 = position + vec3(0.0,0.0, 1.0);
		neighbour_2.y = displace(neighbour_2.xz);

		var tangent = neighbour_1 - displaced_position;
		var bitangent = neighbour_2 - displaced_position;
		var displaced_normal = normalize(cross(bitangent,tangent ));

        let dt_normal = compute_normal(out.world_position.y, displace_dt(out.world_position.xz, out.world_position.y));
        let test_normal = dt_normal;

		// var bitangent = normalize(cross(normal, tangent));
		// var neighbour_1 = position + tangent;
		// var neighbour_2 = position + bitangent;

		// var displaced_neighbour_1 = neighbour_1 + normal * displace(neighbour_1);
		// var displaced_neighbour_2 = neighbour_2 + normal * displace(neighbour_2);

		// var displaced_tangent = displaced_neighbour_1 - displaced_position;
		// var displaced_bitangent = displaced_neighbour_2 - displaced_position;

		// var displaced_normal = normalize(cross(displaced_tangent, displaced_bitangent));
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
