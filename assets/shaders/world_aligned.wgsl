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

#import bevy_pbr::pbr_types::{PbrInput, StandardMaterial, pbr_input_new};
#import bevy_pbr::pbr_bindings;
#import bevy_pbr::utils::HALF_PI;

struct Uniforms {
	uv_size: f32
}


@group(2) @binding(100)
var<uniform> uniforms: Uniforms; 

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
		var factors = box_project_blended_factors(in);
		var uv_size = uniforms.uv_size;

		// var in_modified: VertexOutput = clone_vertex_output(in);
		// var in_modified = world_aligned_in(in, uniforms.uv_size);
    // generate a PbrInput struct from the StandardMaterial bindings
		// Init Top / Bottom
		var xy_in:PbrInput;
		// if(factors.z >= 0){
			xy_in = tiled_projected_pbr_input(in, uv_size, in.world_position.xy, is_front);
		// } else {
			// xy_in = tiled_projected_pbr_input_negative(in, uv_size, in.world_position.xy, is_front);
		// }
		
		// Right / Left
		if (factors.x != 0.0) {
			var yz_in = tiled_projected_pbr_input(in, uv_size, in.world_position.yz, is_front);
			xy_in.material = blend_material(xy_in.material, yz_in.material, abs(factors.x));
		} 
		// else if (factors.x < 0) {
		// 	var yz_in = tiled_projected_pbr_input_negative(in, uv_size, in.world_position.yz, is_front);
		// 	xy_in.material = blend_material(xy_in.material, yz_in.material, -factors.x);
		// }

		// Front / Back
		if (factors.y != 0.0) {
			var xz_in = tiled_projected_pbr_input(in, uv_size, in.world_position.xz, is_front);
			xy_in.material = blend_material(xy_in.material, xz_in.material,abs(factors.y));
		} 
		// else if (factors.y < 0){
		// 	var xz_in = tiled_projected_pbr_input_negative(in, uv_size, in.world_position.xz, is_front);
		// 	xy_in.material = blend_material(xy_in.material, xz_in.material, -factors.y);
		// }

		var pbr_input = xy_in;

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

#endif

    return out;
}


fn blend_material(a: StandardMaterial, b: StandardMaterial, factor:f32) -> StandardMaterial {
	var copied = copy_material(a);
	copied.base_color = mix(a.base_color, b.base_color, factor);
	copied.emissive = mix(a.emissive, b.emissive, factor);
	copied.perceptual_roughness = mix(a.perceptual_roughness, b.perceptual_roughness, factor);
	copied.metallic = mix(a.metallic, b.metallic, factor);
	copied.reflectance = mix(a.reflectance, b.reflectance, factor);
	

	return copied;

// 	struct StandardMaterial {
//     base_color: vec4<f32>,
//     emissive: vec4<f32>,
//     attenuation_color: vec4<f32>,
//     uv_transform: mat3x3<f32>,
//     perceptual_roughness: f32,
//     metallic: f32,
//     reflectance: f32,
//     diffuse_transmission: f32,
//     specular_transmission: f32,
//     thickness: f32,
//     ior: f32,
//     attenuation_distance: f32,
//     // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
//     flags: u32,
//     alpha_cutoff: f32,
//     parallax_depth_scale: f32,
//     max_parallax_layer_count: f32,
//     lightmap_exposure: f32,
//     max_relief_mapping_search_steps: u32,
//     /// ID for specifying which deferred lighting pass should be used for rendering this material, if any.
//     deferred_lighting_pass_id: u32,
// };
}

fn copy_material(in: StandardMaterial) -> StandardMaterial {
	var copied: StandardMaterial;
	copied.base_color = in.base_color;
	copied.emissive = in.emissive;
	copied.attenuation_color = in.attenuation_color;
	// copied.uv_transform = in.uv_transform;
	copied.perceptual_roughness = in.perceptual_roughness;
	copied.metallic = in.metallic;
	copied.reflectance = in.reflectance;
	copied.diffuse_transmission = in.diffuse_transmission;
	copied.specular_transmission = in.specular_transmission;
	copied.thickness = in.thickness;
	copied.ior = in.ior;
	copied.attenuation_distance = in.attenuation_distance;
	copied.flags = in.flags;
	copied.alpha_cutoff = in.alpha_cutoff;
	copied.parallax_depth_scale = in.parallax_depth_scale;
	copied.max_parallax_layer_count = in.max_parallax_layer_count;
	copied.lightmap_exposure = in.lightmap_exposure;
	copied.max_relief_mapping_search_steps = in.max_relief_mapping_search_steps;
	copied.deferred_lighting_pass_id = in.deferred_lighting_pass_id;

	return copied;
}


fn tiled_projected_pbr_input(
	in: VertexOutput,
	uv_size: f32,
	coordinate: vec2<f32>,
	is_front: bool,
) -> PbrInput {
	var in_cloned = clone_vertex_output(in);
	in_cloned.uv = fract(coordinate * uv_size);
	var pbr_input = pbr_input_from_standard_material(in_cloned, is_front);
	
	return pbr_input;
}

fn tiled_projected_pbr_input_negative(
	in: VertexOutput,
	uv_size: f32,
	coordinate: vec2<f32>,
	is_front: bool,
) -> PbrInput {
	var in_cloned = clone_vertex_output(in);
	in_cloned.uv = 1.0 - fract(coordinate * uv_size);
	var pbr_input = pbr_input_from_standard_material(in_cloned, is_front);
	
	return pbr_input;
}

fn box_project_blended_factors(in: VertexOutput) -> vec3<f32> {
		var factors = in.world_normal;
		factors.z *= 2.0;

		return normalize(factors);
}

fn clone_vertex_output(in: VertexOutput) -> VertexOutput {
		var in_modified: VertexOutput;
		in_modified.position = in.position;
    in_modified.world_position = in.world_position;
    in_modified.world_normal = in.world_normal;
		#ifdef MESHLET_MESH_MATERIAL_PASS
				in_modified.flags = in.mesh_flags;
		#endif
		#ifdef VERTEX_UVS
				in_modified.uv = in.uv;
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

		return in_modified;
}