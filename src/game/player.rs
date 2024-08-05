use super::prelude::*;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), spawn_player);
}

pub fn spawn_player(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        StateScoped(Screen::Playing),
        Movement { speed: 5.0 },
        MovementController::default(),
        CameraTarget { zoom: 5.0 },
        PbrBundle {
            mesh: asset_server.add(Cuboid::new(1.0, 1.0, 1.0).into()),
            material: asset_server.add(Color::srgb(0.5, 0.5, 0.5).into()),
            ..Default::default()
        },
    ));
}
