use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin,
    prelude::*,
    utils::{hashbrown::HashMap, HashMap},
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wanderer_tales::gameplay::map::{
    components::MapContent,
    spawner::{resources::MapData, systems::spawn_map_data, MapAddEvent, MoveSightEvent},
    utils::hex_layout::HexLayout,
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
    let layout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(5.0, 5.0),
        origin: vec2(0.0, 0.0),
    };
    let map_data = MapData {
        hex_to_data_entity: HashMap::new(),
    };
    app.add_event::<MoveSightEvent>()
        .add_event::<MapAddEvent>()
        .insert_resource(layout)
        .insert_resource(map_data);

    let map_display = app.world.spawn(MapDisplay).id();
    app.world.spawn(MapContent).add_child(map_display);

    app.add_systems(Update, spawn_map_data);
    app.update();

    app.world.send_event(MoveSightEvent {
        pos: Vec2::new(0.0, 0.0),
        sight,
        force_render: true,
        map_display,
        ..default()
    });

    app.update();
}
