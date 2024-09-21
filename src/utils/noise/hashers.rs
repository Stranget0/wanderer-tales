use bevy::prelude::*;

pub trait NoiseHasher {
    fn hash(&self, p: Vec2) -> f32;
}

pub struct PcgHasher {
    seed: u32,
}

impl PcgHasher {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn pcg(&self, n: u32) -> u32 {
        let mut h = n
            .wrapping_add(self.seed)
            .wrapping_mul(747796405)
            .wrapping_add(2891336453);
        h = (h >> ((h >> 28).wrapping_add(4)) ^ h).wrapping_mul(277803737);
        (h >> 22) ^ h
    }

    fn rand11(&self, f: f32) -> f32 {
        self.pcg(f.to_bits()) as f32 / 0xffffffff_u32 as f32
    }

    fn rand21(&self, p: Vec2) -> f32 {
        let n = p.x * 3.0 + p.y * 113.0;
        self.rand11(n)
    }
}

impl NoiseHasher for PcgHasher {
    fn hash(&self, p: Vec2) -> f32 {
        self.rand21(p)
    }
}
