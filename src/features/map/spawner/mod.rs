use bevy::{
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::vec2,
    prelude::SpatialBundle,
};
use rand::Rng;

use super::utils::{
    hex_layout::HexLayout,
    hex_map_item::{Biome, Height, HexMapItemBundle},
    hex_vector::{iterators::HexVectorSpiral, HexVector},
    layout_orientation::POINTY_TOP_ORIENTATION,
};

pub fn spawn_layout(mut commands: Commands) {
    let layout: HexLayout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(32.0, 32.0),
        origin: vec2(0.0, 0.0),
    };

    commands.spawn((
        layout,
        SpatialBundle {
            ..Default::default()
        },
    ));
}

pub fn spawn_map_data(mut commands: Commands, layout: Query<Entity, With<HexLayout>>) {
    let origin_hex = HexVector(0, 0, 0);
    let layout_entity = layout.single();
    for v in HexVectorSpiral::new(&origin_hex, 3) {
        let bundle = HexMapItemBundle {
            biome: get_biome(&v),
            height: get_height(&v),
            pos: v,
        };
        let hex_entity = commands.spawn(bundle).id();
        let mut layout_controls = commands.entity(layout_entity);
        layout_controls.add_child(hex_entity);
    }
}

pub fn despawn_map_data(mut commands: Commands, layout: Query<Entity, With<HexLayout>>) {
    let controls = commands.entity(layout.single());
    controls.despawn_recursive();
}

fn get_height(_hex: &HexVector) -> Height {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    Height((x * 50.0) as u8)
}

fn get_biome(_hex: &HexVector) -> Biome {
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen();
    match x.round() as u8 {
        0 => Biome::Forest,
        _ => Biome::Grass,
    }
}
