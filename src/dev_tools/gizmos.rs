use crate::game::CameraOrbitTarget;
use crate::prelude::*;

enum Flags {
    WorldGizmo,
    LocalGizmo,
}

impl DebugFlagsExt for Flags {
    fn group(&self) -> &'static str {
        "Gizmos"
    }
    fn as_str(&self) -> &'static str {
        match self {
            Flags::WorldGizmo => "World gizmo",
            Flags::LocalGizmo => "Local gizmo",
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    register_debug_flags(app, vec![Flags::WorldGizmo, Flags::LocalGizmo]);

    app.add_systems(
        Update,
        add_world_gizmo
            .run_if(debug_flag_enabled(&Flags::WorldGizmo))
            .in_set(GameSet::PostUpdate),
    );
}

fn add_world_gizmo(mut gizmos: Gizmos, query: Query<&Transform, With<CameraOrbitTarget>>) {
    let start = query
        .get_single()
        .map(|t| t.translation)
        .unwrap_or_default();

    let g = vec![
        (Vec3::X, Srgba::RED),
        (Vec3::Y, Srgba::GREEN),
        (Vec3::Z, Srgba::BLUE),
    ];
    for (dir, color) in g {
        gizmos.arrow(start, start + dir, color.with_alpha(0.3));
    }
}
