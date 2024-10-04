use core::f32;

use crate::prelude::*;

use super::fract_gl;

pub trait NoiseHasher {
    fn hash(&self, p: Vec2) -> f32;
    fn hash_2d(&self, p: Vec2) -> Vec2;
}

pub struct PcgHasher {
    seed: u32,
}

impl PcgHasher {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    pub fn next_hasher(&self) -> Self {
        Self {
            seed: self.seed.wrapping_add(1),
        }
    }

    fn pcg(&self, n: u32) -> u32 {
        // TODO: replace with something better
        let mut h = n.wrapping_mul(747796405).wrapping_add(2891336453);
        h = (h >> ((h >> 28).wrapping_add(4)) ^ h)
            .wrapping_mul(277803737)
            .wrapping_mul(self.seed);

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
    fn hash_2d(&self, mut p: Vec2) -> Vec2 {
        let k = vec2(f32::consts::FRAC_1_PI, 0.3678794);
        p = p * k + k.yx();
        -1.0 + 2.0 * (16.0 * k * fract_gl(p.x * p.y * (p.x + p.y))).fract_gl()
    }
}
