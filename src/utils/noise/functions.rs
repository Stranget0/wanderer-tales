use super::*;

// Value noise and gradient noise
pub fn value_noise_2d(p: Vec2, hasher: &impl NoiseHasher) -> ValueDt2 {
    let i = p.floor();
    let f = p.fract_gl();

    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f - 2.0) + 1.0);

    let va = hasher.hash(i + vec2(0.0, 0.0));
    let vb = hasher.hash(i + vec2(1.0, 0.0));
    let vc = hasher.hash(i + vec2(0.0, 1.0));
    let vd = hasher.hash(i + vec2(1.0, 1.0));

    ValueDt2::new(
        va + (vb - va) * u.x + (vc - va) * u.y + (va - vb - vc + vd) * u.x * u.y,
        du * (u.yx() * (va - vb - vc + vd) + vec2(vb, vc) - va),
    )
}
