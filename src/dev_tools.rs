//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::render_resource::ShaderImport,
};

use crate::{game::CameraOrbit, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(WireframePlugin);

    app.insert_resource(WireframeConfig {
        default_color: Color::srgb(1.0, 1.0, 1.0),
        ..Default::default()
    });

    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(
        Update,
        (
            add_forward_gizmo,
            add_world_gizmo,
            add_camera_debug,
            log_shader_load,
        ),
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
        CameraOrbit,
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

fn log_shader_load(mut asset_event: EventReader<AssetEvent<Shader>>, shaders: Res<Assets<Shader>>) {
    for event in asset_event.read() {
        let e = match event {
            AssetEvent::Added { id } => {
                let shader = shaders.get(*id).unwrap();
                Some(("Added", shader))
            }
            AssetEvent::Unused { id } => {
                let shader = shaders.get(*id).unwrap();
                Some(("Unused", shader))
            }
            AssetEvent::Removed { id } => {
                let shader = shaders.get(*id).unwrap();
                Some(("Reoved", shader))
            }
            _ => None,
        };
        let Some((action, path, import_path)) = e.and_then(|e| {
            let ShaderImport::Custom(import_path) = e.1.import_path() else {
                return None;
            };
            if import_path.contains("wanderer_tales") {
                return Some((e.0, &e.1.path, import_path));
            }
            None
        }) else {
            continue;
        };

        info!("{} shader {} => {}", action, path, import_path);
    }
}
