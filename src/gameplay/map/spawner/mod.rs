use bevy::{
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    math::vec2,
    prelude::SpatialBundle,
    prelude::*,
};
use rand::Rng;

use super::{
    events::{MapAddEvent, MoveMapOriginEvent},
    utils::{
        hex_layout::HexLayout,
        hex_map_item::{Biome, Height, HexMapItemBundle},
        hex_vector::{iterators::HexVectorSpiral, HexVector},
        layout_orientation::POINTY_TOP_ORIENTATION,
    },
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

pub fn spawn_map_data(
    mut commands: Commands,
    layout_query: Query<Entity>,
    hexes_query: Query<&HexVector, With<Biome>>,
    mut origin_event: EventReader<MoveMapOriginEvent>,
    mut render_event: EventWriter<MapAddEvent>,
) {
    let layout_res = layout_query.get_single();
    match layout_res {
        Err(err) => {
            error!("{}", err);
        }
        Ok(layout_entity) => {
            for move_map_event in origin_event.read() {
                let current_hexes: Vec<&HexVector> = hexes_query.iter().collect();
                let mut hexes: Vec<Entity> = vec![];
                let new_origin = &move_map_event.0;
                for v in HexVectorSpiral::new(new_origin, 3, 0) {
                    if current_hexes.contains(&&v) {
                        continue;
                    };
                    let bundle = HexMapItemBundle {
                        biome: get_biome(&v),
                        height: get_height(&v),
                        pos: v,
                    };
                    let hex_entity = commands.spawn(bundle.clone()).id();
                    let mut layout_controls = commands.entity(layout_entity);
                    layout_controls.add_child(hex_entity);
                    hexes.push(hex_entity);
                }
                render_event.send(MapAddEvent(hexes));
            }
        }
    };
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
