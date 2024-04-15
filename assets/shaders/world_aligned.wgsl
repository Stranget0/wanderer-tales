#import bevy_pbr::pbr_fragment::pbr_input_from_standard_material;
#import bevy_pbr::pbr_functions::alpha_discard;


#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::forward_io::{VertexOutput, FragmentOutput};
#import bevy_pbr::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};
#endif

#import bevy_pbr::pbr_bindings;


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
		var in_modified: VertexOutput;
		in_modified.position = in.position;
    in_modified.world_position = in.world_position;
    in_modified.world_normal = in.world_normal;
		#ifdef MESHLET_MESH_MATERIAL_PASS
				in_modified.flags = in.mesh_flags;
		#endif
		#ifdef VERTEX_UVS
				in_modified.uv = fract(in.world_position.xy * 0.1);
		#endif
		#ifdef VERTEX_UVS_B
				in_modified.uv_b = in.uv_b;
		#endif
		#ifdef VERTEX_TANGENTS
				in_modified.world_tangent = in.world_tangent;
		#endif
		#ifdef VERTEX_COLORS
				in_modified.color = in.color;
		#endif
		#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
				in_modified.instance_index = in.instance_index;
		#endif

    // generate a PbrInput struct from the StandardMaterial bindings
		var pbr_input = pbr_input_from_standard_material(in_modified, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
		// pbr_input.material.base_color = textureSample(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, fract(in.world_position.xy * 0.1));

		
    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in_in_modified, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
		out.color = apply_pbr_lighting(pbr_input);



    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    // out.color = out.color * 2.0;
#endif

    return out;
}