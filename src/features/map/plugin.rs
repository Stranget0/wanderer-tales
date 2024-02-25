use bevy::{math::vec2, prelude::*};

use crate::global_state::SceneState;

use super::{
    hex_layout::HexLayout,
    hex_vector::{iterators::HexVectorSpiral, HexVector},
    layout_orientation::POINTY_TOP_ORIENTATION,
};

pub struct MapPlugin;

#[derive(Resource)]
struct MapData {
    layout_entity: Entity,
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>().add_systems(
            OnEnter(SceneState::Menu),
            setup.run_if(in_state(SceneState::Menu)),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());
    let layout = HexLayout {
        orientation: POINTY_TOP_ORIENTATION,
        size: vec2(32.0, 32.0),
        origin: vec2(0.0, 0.0),
    };

    let origin_hex = HexVector(0, 0, 0);

    for v in HexVectorSpiral::new(&origin_hex, 3) {

        // let item = HexMapItem { pos: v };
        // item.paint(&layout, &mut commands, &mut meshes, &mut materials);
    }
}
