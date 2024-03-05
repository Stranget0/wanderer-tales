use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*, utils::HashMap};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wanderer_tales::gameplay::map::utils::hex_vector::{iterators::HexVectorSpiral, HexVector};

criterion_group!(benches, map_bench);
criterion_main!(benches);

pub fn map_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("hex spiral forward");

    for sight in [50, 200, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(sight), &sight, |b, sight| {
            b.iter(|| hex_spiral_bench(*sight))
        });
    }
}

fn hex_spiral_bench(sight: u16) {
    let origin = HexVector::new(0, 0, 0);
    let iterator = HexVectorSpiral::new(&origin, 0, sight);
    for e in iterator {}
}
