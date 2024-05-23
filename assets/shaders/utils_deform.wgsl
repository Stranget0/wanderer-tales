#define_import_path wanderer_tales::my_noise
#import noisy_bevy::simplex_noise_2d_seeded;

const EPSILON:f32 = 0.1;

fn displace(pos: vec2<f32>) -> f32 {
	return simplex_noise_2d_seeded(pos / 100.0, 1.0) * 10.0;
}

fn displace_dt(pos: vec2<f32>, v: f32) -> vec2<f32> {
    let v_x = displace(pos + vec2(EPSILON, 0.0)) - v;
    let v_y = displace(pos + vec2(0.0, EPSILON)) - v;
    return vec2(v_x, v_y) / EPSILON;
}

fn compute_normal(v: f32, derivatives: vec2<f32>) -> vec3<f32> {
    // Construct the normal vector
    // The normal vector in tangent space is (d_x, d_y, 1)
    var normal = cross(vec3(-derivatives.x, 1.0, 1.0), vec3(1.0, 1.0,-derivatives.y));

    // Normalize the normal vector
    normal = normalize(normal);

    // Transform the normal vector to the [0, 1] range for normal map representation
    let normal_map = (normal * 0.5) + vec3(0.5, 0.5, 0.5);

    return normal_map;
}
