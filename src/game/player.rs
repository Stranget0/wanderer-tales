use super::camera::*;
use super::map::ChunkOrigin;
use super::map::Terrain;
use super::movement::*;
use super::prelude::*;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), spawn_player);
}

pub fn spawn_player(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    terrain: Res<Terrain>,
) {
    commands.spawn((
        Name::new("Player"),
        StateScoped(Screen::Playing),
        MovementSpeed(1000.0),
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
                terrain.sample(Vec2::default()).value,
                0.0,
            )),
            ..Default::default()
        },
    ));
}
