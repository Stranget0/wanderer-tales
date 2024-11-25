use crate::prelude::*;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    render::{
        mesh::VertexAttributeValues,
        render_resource::{AsBindGroup, ShaderRef},
    },
};

enum Flags {
    DebugNormalsArrows,
    DebugNormalsShader,
}

impl DebugFlagsExt for Flags {
    fn group(&self) -> &'static str {
        "Normals"
    }

    fn as_str(&self) -> &'static str {
        match self {
            Flags::DebugNormalsArrows => "Debug normals arrows",
            Flags::DebugNormalsShader => "Debug normals shader",
        }
    }
}

pub(super) fn plugin(app: &mut bevy::prelude::App) {
    register_debug_flags(
        app,
        vec![Flags::DebugNormalsArrows, Flags::DebugNormalsShader],
    );

    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>,
    >::default())
        .add_systems(
            Update,
            (
                replace_standard_with_normal_shader
                    .run_if(debug_flag_enabled(&Flags::DebugNormalsShader)),
                replace_normal_with_standard_shader
                    .run_if(not(debug_flag_enabled(&Flags::DebugNormalsShader))),
                draw_debug_normals.run_if(debug_flag_enabled(&Flags::DebugNormalsArrows)),
            )
                .in_set(GameSet::PostUpdate),
        );
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct DebugNormalsMaterialExtension {}

impl MaterialExtension for DebugNormalsMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/fragment_debug_normals.wgsl".into()
    }
}

fn replace_standard_with_normal_shader(
    mut commands: Commands,
    mut debug_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>,
    >,
    materials: ResMut<Assets<StandardMaterial>>,
    query_with_standard_material: Query<(Entity, &Handle<StandardMaterial>)>,
) {
    // Replace all StandardMaterials with DebugNormalsMaterials
    for (entity, handle) in query_with_standard_material.iter() {
        let material = materials.get(handle).unwrap();
        let bundle = debug_materials.add(with_map_debug(material.clone()));
        commands.entity(entity).remove::<Handle<StandardMaterial>>();
        commands.entity(entity).insert(bundle);
    }
}

fn replace_normal_with_standard_shader(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    debug_materials: ResMut<
        Assets<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>,
    >,
    query_with_debug_material: Query<(
        Entity,
        &Handle<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>,
    )>,
) {
    // Replace all DebugNormalsMaterials with StandardMaterials
    for (entity, handle) in query_with_debug_material.iter() {
        let material = debug_materials.get(handle).unwrap();
        let bundle = materials.add(material.base.clone());
        commands
            .entity(entity)
            .remove::<Handle<ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>>>();
        commands.entity(entity).insert(bundle);
    }
}

fn draw_debug_normals(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    for (transform, mesh_handle) in query.iter() {
        let Some(mesh) = meshes.get(mesh_handle) else {
            warn!("No mesh found for handle {mesh_handle:?}, perhaps asset usage is not set to main world?");
            continue;
        };

        for normal in debug_normals_from_mesh(mesh) {
            gizmos.arrow(
                normal.0 + transform.translation,
                normal.1 + transform.translation,
                Color::srgb(
                    normal.1.x - normal.0.x,
                    normal.1.y - normal.0.y,
                    normal.1.z - normal.0.z,
                ),
            );
        }
    }
}

fn with_map_debug<T: Material>(base: T) -> ExtendedMaterial<T, DebugNormalsMaterialExtension> {
    ExtendedMaterial {
        extension: DebugNormalsMaterialExtension {},
        base,
    }
}

fn debug_normals_from_mesh(mesh: &Mesh) -> Vec<(Vec3, Vec3)> {
    let mut debug_normals = Vec::new();
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        if let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        {
            for (i, position) in positions.iter().enumerate() {
                let pos = Vec3::new(position[0], position[1], position[2]);
                let normal = Vec3::new(normals[i][0], normals[i][1], normals[i][2]);

                debug_normals.push((pos, pos + normal));
            }
        }
    }

    debug_normals
}
