pub use bevy::math::*;

pub struct ValueDt2 {
    pub value: f32,
    pub derivative: Vec2,
}
impl ValueDt2 {
    pub fn new(value: f32, derivative: Vec2) -> Self {
        Self { value, derivative }
    }
    pub fn get_normal(&self) -> Vec3 {
        vec3(-self.derivative.x, 1.0, -self.derivative.x)
    }
}

pub struct ValueDt3 {
    pub value: f32,
    pub derivative: Vec3,
}
impl ValueDt3 {
    pub fn new(value: f32, derivative: Vec3) -> Self {
        Self { value, derivative }
    }
}

// Wrapping pcg function
fn pcg(n: u32) -> u32 {
    let mut h = n.wrapping_mul(747796405).wrapping_add(2891336453);
    h = (h >> ((h >> 28).wrapping_add(4)) ^ h).wrapping_mul(277803737);
    (h >> 22) ^ h
}

fn rand11(f: f32) -> f32 {
    pcg(f.to_bits()) as f32 / 0xffffffff_u32 as f32
}

fn rand21(p: Vec2) -> f32 {
    let n = p.x * 3.0 + p.y * 113.0;
    rand11(n)
}

// Value noise and gradient noise
pub fn value_noise_2d(p: Vec2) -> ValueDt2 {
    let i = p.floor();
    let f = p.fract_gl();

    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let du = 30.0 * f * f * (f * (f - 2.0) + 1.0);

    let va = rand21(i + vec2(0.0, 0.0));
    let vb = rand21(i + vec2(1.0, 0.0));
    let vc = rand21(i + vec2(0.0, 1.0));
    let vd = rand21(i + vec2(1.0, 1.0));

    ValueDt2::new(
        va + (vb - va) * u.x + (vc - va) * u.y + (va - vb - vc + vd) * u.x * u.y,
        du * (u.yx() * (va - vb - vc + vd) + vec2(vb, vc) - va),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_noise_2d() {
        for x in -10000..10000 {
            for y in -10000..10000 {
                let v = value_noise_2d(vec2(x as f32 / 100.0, y as f32 / 100.0));
                assert!(v.value.abs() <= 1.0);
            }
        }
    }
}
