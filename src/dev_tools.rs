//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};

use crate::{game::CameraObserver, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(
        Update,
        (add_forward_gizmo, add_world_gizmo, add_camera_debug),
    );
}

fn add_world_gizmo(mut gizmos: Gizmos) {
    let start = Vec3::ZERO;
    let g = vec![
        (Vec3::X, Srgba::RED),
        (Vec3::Y, Srgba::GREEN),
        (Vec3::Z, Srgba::BLUE),
    ];
    for (dir, color) in g {
        gizmos.arrow(start, start + dir, color.with_alpha(0.3));
    }
}

fn add_forward_gizmo(mut gizmos: Gizmos, query: Query<(&Transform, &Visibility)>) {
    for (transform, visibility) in query.iter() {
        if visibility == Visibility::Visible {
            continue;
        }

        add_directions_gizmos(&mut gizmos, transform.translation, transform.rotation);
    }
}

fn add_camera_debug(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera debug"),
        CameraObserver,
        SpatialBundle::default(),
    ));
}

fn add_directions_gizmos(gizmos: &mut Gizmos, start: Vec3, rotation: Quat) {
    let g = vec![
        (Vec3::X, Srgba::RED),
        (Vec3::Y, Srgba::GREEN),
        (Vec3::Z, Srgba::BLUE),
    ];
    for (dir, color) in g {
        gizmos.arrow(start, start + (rotation * dir), color.with_alpha(0.3));
    }
}
