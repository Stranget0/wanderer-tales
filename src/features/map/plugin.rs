use bevy::{math::vec2, prelude::*, utils::HashMap};
use rand::Rng;

use crate::global_state::SceneState;

use super::{
    hex_layout::HexLayout,
    hex_map_item::{Biome, Height, HexMapItemBundle},
    hex_vector::{iterators::HexVectorSpiral, HexVector},
    layout_orientation::POINTY_TOP_ORIENTATION,
    renderer::rendered_2d::render_map,
};

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_systems(
                // OnEnter(SceneState::Menu),
                Startup,
                ((spawn_layout, spawn_map_data, render_map).chain(),),
            )
            .add_systems(OnExit(SceneState::Menu), despawn_map_data);
    }
}
pub struct MapPlugin;

#[derive(Resource)]
struct MapHexData(HashMap<HexVector, HexMapItemBundle>);

fn spawn_layout(mut commands: Commands) {
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

fn spawn_map_data(mut commands: Commands, layout: Query<Entity, With<HexLayout>>) {
    let origin_hex = HexVector(0, 0, 0);
    commands.entity(layout.single()).with_children(|parent| {
        for v in HexVectorSpiral::new(&origin_hex, 3) {
            let bundle = HexMapItemBundle {
                biome: get_biome(&v),
                height: get_height(&v),
                pos: v,
            };

            parent.spawn(bundle);
        }
    });
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

fn despawn_map_data(mut commands: Commands, layout: Query<Entity, With<HexLayout>>) {
    commands.entity(layout.single()).despawn_recursive();
}
