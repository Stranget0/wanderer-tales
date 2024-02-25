use bevy::{
    math::vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_editor_pls::{
    editor_window::{EditorWindow, EditorWindowContext},
    egui,
    prelude::*,
};
use utils::map::{
    hex_map_item::{
        hex_vector::{HexVector, HEX_DIRECTIONS},
        HexMapItem, HexVectorRing, HexVectorSpiral,
    },
    layout_orientation::{FLAT_TOP_ORIENTATION, POINTY_TOP_ORIENTATION},
    HexLayout, PaintHex,
};

mod utils;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EditorPlugin::default()))
        .add_systems(Startup, setup)
        .run();
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
        let item = HexMapItem { pos: v };
        item.paint(&layout, &mut commands, &mut meshes, &mut materials);
    }
}
