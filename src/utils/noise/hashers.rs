use crate::prelude::*;

pub trait NoiseHasher {
    fn hash(v: i32) -> u32;

    fn seed(&self) -> u32;

    fn from_seed(seed: u32) -> Self;
    fn with_next_seed(&self) -> Self;

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

#[derive(Clone)]
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
    fn from_seed(seed: u32) -> Self {
        Self::new(seed)
    }
    fn with_next_seed(&self) -> Self {
        Self::new(self.seed.wrapping_add(1))
    }

    fn hash_22f(p: IVec2) -> Vec2 {
        // 2D -> 1D
        //let mut n = p.x * ivec2(3, 37) + p.y * ivec2(311, 113);
        let mut n_x = p.x.wrapping_mul(3).wrapping_add(p.y.wrapping_mul(311));
        let mut n_y = p.x.wrapping_mul(37).wrapping_add(p.y.wrapping_mul(113));

        // 1D hash by Hugo Elias
        n_x = (n_x << 13) ^ n_x;
        n_y = (n_y << 13) ^ n_y;
        // n = n * (n * n * 15731 + 789221) + 1376312589;
        n_x = n_x
            .wrapping_mul(
                n_x.wrapping_mul(n_x)
                    .wrapping_mul(15731)
                    .wrapping_add(789221),
            )
            .wrapping_add(1376312589);
        n_y = n_y
            .wrapping_mul(
                n_y.wrapping_mul(n_y)
                    .wrapping_mul(15731)
                    .wrapping_add(789221),
            )
            .wrapping_add(1376312589);

        // return -1.0 + 2.0 * vec2(n & ivec2(0x0fffffff)) / float(0x0fffffff);
        let n_x = -1.0 + 2.0 * (n_x & 0x0fffffff) as f32 / 0x0fffffff as f32;
        let n_y = -1.0 + 2.0 * (n_y & 0x0fffffff) as f32 / 0x0fffffff as f32;

        vec2(n_x, n_y)
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

    fn calculate_entropy(data: &[f32], bins: usize) -> f64 {
        let histogram = calculate_histogram(bins, data);

        // Convert histogram counts to probabilities
        let total: usize = histogram.iter().sum();
        let probabilities: Vec<f64> = histogram
            .into_iter()
            .map(|count| count as f64 / total as f64)
            .collect();

        // Calculate Shannon entropy
        let mut entropy = 0.0;
        for &p in &probabilities {
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    fn calculate_histogram(bins: usize, data: &[f32]) -> Vec<usize> {
        let mut histogram = vec![0; bins];

        // Fill the histogram based on bin distribution
        for &value in data {
            let bin = (value * (bins as f32)) as usize;
            if bin < bins {
                histogram[bin] += 1;
            }
        }
        histogram
    }

    #[test]
    fn pcg_seed_difference_22() {
        let hasher_f = |seed: u32, pos: Vec2| PcgHasher::new(seed).hash_22f_seeded(pos.as_ivec2());
        assert_seeds_difference(hasher_f, |a, b| {
            (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() < 0.01
        });
    }

    #[test]
    fn pcg_seed_entropy_22() {
        let hasher_f = |seed: u32, pos: Vec2| PcgHasher::new(seed).hash_22f_seeded(pos.as_ivec2());
        let sample = generate_sample(hasher_f, 0)
            .into_iter()
            .flat_map(|v| [v.x, v.y])
            .collect_vec();

        let entropy = calculate_entropy(&sample, 100);

        // Maximum theoretical entropy for 100 bins (log2(100) ≈ 6.64)
        let max_entropy = 6.64;
        let min_entropy = 6.0; // For some margin of error

        // Assert entropy is within a reasonable range
        assert!(
            entropy >= min_entropy && entropy <= max_entropy,
            "Entropy is outside expected range: {}",
            entropy
        );
    }

    #[test]
    fn pcg_seed_distribution_22() {
        let hasher_f = |seed: u32, pos: Vec2| PcgHasher::new(seed).hash_22f_seeded(pos.as_ivec2());
        let sample = generate_sample(hasher_f, 0)
            .into_iter()
            .flat_map(|v| [v.x, v.y])
            .collect_vec();

        // println!("sample: {:?}", sample);

        let histogram = calculate_histogram(100, &sample);

        // Assert histogram is uniform
        // Calculate the expected average count per bin
        let avg_count = sample.len() as f32 / 100.0; // / bins
        let tolerance = avg_count * 0.1; // 10% tolerance

        for count in histogram.clone() {
            assert!(
                (count as f32 - avg_count).abs() <= tolerance,
                // "avg_count = {}, count = {},\n{histogram:#?}",
                // avg_count,
                // count
            );
        }
    }

    #[test]
    fn pcg_range() {
        let hasher = PcgHasher::new(0);
        let mut values = Vec::new();
        for i in -1000_i32..1000_i32 {
            let v = hasher.hash_22f_seeded(ivec2(i, i));
            values.push(v.x);
            values.push(v.y);
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

        assert!(min < 0.01, "min is too big: {min}");
        assert!(max > 0.99, "max is too small: {max}");
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
