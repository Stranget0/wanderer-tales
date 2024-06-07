use std::fmt::Debug;

use bevy::{
    math::*,
    pbr::{wireframe::WireframeConfig, ExtendedMaterial},
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        primitives::Aabb,
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};

use bevy::render::{
    mesh::{Indices, PrimitiveTopology},
    render_asset::RenderAssetUsages,
};

use bevy_easings::Lerp;
use bevy_editor_pls::EditorPlugin;
use bevy_flycam::{FlyCam, PlayerPlugin};
use itertools::Itertools;
use noisy_bevy::{simplex_noise_2d_seeded, NoisyShaderPlugin};
use wanderer_tales::{
    debug::fps_counter::FPSPlugin,
    render::map::*,
    shaders::plugin::MyShadersPlugin,
    utils::{
        WorldAlignedExtension, WorldDisplacementExtension, CHUNK_ITEM_COUNT, CHUNK_SLICES,
        QUAD_TREE_DIRECTIONS,
    },
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            NoisyShaderPlugin,
            EditorPlugin::default(),
            MyShadersPlugin,
            PlayerPlugin,
            FPSPlugin,
        ))
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: false,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Color::YELLOW_GREEN,
        })
        .insert_resource(LODSetter::new(1500, 255, 10))
        .insert_resource(MapLOD::default())
        .insert_resource(LastChunkRenderPos::default())
        .register_type::<LODSetter>()
        .add_event::<LODTreeCreated>()
        .add_plugins((
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, WorldAlignedExtension>>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, WorldDisplacementExtension>>::default(),
            MaterialPlugin::<ExtendedMaterial<ExtendedMaterial<StandardMaterial, WorldDisplacementExtension>, WorldAlignedExtension>>::default(),
            MaterialPlugin::<ExtendedMaterial<ExtendedMaterial<StandardMaterial, WorldAlignedExtension>, WorldDisplacementExtension>>::default(),
        ))
        .add_systems(Startup, (
					update_lod_tree,
					// spawn_lights,
					spawn_primitives
				))
        .add_systems(
            Update,
            (
                draw_gizmos,
                update_lod_tree,
                render_lod_chunks.run_if(|e:EventReader<LODTreeCreated>, last_chunk_render_pos: Option<Res<LastChunkRenderPos>>, player_pos: Query<&Transform, With<FlyCam>>, map_lod: Res<MapLOD>| !e.is_empty() ||  match player_pos.get_single(){
                    Ok(player_pos) => {
                        match last_chunk_render_pos {
                            Some(last_pos) => player_pos.translation.xz().distance(last_pos.0) > map_lod.min_size,
                            None => false,
                        }
                    },
                    Err(_) => false,
                }).after(update_lod_tree),
                map_chunks_aabb,
            ),
        )
        .run();
}

#[derive(Debug, Component)]
struct MapChunkParent;

#[derive(Debug, Event, Default)]
struct LODTreeCreated;

#[derive(Debug, Resource, Default)]
struct LastChunkRenderPos(pub Vec2);

fn draw_gizmos(mut gizmos: Gizmos, lod: Res<LODSetter>) {
    let factor = (lod.view_distance / 2) as f32;
    gizmos.arrow(Vec3::ZERO, Vec3::X * factor, Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y * factor, Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z * factor, Color::BLUE);

    for x in 0..20 {
        for y in 0..20 {
            let x_f32 = x as f32 * 10.0;
            let y_f32 = y as f32 * 10.0;
            gizmos.sphere(
                vec3(x_f32, terrain_noise(&vec2(x_f32, y_f32)), y_f32),
                Quat::default(),
                0.1,
                Color::YELLOW,
            );
        }
    }
}

fn update_lod_tree(
    mut map_tree: ResMut<MapLOD>,
    chunk_setter: Res<LODSetter>,
    mut lod_created: EventWriter<LODTreeCreated>,
) {
    if !chunk_setter.is_added() && !chunk_setter.is_changed() {
        return;
    }

    *map_tree = MapLOD::from_setter(&chunk_setter);

    if chunk_setter.is_changed() {
        lod_created.send_default();
    }
}

fn render_lod_chunks(
    mut commands: Commands,
    mut map_tree: ResMut<MapLOD>,
    player_pos: Query<&Transform, With<FlyCam>>,
    asset_server: Res<AssetServer>,
    prev_parents: Query<Entity, With<MapChunkParent>>,
    last_chunk_render_pos: Option<ResMut<LastChunkRenderPos>>,
) {
    for p in prev_parents.iter() {
        commands.entity(p).despawn_recursive();
    }

    let offset_float = player_pos.single().translation.xz();
    let offset = offset_float - offset_float % map_tree.min_size;
    match last_chunk_render_pos {
        Some(mut pos) => {
            pos.0 = offset;
        }
        None => commands.insert_resource(LastChunkRenderPos(offset)),
    }

    let material = asset_server.add(ExtendedMaterial {
        base: ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/grass.jpg")),
                normal_map_texture: Some(asset_server.load("textures/grass_normal.jpg")),
                ..default()
            },
            extension: WorldDisplacementExtension::new(),
        },
        extension: WorldAlignedExtension::new(0.1),
    });

    commands
        .spawn((
            MapChunkParent,
            Name::new("MapChunks"),
            SpatialBundle::from_transform(Transform::from_xyz(offset.x, 0.0, offset.y)),
        ))
        .with_children(|builder| {
            for chunk in map_tree.leafs.iter_mut() {
                let transform = Transform::from_xyz(chunk.pos.x, 0.0, chunk.pos.y);

                let mesh = create_subdivided_plane(chunk.size, CHUNK_SLICES, |pos| {
                    Vec3::new(pos.x, 0.0, pos.y)
                });

                let render_id = builder
                    .spawn((
                        Name::new(format!("Chunk-{}", chunk.precision)),
                        MapChunk,
                        MaterialMeshBundle {
                            mesh: asset_server.add(mesh),
                            material: material.clone(),
                            transform,
                            ..default()
                        },
                    ))
                    .id();

                chunk.entity = Some(render_id);
            }
        });
}

fn map_chunks_aabb(
    mut chunks: Query<(Entity, &mut Aabb, &GlobalTransform), With<MapChunk>>,
    chunk_tree: Res<MapLOD>,
) {
    for chunk in chunk_tree.leafs.iter() {
        if let Some((entity, mut aabb, chunk_transform)) =
            chunk.entity.and_then(|entity| chunks.get_mut(entity).ok())
        {
            let chunk_pos = chunk_transform.translation().xz();
            let points = QUAD_TREE_DIRECTIONS
                .iter()
                .map(|direction| {
                    let vec2 = *direction * chunk.size;
                    Vec3::new(vec2.x, terrain_noise(&(vec2 + chunk_pos)), vec2.y)
                })
                .collect_vec();

            *aabb = Aabb::enclosing(points).unwrap();
        }
    }
}

fn create_subdivided_plane(size: f32, slices: usize, f_3d: impl Fn(Vec2) -> Vec3) -> Mesh {
    let total = (slices - 1) as f32;
    let capacity = slices * 6;
    let mut vertices = Vec::with_capacity(capacity);
    let mut uvs = Vec::with_capacity(capacity);
    let mut normals = Vec::with_capacity(capacity);
    let mut indices = Vec::with_capacity(capacity);

    for i in 0..slices {
        for j in 0..slices {
            let u = j as f32 / total - 0.5;
            let v = i as f32 / total - 0.5;

            let x = u * size;
            let z = v * size;

            let pos = f_3d(vec2(x, z));

            vertices.push(pos);
            uvs.push([u, v]);
            normals.push([0.0, 1.0, 0.0]);
        }
    }

    let total_u = total as u16;
    for i in 0..total_u {
        for j in 0..total_u {
            let index_1 = j + i * (total_u + 1);
            let index_2 = index_1 + 1;
            let index_3 = index_1 + total_u + 2;
            let index_4 = index_1 + total_u + 1;

            // Triangle 1
            indices.push(index_3);
            indices.push(index_2);
            indices.push(index_1);

            // Triangle 2
            indices.push(index_4);
            indices.push(index_3);
            indices.push(index_1);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U16(indices))
}

// Reflect in assets\shaders\height_map.wgsl
pub fn terrain_noise(pos: &Vec2) -> f32 {
    simplex_noise_2d_seeded(*pos / 100.0, 1.0) * 10.0
}

pub fn spawn_primitives(mut commands: Commands, asset_server: Res<AssetServer>) {
    let material = asset_server.add(ExtendedMaterial {
        base: ExtendedMaterial {
            base: StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/grass.jpg")),
                normal_map_texture: Some(asset_server.load("textures/grass_normal.jpg")),
                ..default()
            },
            extension: WorldDisplacementExtension::new(),
        },
        extension: WorldAlignedExtension::new(0.1),
    });

    let shapes = vec![Mesh::from(Cuboid::default()), Mesh::from(Sphere::default())];

    for (i, mesh) in shapes
        .iter()
        .map(|m| asset_server.add(m.clone()))
        .enumerate()
    {
        commands.spawn(MaterialMeshBundle {
            mesh,
            material: material.clone(),
            transform: Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            ..default()
        });
    }
}

pub fn spawn_lights(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::hex("#FFEEE3").unwrap(),
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::PI / 4.,
            0.0,
            -std::f32::consts::PI / 4.,
        )),
        ..default()
    });
}
