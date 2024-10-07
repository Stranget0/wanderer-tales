use core::f32;

use crate::prelude::*;

pub trait NoiseHasher {
    fn hash(v: i32) -> u32;

    fn seed(&self) -> u32;

    fn hashf(v: i32) -> f32 {
        u_to_f(Self::hash(v))
    }

    fn hash_22f(v: IVec2) -> Vec2 {
        vec2(Self::hashf(v.x), Self::hashf(v.y))
    }
    fn hash_33f(v: IVec3) -> Vec3 {
        vec3(Self::hashf(v.x), Self::hashf(v.y), Self::hashf(v.z))
    }
    fn hash_44f(v: IVec4) -> Vec4 {
        vec4(
            Self::hashf(v.x),
            Self::hashf(v.y),
            Self::hashf(v.z),
            Self::hashf(v.w),
        )
    }
    fn hash_21f(v: IVec2) -> f32 {
        let v = Self::hash((v.x as u32 ^ Self::hash(v.y)) as i32);
        u_to_f(v)
    }
    fn hash_31f(v: IVec3) -> f32 {
        let v = Self::hash((v.x as u32 ^ Self::hash(v.y) ^ Self::hash(v.z)) as i32);
        u_to_f(v)
    }
    fn hash_41f(v: IVec4) -> f32 {
        let v =
            Self::hash((v.x as u32 ^ Self::hash(v.y) ^ Self::hash(v.z) ^ Self::hash(v.w)) as i32);
        u_to_f(v)
    }

    fn hashf_seeded(&self, v: u32) -> f32 {
        Self::hashf((v ^ self.seed()) as i32)
    }
    fn hash_22f_seeded(&self, v: IVec2) -> Vec2 {
        Self::hash_22f((v.as_uvec2() ^ UVec2::splat(self.seed())).as_ivec2())
    }
    fn hash_33f_seeded(&self, v: IVec3) -> Vec3 {
        Self::hash_33f((v.as_uvec3() ^ UVec3::splat(self.seed())).as_ivec3())
    }
    fn hash_44f_seeded(&self, v: IVec4) -> Vec4 {
        Self::hash_44f((v.as_uvec4() ^ UVec4::splat(self.seed())).as_ivec4())
    }
}

pub struct PcgHasher {
    seed: u32,
}

impl PcgHasher {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    // https://www.pcg-random.org/
    fn pcg(v: u32) -> u32 {
        let state = v.wrapping_mul(747796405).wrapping_add(2891336453);
        let word = ((state >> ((state >> 28).wrapping_add(4))) ^ state).wrapping_mul(277803737);
        (word >> 22) ^ word
    }

    fn pcg2d(mut v: UVec2) -> UVec2 {
        v.x = v.x.wrapping_mul(1664525).wrapping_add(1013904223);
        v.y = v.y.wrapping_mul(1664525).wrapping_add(1013904223);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(1664525));
        v.y = v.y.wrapping_add(v.x.wrapping_mul(1664525));

        v = v ^ (v >> 16);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(1664525));
        v.y = v.y.wrapping_add(v.x.wrapping_mul(1664525));

        v = v ^ (v >> 16);

        v
    }

    // http://www.jcgt.org/published/0009/03/02/
    fn pcg3d(mut v: UVec3) -> UVec3 {
        v.x = v.x.wrapping_mul(1664525).wrapping_add(1013904223);
        v.y = v.y.wrapping_mul(1664525).wrapping_add(1013904223);
        v.z = v.z.wrapping_mul(1664525).wrapping_add(1013904223);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(v.z));
        v.y = v.y.wrapping_add(v.z.wrapping_mul(v.x));
        v.z = v.z.wrapping_add(v.x.wrapping_mul(v.y));

        v = v ^ (v >> 16);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(v.z));
        v.y = v.y.wrapping_add(v.z.wrapping_mul(v.x));
        v.z = v.z.wrapping_add(v.x.wrapping_mul(v.y));

        v
    }

    // http://www.jcgt.org/published/0009/03/02/
    fn pcg3d16(mut v: UVec3) -> UVec3 {
        v.x = v.x.wrapping_mul(12829).wrapping_add(47989);
        v.y = v.y.wrapping_mul(12829).wrapping_add(47989);
        v.z = v.z.wrapping_mul(12829).wrapping_add(47989);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(v.z));
        v.y = v.y.wrapping_add(v.z.wrapping_mul(v.x));
        v.z = v.z.wrapping_add(v.x.wrapping_mul(v.y));

        v.x = v.x.wrapping_add(v.y.wrapping_mul(v.z));
        v.y = v.y.wrapping_add(v.z.wrapping_mul(v.x));
        v.z = v.z.wrapping_add(v.x.wrapping_mul(v.y));

        v = v >> 16;

        v
    }

    // http://www.jcgt.org/published/0009/03/02/
    fn pcg4d(mut v: UVec4) -> UVec4 {
        v.x = v.x.wrapping_mul(1664525).wrapping_add(1013904223);
        v.y = v.y.wrapping_mul(1664525).wrapping_add(1013904223);
        v.z = v.z.wrapping_mul(1664525).wrapping_add(1013904223);
        v.w = v.w.wrapping_mul(1664525).wrapping_add(1013904223);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(v.w));
        v.y = v.y.wrapping_add(v.z.wrapping_mul(v.x));
        v.z = v.z.wrapping_add(v.x.wrapping_mul(v.y));
        v.w = v.w.wrapping_add(v.y.wrapping_mul(v.z));

        v = v ^ (v >> 16);

        v.x = v.x.wrapping_add(v.y.wrapping_mul(v.w));
        v.y = v.y.wrapping_add(v.z.wrapping_mul(v.x));
        v.z = v.z.wrapping_add(v.x.wrapping_mul(v.y));
        v.w = v.w.wrapping_add(v.y.wrapping_mul(v.z));

        v
    }
}

impl NoiseHasher for PcgHasher {
    fn hash(v: i32) -> u32 {
        Self::pcg(v as u32)
    }
    fn seed(&self) -> u32 {
        self.seed
    }

    fn hash_22f(v: IVec2) -> Vec2 {
        let v = Self::pcg2d(v.as_uvec2());
        vec2(u_to_f(v.x), u_to_f(v.y))
    }

    fn hash_33f(v: IVec3) -> Vec3 {
        let v = Self::pcg3d(v.as_uvec3());
        vec3(u_to_f(v.x), u_to_f(v.y), u_to_f(v.z))
    }

    fn hash_44f(v: IVec4) -> Vec4 {
        let v = Self::pcg4d(v.as_uvec4());
        vec4(u_to_f(v.x), u_to_f(v.y), u_to_f(v.z), u_to_f(v.w))
    }
}

fn u_to_f(v: u32) -> f32 {
    v as f32 / u32::MAX as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::utils::hashbrown::HashMap;

    fn calculate_entropy<T: std::hash::Hash + Eq>(samples: &[T]) -> f64 {
        let mut frequencies = HashMap::new();
        let total_samples = samples.len() as f64;

        // Count frequencies of each unique output
        for sample in samples {
            *frequencies.entry(sample).or_insert(0) += 1;
        }

        // Calculate entropy
        let entropy = frequencies
            .values()
            .map(|&count| {
                let probability = count as f64 / total_samples;
                if probability > 0.0 {
                    -probability * probability.log2()
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        entropy / size_of::<T>() as f64
    }

    #[test]
    fn pcg_seed_difference_22() {
        let hasher_f = |seed: u32, pos: Vec2| PcgHasher::new(seed).hash_22f_seeded(pos.as_ivec2());
        assert_seeds_difference(hasher_f, |a, b| {
            (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01
        });
    }

    // #[test]
    // fn pcg_seed_entropy_22() {
    //     let hasher_f = |seed: u32, pos: Vec2| PcgHasher::new(seed).hash_22f_seeded(pos.as_ivec2());
    //     let sample = generate_sample(hasher_f, 0)
    //         .into_iter()
    //         .map(|v| uvec2(v.x.to_bits(), v.y.to_bits()))
    //         .collect_vec();
    //
    //     panic!("{}", calculate_entropy(&sample));
    // }

    #[test]
    fn pcg_range() {
        let hasher = PcgHasher::new(0);
        let mut values = Vec::new();
        for i in -1000_i32..1000_i32 {
            let v = hasher.hashf_seeded(i as u32);
            values.push(v);
        }
        let max = *values
            .iter()
            .max_by(|a, b| a.to_owned().partial_cmp(b.to_owned()).unwrap())
            .unwrap();

        let min = *values
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let avg = values.iter().sum::<f32>() / values.len() as f32;

        assert!(min < 0.001, "min is too big: {min}");
        assert!(max > 0.999, "max is too small: {max}");
        assert!((avg - 0.5).abs() < 0.01, "avg is too far from 0.5: {avg}");
    }

    fn assert_seeds_difference<T: std::ops::Sub, F: Fn(u32, Vec2) -> T, C: Fn(&T, &T) -> bool>(
        f: F,
        checker: C,
    ) {
        let mut samples = Vec::new();
        for seed in 0..10 {
            let sample = generate_sample(&f, seed);
            samples.push(sample);
        }

        for (i, sample1) in samples.iter().enumerate() {
            for (j, sample2) in samples.iter().enumerate() {
                if i == j {
                    continue;
                }

                let mut equal_sum = 0;
                for (a, b) in sample1.iter().zip_eq(sample2) {
                    if checker(a, b) {
                        equal_sum += 1;
                    }
                }

                let average_eq = equal_sum as f32 / sample1.len() as f32;
                assert!(
                    average_eq < 0.5,
                    "Samples groups {i} and {j} are too similar, which ratio is {}%",
                    (average_eq * 100.0).floor()
                );
            }
        }
    }

    fn generate_sample<T, F: Fn(u32, Vec2) -> T>(f: F, seed: u32) -> Vec<T> {
        let mut sample = Vec::new();
        for x in -100..100 {
            for y in -100..100 {
                let pos = vec2(x as f32 / 3.0, y as f32 / 3.0);
                let v = f(seed, pos);
                sample.push(v);
            }
        }
        sample
    }
}
