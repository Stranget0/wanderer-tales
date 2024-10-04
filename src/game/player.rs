use crate::prelude::*;
use crate::utils::noise::PcgHasher;

use super::camera::*;
use super::map::map_generator;
use super::map::ChunkOrigin;
use super::movement::*;
use super::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), spawn_player);
}

pub fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Playing),
        MovementSpeed(100.0),
        RotationSpeed(10.0),
        MovementController::default(),
        RotationController::default(),
        CameraOrbitTarget { zoom: 5.0 },
        ChunkOrigin,
        PbrBundle {
            mesh: asset_server.add(Cuboid::new(1.0, 1.0, 1.0).into()),
            material: asset_server.add(Color::srgb(0.5, 0.5, 0.5).into()),
            transform: Transform::from_translation(vec3(
                0.0,
                map_generator(vec2(0.0, 0.0), &PcgHasher::new(0))(0.0, 0.0).0,
                0.0,
            )),
            ..Default::default()
        },
    ));
}
