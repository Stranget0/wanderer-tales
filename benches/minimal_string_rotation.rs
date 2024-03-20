use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use wanderer_tales::gameplay::map::utils::cycle::Cycle;

fn rotation_naive(cycles: &Vec<[i32; 6]>) {
    for cycle in cycles {
        Cycle::naive_minimal_rotation(cycle);
    }
}
fn rotation_shiloah(cycles: &Vec<[i32; 6]>) {
    for cycle in cycles {
        Cycle::shiloah_minimal_rotation(cycle);
    }
}

fn rotation_booth(cycles: &Vec<[i32; 6]>) {
    for cycle in cycles {
        Cycle::booth_minimal_rotation(cycle);
    }
}

fn lexicographically_minimal_rotation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Minimal Rotation");
    let input = (-3..3)
        .permutations(6)
        .map(|vec| {
            let arr: [i32; 6] = vec.try_into().unwrap();
            arr
        })
        .collect_vec();

    group.bench_with_input(
        BenchmarkId::new("naive", input.len()),
        &input,
        |b, cycles| b.iter(|| rotation_naive(cycles)),
    );
    group.bench_with_input(
        BenchmarkId::new("shiloah", input.len()),
        &input,
        |b, cycles| b.iter(|| rotation_shiloah(cycles)),
    );
    group.bench_with_input(
        BenchmarkId::new("booth", input.len()),
        &input,
        |b, cycles| b.iter(|| rotation_shiloah(cycles)),
    );
}

criterion_group!(benches, lexicographically_minimal_rotation);
criterion_main!(benches);
