//! Development tools for the game. This plugin is only enabled in dev builds.
pub mod data;

pub use data::*;

#[cfg(feature = "dev")]
pub(super) use plugin::plugin;

#[cfg(feature = "dev")]
pub(super) mod plugin {
    use super::*;
    use crate::{game, prelude::*};
    use crate::{game::CameraOrbit, screen::Screen};
    use bevy::pbr::ExtendedMaterial;
    use bevy::{
        color::palettes::tailwind,
        dev_tools::states::log_transitions,
        pbr::{
            wireframe::{WireframeConfig, WireframePlugin},
            MaterialExtension,
        },
        prelude::*,
        render::render_resource::{AsBindGroup, ShaderImport, ShaderRef},
    };

    pub fn plugin(app: &mut App) {
        app.add_plugins((
            WireframePlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>::default(),
            game::devtools::map_devtools_plugin
        ));

        app.insert_resource(WireframeConfig {
            default_color: Color::srgb(1.0, 1.0, 1.0),
            ..Default::default()
        });

        // Print state transitions in dev builds
        app.add_systems(Update, log_transitions::<Screen>);
        app.add_systems(
            Update,
            (
                // add_forward_gizmo,
                // add_world_gizmo,
                add_camera_debug,
                log_shader_load,
                // draw_debug_normals,
                game::devtools::change_map_seed.run_if(input_just_pressed(KeyCode::Numpad0)),
                toggle_debug_normals.run_if(input_just_pressed(KeyCode::Numpad1)),
                game::devtools::toggle_debug_chunks.run_if(input_just_pressed(KeyCode::Numpad2)),
            ),
        );
    }

    #[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
    struct DebugNormalsMaterialExtension {}

    impl MaterialExtension for DebugNormalsMaterialExtension {
        fn fragment_shader() -> ShaderRef {
            "shaders/fragment_debug_normals.wgsl".into()
        }
    }

    fn toggle_debug_normals(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut debug_materials: ResMut<
            Assets<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>,
        >,
        standard_query: Query<(Entity, &Handle<StandardMaterial>)>,
        debug_query: Query<(
            Entity,
            &Handle<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>,
        )>,
    ) {
        for (entity, handle) in standard_query.iter() {
            let material = materials.get(handle).unwrap();
            let bundle = debug_materials.add(with_map_debug(material.clone()));
            commands.entity(entity).remove::<Handle<StandardMaterial>>();
            commands.entity(entity).insert(bundle);
        }

        if !standard_query.is_empty() {
            return;
        }

        for (entity, handle) in debug_query.iter() {
            let material = debug_materials.get(handle).unwrap();
            let bundle = materials.add(material.base.clone());
            commands
            .entity(entity)
            .remove::<Handle<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>>();
            commands.entity(entity).insert(bundle);
        }
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

    fn log_shader_load(
        mut asset_event: EventReader<AssetEvent<Shader>>,
        shaders: Res<Assets<Shader>>,
    ) {
        for event in asset_event.read() {
            let e = match event {
                AssetEvent::Added { id } => shaders.get(*id).map(|s| ("Added", s)),
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

    fn draw_debug_normals(mut gizmos: Gizmos, query: Query<&DebugNormals>) {
        for normals in query.iter() {
            for normal in &normals.0 {
                gizmos.line(normal.0, normal.1, tailwind::RED_400);
            }
        }
    }

    fn with_map_debug<T: Material>(base: T) -> ExtendedMaterial<T, DebugNormalsMaterialExtension> {
        ExtendedMaterial {
            extension: DebugNormalsMaterialExtension {},
            base,
        }
    }
}
