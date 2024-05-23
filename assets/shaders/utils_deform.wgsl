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

fn compute_normal(gradient: vec2<f32>) -> vec3<f32> {
    let normal = vec3(-gradient.x, 1.0, -gradient.y);
    return normalize(normal);
}
