use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, math::vec2, prelude::*, utils::hashbrown::HashMap};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wanderer_tales::gameplay::{
    map::{
        components::SourceLayout,
        data_source_layer::{
            resources::{HexToMapSourceEntity, SeedTable},
            systems::spawn_map_data,
            MapAddEvent, MapSubEvent,
        },
        utils::{
            hex_layout::HexLayout, hex_vector::FractionalHexVector,
            layout_orientation::POINTY_TOP_ORIENTATION,
        },
    },
    player::{
        components::{HexPositionFractional, Sight},
        events::{CharacterMovedEvent, PlayerWithSightSpawnedEvent},
    },
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

    app.add_event::<PlayerWithSightSpawnedEvent>()
        .add_event::<MapAddEvent>()
        .add_event::<MapSubEvent>()
        .add_event::<CharacterMovedEvent>()
        .insert_resource(SeedTable::default())
        .insert_resource(HexToMapSourceEntity::default());

    app.add_systems(Update, spawn_map_data);

    let layout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(5.0, 5.0),
        origin: vec2(0.0, 0.0),
    };

    app.world.spawn((layout, SourceLayout));

    app.update();

    app.world.send_event(PlayerWithSightSpawnedEvent {
        pos: HexPositionFractional(FractionalHexVector(0.0, 0.0, 0.0)),
        sight: Sight(sight),
    });

    app.update();
}
