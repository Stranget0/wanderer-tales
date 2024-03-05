use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*, utils::HashMap};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wanderer_tales::gameplay::map::{
    events::{MapAddEvent, MoveSightEvent},
    renderer::renderer_2d::render_map,
    spawner::{spawn_layout, spawn_map_data, MapData},
};

criterion_group!(benches, map_bench);
criterion_main!(benches);

pub fn map_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("initial map render");

    for sight in [10, 20, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(sight), &sight, |b, sight| {
            b.iter(|| map_init_render(*sight))
        });
    }
}

fn map_init_render(sight: u16) {
    let mut app = App::new();

    app.add_event::<MoveSightEvent>()
        .add_event::<MapAddEvent>()
        .insert_resource(MapData {
            hex_to_entity: HashMap::new(),
        })
        .add_systems(Startup, spawn_layout)
        .add_systems(Update, spawn_map_data);

    app.update();

    app.world.send_event(MoveSightEvent {
        pos: Vec2::new(0.0, 0.0),
        sight,
        force_render: true,
        ..default()
    });

    app.update();
}
