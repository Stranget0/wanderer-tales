use bevy::math::vec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wanderer_tales::game::map::TerrainSampler;

pub fn terrain_chunk_sample(c: &mut Criterion) {
    let terrain = TerrainSampler::default();
    c.bench_function("terrain chunk sampler", |b| {
        b.iter(|| {
            let chunk_sampler = terrain.chunk_sampler(black_box(vec2(1.1, 2.2)));
            chunk_sampler(black_box(10.0), black_box(20.0));
        })
    });
}

criterion_group!(benches, terrain_chunk_sample);
criterion_main!(benches);
