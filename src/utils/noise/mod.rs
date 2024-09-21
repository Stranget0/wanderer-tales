mod data;
mod functions;
mod hashers;

pub use bevy::math::*;
pub use data::*;
pub use functions::*;
pub use hashers::*;

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: [u32; 2] = [0, u32::MAX];

    #[test]
    fn should_over_or_equal_0() {
        for seed in SEEDS.into_iter() {
            let hasher = PcgHasher::new(seed);
            for x in -10000..10000 {
                for y in -10000..10000 {
                    let v = value_noise_2d(vec2(x as f32 / 100.0, y as f32 / 100.0), &hasher);
                    assert!(v.value >= 0.0);
                }
            }
        }
    }
    #[test]
    fn should_less_or_equal_1() {
        for seed in SEEDS {
            let hasher = PcgHasher::new(seed);
            for x in -10000..10000 {
                for y in -10000..10000 {
                    let v = value_noise_2d(vec2(x as f32 / 100.0, y as f32 / 100.0), &hasher);
                    assert!(v.value <= 1.0);
                }
            }
        }
    }
}
