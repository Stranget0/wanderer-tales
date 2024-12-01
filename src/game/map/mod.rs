// #[cfg(feature = "dev")]
// pub(crate) mod devtools;

use std::iter;

use crate::prelude::*;
use avian3d::prelude::{Collider, CollidingEntities, CollisionLayers};
use bevy::{ecs::system::SystemParam, render::render_asset::RenderAssetUsages};
use camera::GameplayCamera;
use utils::noise::{self, SimpleHasher};

use super::physics::CollisionLayersExt;

const CHUNK_SUBDIVISIONS: u32 = 4;

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
struct Chunk {
    subdivisions: u32,
    size: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
struct ChunksContainer;

impl Chunk {
    fn new(subdivisions: u32, size: f32) -> Self {
        Self { subdivisions, size }
    }

    fn contains_point(&self, chunk_transform: &Transform, point: Vec2) -> bool {
        let size_half = Vec2::splat(self.size / 2.0);
        let min = chunk_transform.translation.xz() - size_half;
        let max = chunk_transform.translation.xz() + size_half;

        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    fn contains_chunk(&self, chunk_transform: &Transform, other_transform: &Transform) -> bool {
        self.contains_point(chunk_transform, other_transform.translation.xz())
    }
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct TerrainSampler {
    weights: Vec<TerrainWeight>,
}

#[derive(Resource, Reflect, Clone, Copy)]
#[reflect(Resource)]
pub struct BaseSeed(u32);

#[derive(Resource, Reflect, Clone, Copy)]
#[reflect(Resource)]
struct DesiredSurfaceArea(f32);

impl Default for TerrainSampler {
    fn default() -> Self {
        Self {
            weights: TerrainWeight::from_vec(vec![
                (2000.0, 1000.0, 0.2),
                (1000.0, 1000.0, 0.9),
                (500.0, 500.0, 0.25),
            ]),
        }
    }
}

impl TerrainSampler {
    pub fn sample(&self, pos: Vec2, base_seed: &BaseSeed) -> noise::Value2Dt1 {
        TerrainWeight::sample_many(pos, base_seed, self.weights.iter()).0
    }
    pub fn max_height(&self) -> f32 {
        self.weights.iter().map(|w| w.amplitude).sum()
    }
}

#[derive(Debug, Reflect, Clone, Copy)]
pub struct TerrainWeight {
    size: f32,
    amplitude: f32,
    erosion: f32,
    seed_offset: u32,
}

impl TerrainWeight {
    pub fn new(size: f32, amplitude: f32, erosion: f32, seed_offset: u32) -> Self {
        Self {
            size,
            amplitude,
            erosion,
            seed_offset,
        }
    }

    pub fn from_vec(weights: Vec<(f32, f32, f32)>) -> Vec<Self> {
        weights
            .into_iter()
            .enumerate()
            .map(|(seed_offset, (size, amplitude, erosion))| {
                Self::new(size, amplitude, erosion, seed_offset as u32)
            })
            .collect_vec()
    }

    pub fn sample(&self, pos: Vec2, base_seed: &BaseSeed) -> noise::Value2Dt2 {
        let hasher = SimpleHasher::new(self.seed_offset + base_seed.0);

        (utils::noise::perlin_noise_2d(pos, 1.0 / self.size, &hasher) / 2.0 + 0.5) * self.amplitude
    }

    // fn sample_erosion_base(
    //     &self,
    //     hasher: &impl noise::NoiseHasher,
    //     pos: Vec2,
    //     // t(x,y)
    //     erosion_factor: noise::Value2Dt1,
    // ) -> (noise::Value2Dt1, noise::Value2Dt1) {
    //     // f(x,y)
    //     let layer = self.sample(hasher, pos);
    //     // s(x,y)
    //     let layer_steepiness = layer.dt_length();
    //
    //     // d(x,y)
    //     let pre_erosion_factor = erosion_factor + layer_steepiness;
    //     // v(x,y)
    //     let factor = 1.0 + self.erosion * pre_erosion_factor;
    //     // p(x,y)
    //     let eroded_layer = layer.to_dt1() / factor;
    //
    //     // if you dont have enough derivatives it might be possible to calculate them
    //     let eroded_layer_steepiness = Value2Dt1::new(
    //         eroded_layer.dt_length(),
    //         layer.d1.0 / factor.value
    //             - self.erosion
    //                 * layer.value
    //                 * (2.0 * layer.d1.0.y * layer.d2.zy() + 2.0 * layer.d1.0.x * layer.d2.xz())
    //                 / (2.0 * layer_steepiness.value * factor.value.powi(2)),
    //     );
    //     (eroded_layer, eroded_layer_steepiness)
    // }

    fn sample_erosion_base(
        &self,
        pos: Vec2,
        erosion_factor: f32,
        base_seed: &BaseSeed,
    ) -> (noise::Value2Dt1, f32) {
        let layer = self.sample(pos, base_seed);
        let layer_steepiness = layer.dt_length();

        let pre_erosion_factor = erosion_factor + layer_steepiness;
        let v = 1.0 + self.erosion * pre_erosion_factor;
        (layer.to_dt1() / v, (layer.to_dt1() / v.value).dt_length())
    }

    pub fn sample_many<'a>(
        pos: Vec2,
        base_seed: &BaseSeed,
        weights: impl Iterator<Item = &'a Self>,
    ) -> (noise::Value2Dt1, f32) {
        let mut erosion_factor = 0.0;
        let mut terrain = noise::Value2Dt1::default();

        for w in weights {
            let (layer, layer_steepiness) = w.sample_erosion_base(pos, erosion_factor, base_seed);
            terrain = terrain + layer;
            erosion_factor = layer_steepiness;
        }

        (terrain, erosion_factor)
    }
}

// #[derive(Resource, Debug, Reflect)]
// #[reflect(Resource)]
// pub struct TerrainSampler {
//     // (size, amplitude, erosion)
//     pub noise_weights: Vec<TerrainWeight>,
//     pub noise_seed: u32,
//     pub chunk_subdivisions: u32,
//     // These are in chunks
//     pub chunk_spawn_radius: u8,
//     pub chunk_visibility_radius: u8,
// }

// #[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
// enum ChunkSystemSet {
//     Mutate,
//     Register,
//     Render,
//     Colliders,
//
//     ChunkReload,
// }

pub fn plugin(app: &mut App) {
    app.insert_resource(BaseSeed(0))
        .init_resource::<TerrainSampler>()
        .insert_resource(DesiredSurfaceArea(1.0))
        .add_systems(OnEnter(GameState::Playing), setup_chunk_container)
        .add_systems(
            Update,
            (
                spawn_chunks.in_set(GameSet::RecordInput),
                (render_chunks, add_colliders_to_chunks)
                    .chain()
                    .in_set(GameSet::UpdateDataLayer),
                remove_colliders_from_chunks.in_set(GameSet::UpdateDataLayer),
            )
                .run_if(in_state(GameState::Playing)),
        );

    // app.configure_sets(
    //     OnEnter(GameState::Playing),
    //     (
    //         ChunkSystemSet::Mutate,
    //         ChunkSystemSet::Register,
    //         ChunkSystemSet::Render,
    //         ChunkSystemSet::Colliders.in_set(GameSet::UpdateDataLayer),
    //     )
    //         .chain(),
    // )
    // .configure_sets(
    //     Update,
    //     (
    //         #[cfg(feature = "dev")]
    //         ChunkSystemSet::ChunkReload,
    //         ChunkSystemSet::Mutate,
    //         ChunkSystemSet::Register,
    //         ChunkSystemSet::Render,
    //         ChunkSystemSet::Colliders.in_set(GameSet::UpdateDataLayer),
    //     )
    //         .chain(),
    // )
    // .add_systems(
    //     OnEnter(GameState::Playing),
    //     (
    //         spawn_chunks.in_set(ChunkSystemSet::Mutate),
    //         render_chunks.in_set(ChunkSystemSet::Render),
    //         register_chunks.in_set(ChunkSystemSet::Register),
    //         add_colliders_to_chunks.in_set(ChunkSystemSet::Colliders),
    //     ),
    // )
    // .add_systems(
    //     Update,
    //     (
    //         // update_map_render_center.before(ChunkSystemSet::Mutate),
    //         // register_chunks.in_set(ChunkSystemSet::Register),
    //         // (
    //         //     spawn_chunks.in_set(ChunkSystemSet::Mutate),
    //         //     despawn_unregister_out_of_range_chunks.in_set(ChunkSystemSet::Mutate),
    //         //     render_chunks.in_set(ChunkSystemSet::Render),
    //         //     derender_chunks.in_set(ChunkSystemSet::Render),
    //         //     add_colliders_to_chunks.in_set(ChunkSystemSet::Colliders),
    //         //     remove_colliders_from_chunks.in_set(ChunkSystemSet::Colliders),
    //         // )
    //         //     .run_if(render_center_changed),
    //     ),
    // );
}

// #[derive(SystemParam)]
// struct TerrainCollidables<'w, 's> {
//     collidables_query: Query<
//         'w,
//         's,
//         (&'static Transform, &'static CollisionLayers),
//         (With<Collider>, Without<Chunk>),
//     >,
// }
//
// impl<'w, 's> TerrainCollidables<'w, 's> {
//     fn iter(&self) -> impl Iterator<Item = Vec2> + '_ {
//         let chunk_collision_layers = CollisionLayers::get_terrain_colliders();
//         self.collidables_query
//             .iter()
//             .filter_map(move |(transform, collision_layers)| {
//                 collision_layers
//                     .interacts_with(chunk_collision_layers)
//                     .then_some(transform.translation.xz())
//             })
//     }
// }

// fn update_map_render_center(
//     query: Query<&Transform, With<ChunkOrigin>>,
//     mut map_render_center: ResMut<MapRenderCenter>,
// ) {
//     let Ok(transform) = query.get_single() else {
//         return;
//     };
//
//     let position = transform.translation;
//     let chunk_position = ChunkPosition3::from_world_pos(position);
//     if map_render_center.center != chunk_position {
//         *map_render_center = MapRenderCenter {
//             center: chunk_position,
//         };
//         info!(
//             "Map render center ({}, {})",
//             chunk_position.x.0, chunk_position.z.0
//         );
//     }
// }

#[derive(Debug, Clone, Copy)]
struct ChunkNode {
    center: Vec3,
    half_size: f32,
}

impl ChunkNode {
    fn root(center: Vec3, size: f32) -> Self {
        Self::new(center, size / 2.0)
    }

    fn new(center: Vec3, half_size: f32) -> Self {
        Self { center, half_size }
    }

    fn subdivide(self, terrain: &TerrainSampler, base_seed: &BaseSeed) -> [Self; 4] {
        let mut result = Vec::with_capacity(4);
        let directions = [
            vec2(1.0, 1.0),
            vec2(1.0, -1.0),
            vec2(-1.0, 1.0),
            vec2(-1.0, -1.0),
        ];

        for direction in directions {
            let center_2d = self.center.xz() + direction * self.half_size / 2.0;
            let center = vec3(
                center_2d.x,
                terrain.sample(center_2d, base_seed).value,
                center_2d.y,
            );

            let half_size = self.half_size / 2.0;

            result.push(Self::new(center, half_size));
        }

        result.try_into().unwrap()
    }

    fn as_chunk(&self) -> (Chunk, Transform) {
        let size = self.half_size * 2.0;
        (
            Chunk::new(CHUNK_SUBDIVISIONS, size),
            Transform::from_translation(self.center),
        )
    }
}

fn setup_chunk_container(mut commands: Commands) {
    commands.spawn((
        Name::new("Chunks"),
        SpatialBundle::default(),
        StateScoped(GameState::Playing),
        ChunksContainer,
    ));
}

fn spawn_chunks(
    mut commands: Commands,
    chunks_container: Query<(Entity, Option<&Children>), With<ChunksContainer>>,
    existing_chunks: Query<(Entity, &Chunk, &Transform)>,
    camera: Query<(&Projection, &Transform), With<GameplayCamera>>,
    base_seed: Res<BaseSeed>,
    terrain_sampler: Res<TerrainSampler>,
    desired_surface_area: Res<DesiredSurfaceArea>,
) {
    let Some((fov, camera_position)) =
        camera
            .get_single()
            .ok()
            .and_then(|(projection, transform)| match projection {
                Projection::Perspective(perspective_projection) => {
                    Some((perspective_projection.fov, transform.translation))
                }
                _ => None,
            })
    else {
        error!("No perspective gameplay camera found");
        return;
    };

    let fov_2_tan = (fov / 2.0).tan();
    let ground_at_camera = terrain_sampler.sample(camera_position.xz(), &base_seed);
    let ground_position = vec3(camera_position.x, ground_at_camera.value, camera_position.z);

    let distance_to_ground = (camera_position.y - ground_at_camera.value).abs();

    #[cfg(feature = "dev")]
    {
        assert!(distance_to_ground > 0.0, "{distance_to_ground}");
        assert!(!chunks_container.is_empty(), "no chunks containers");
    }

    for (container_entity, container_children) in chunks_container.iter() {
        let container_chunks = container_children.map(|children| {
            children
                .iter()
                .filter_map(|child| existing_chunks.get(*child).ok())
        });

        let mut result_nodes = Vec::with_capacity(4);
        let mut nodes_to_check = ChunkNode::root(ground_position, distance_to_ground * fov_2_tan)
            .subdivide(&terrain_sampler, &base_seed)
            .to_vec();

        while let Some(node) = nodes_to_check.pop() {
            let distance = node.center.distance(camera_position);
            let projected_size = node.half_size * 2.0 / (distance * fov_2_tan);

            // subdivide until desired surface area is reached
            if projected_size > desired_surface_area.0 {
                nodes_to_check.extend(node.subdivide(&terrain_sampler, &base_seed));
            } else {
                // then try to create chunk
                let chunk = node.as_chunk();

                if let Some(container_chunks) = container_chunks.clone() {
                    // check if exact chunk is already in container, skip if it is
                    for (_, _, transform) in container_chunks {
                        if (chunk.1.translation.xz() - transform.translation.xz()).element_sum()
                            < 0.01
                        {
                            info!("Skipping");
                            continue;
                        }
                    }
                }

                info!("Pushing");
                result_nodes.push(chunk);
            }
        }

        // despawn replaced chunks
        if let Some(container_chunks) = container_chunks {
            for (entity, chunk, transform) in container_chunks {
                for (new_chunk, new_transform) in result_nodes.iter() {
                    let Some(entity_commands) = commands.get_entity(entity) else {
                        continue;
                    };
                    if chunk.contains_chunk(transform, new_transform)
                        || new_chunk.contains_chunk(new_transform, transform)
                    {
                        entity_commands.despawn_recursive();
                    }
                }
            }
        }

        // Spawn chunks
        commands.entity(container_entity).with_children(|commands| {
            info!("Spawning {} chunks", result_nodes.len());
            for (chunk, transform) in result_nodes {
                commands.spawn((
                    StateScoped(GameState::Playing),
                    Name::from(format!("Chunk {:.0}", chunk.size)),
                    chunk,
                    SpatialBundle {
                        transform,
                        ..default()
                    },
                    CollisionLayers::get_terrain_colliders(),
                ));
            }
        });
    }
}

fn render_chunks(
    mut commands: Commands,
    chunks: Query<(Entity, &Chunk), Without<Handle<Mesh>>>,
    base_seed: Res<BaseSeed>,
    terrain_sampler: Res<TerrainSampler>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, chunk) in chunks.iter() {
        let mut mesh =
            utils::primitives::create_subdivided_plane(chunk.subdivisions, chunk.size, |x, y| {
                terrain_sampler
                    .sample(vec2(x, y), &base_seed)
                    .to_mesh_input()
            });

        mesh.asset_usage = RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD;

        let render_entity = commands
            .spawn((
                Name::new("Chunk Render"),
                PbrBundle {
                    mesh: meshes.add(mesh),
                    ..default()
                },
            ))
            .id();

        commands.entity(entity).add_child(render_entity);
    }
}

#[derive(SystemParam)]
struct TerrrainCollidablesQuery<'w, 's> {
    // TODO: replace Collider with some marker
    collidables_query:
        Query<'w, 's, (&'static Transform, &'static CollisionLayers), With<Collider>>,
}

impl<'w, 's> TerrrainCollidablesQuery<'w, 's> {
    fn iter(&self) -> impl Iterator<Item = &'_ Transform> {
        let terrain_collision_layers = CollisionLayers::get_terrain_colliders();
        self.collidables_query
            .iter()
            .filter_map(move |(transform, layers)| {
                layers
                    .interacts_with(terrain_collision_layers)
                    .then_some(transform)
            })
    }

    fn is_chunk_in_proximity(
        &self,
        chunk: &Chunk,
        chunk_transform: &Transform,
        radius: f32,
    ) -> bool {
        // slow, use quad tree
        for collidable_transform in self.iter() {
            let collidable_center = collidable_transform.translation.xz();
            let points = vec![
                collidable_center + vec2(radius, radius),
                collidable_center + vec2(-radius, -radius),
                collidable_center + vec2(radius, -radius),
                collidable_center + vec2(-radius, radius),
            ];

            for point in points {
                if chunk.contains_point(chunk_transform, point) {
                    return true;
                }
            }
        }

        false
    }
}

fn add_colliders_to_chunks(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    chunks_query: Query<(Entity, &Chunk, &Handle<Mesh>, &Transform), Without<Collider>>,
    collidables_query: TerrrainCollidablesQuery,
) {
    for (entity, chunk, mesh_handle, chunk_transform) in chunks_query.iter() {
        let Some(mesh) = collidables_query
            .is_chunk_in_proximity(chunk, chunk_transform, 1.0)
            .then(|| meshes.get(mesh_handle))
            .flatten()
        else {
            continue;
        };

        let collider = Collider::trimesh_from_mesh(mesh).unwrap();
        commands.entity(entity).insert(collider);
    }
}

fn remove_colliders_from_chunks(
    mut commands: Commands,
    chunks_query: Query<(Entity, &Chunk, &Transform), With<Collider>>,
    collidables_query: TerrrainCollidablesQuery,
) {
    for (entity, chunk, chunk_transform) in chunks_query.iter() {
        let Some(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };
        if !collidables_query.is_chunk_in_proximity(chunk, chunk_transform, 1.0) {
            entity_commands.remove::<Collider>();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn chunk_contains_point() {
        let chunk = Chunk::new(2, 10.0);
        let transform = Transform::from_translation(vec3(3.0, 3.0, 3.0));

        let points_to_contain = vec![
            vec2(13.0, 13.0),
            vec2(-7.0, 13.0),
            vec2(13.0, -7.0),
            vec2(-7.0, -7.0),
            vec2(13.0, 13.0),
            vec2(13.0, -7.0),
            vec2(-7.0, 13.0),
            vec2(-7.0, -7.0),
            vec2(1.5, 1.5),
            vec2(1.5, -1.5),
            vec2(-1.5, 1.5),
            vec2(-1.5, -1.5),
        ];

        for point in points_to_contain {
            assert!(
                chunk.contains_point(&transform, point),
                "should contain point: {point:?}"
            );
        }
    }

    #[test]
    fn chunk_not_contain_point() {
        let chunk = Chunk::new(2, 10.0);
        let transform = Transform::from_translation(vec3(3.0, 3.0, 3.0));

        let points_not_to_contain = vec![
            vec2(14.0, 14.0),
            vec2(-8.0, 14.0),
            vec2(14.0, -8.0),
            vec2(-8.0, -8.0),
            vec2(14.0, 14.0),
            vec2(14.0, -8.0),
            vec2(-8.0, 14.0),
            vec2(-8.0, -8.0),
        ];

        for point in points_not_to_contain {
            assert!(
                !chunk.contains_point(&transform, point),
                "should not contain point: {point:?}"
            );
        }
    }
}

// fn render_center_changed(center: Res<MapRenderCenter>) -> bool {
//     center.is_changed()
// }

// fn despawn_unregister_out_of_range_chunks(
//     mut commands: Commands,
//     center: Res<MapRenderCenter>,
//     mut chunk_manager: ResMut<ChunkManager>,
//     terrain: Res<TerrainSampler>,
// ) {
//     let chunk_rect =
//         ChunkRect::from_circle_outside(center.center.to_2d(), terrain.chunk_spawn_radius.into());
//
//     let removed_chunks = chunk_manager
//         .chunks
//         .extract_if(|pos, _| !chunk_rect.contains(pos));
//
//     let mut count = 0;
//     for (_, chunk) in removed_chunks {
//         count += 1;
//         commands.entity(chunk).despawn();
//     }
//
//     if count > 0 {
//         info!("Despawned {} chunks", count);
//     }
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// struct ChunkRect {
//     from_x: ChunkUnit,
//     to_x: ChunkUnit,
//     from_y: ChunkUnit,
//     to_y: ChunkUnit,
// }
//
// impl ChunkRect {
//     fn from_circle_outside(center: ChunkPosition2, chunk_radius: i32) -> Self {
//         let center = center.to_ivec();
//         let from_x = center.x - chunk_radius;
//         let to_x = center.x + chunk_radius;
//
//         let from_y = center.y - chunk_radius;
//         let to_y = center.y + chunk_radius;
//
//         Self {
//             from_x: ChunkUnit(from_x),
//             to_x: ChunkUnit(to_x),
//             from_y: ChunkUnit(from_y),
//             to_y: ChunkUnit(to_y),
//         }
//     }
//     fn contains(&self, chunk_position: &ChunkPosition2) -> bool {
//         self.from_x <= chunk_position.x
//             && chunk_position.x <= self.to_x
//             && self.from_y <= chunk_position.y
//             && chunk_position.y <= self.to_y
//     }
// }
//
// fn register_chunks(
//     mut chunk_manager: ResMut<ChunkManager>,
//     chunks: Query<(Entity, &ChunkPosition3), Added<Chunk>>,
// ) {
//     for (chunk, chunk_position) in chunks.iter() {
//         chunk_manager.chunks.insert(chunk_position.to_2d(), chunk);
//     }
// }
//
// fn render_chunks(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     center: Res<MapRenderCenter>,
//     chunks: Query<(Entity, &ChunkPosition3), (With<Chunk>, Without<Handle<Mesh>>)>,
//     terrain: Res<TerrainSampler>,
// ) {
//     let center = center.center.to_ivec().xz();
//     let render_radius_squared: i32 =
//         terrain.chunk_visibility_radius as i32 * terrain.chunk_visibility_radius as i32;
//
//     let mut count = 0;
//
//     for (chunk_entity, chunk_position) in chunks.iter() {
//         if !is_chunk_in_range(center, chunk_position, render_radius_squared) {
//             continue;
//         }
//         let chunk_translation = chunk_position.to_2d().to_world_pos();
//         let mut mesh = utils::primitives::create_subdivided_plane(
//             terrain.chunk_subdivisions,
//             CHUNK_SIZE,
//             terrain.chunk_sampler(chunk_translation),
//         );
//         mesh.asset_usage = RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD;
//
//         let transform = Transform::from_translation(chunk_position.to_world_pos());
//         let material = StandardMaterial {
//             base_color: tailwind::GRAY_400.into(),
//             perceptual_roughness: 0.9,
//
//             ..default()
//         };
//
//         commands.entity(chunk_entity).insert(MaterialMeshBundle {
//             mesh: asset_server.add(mesh),
//             material: asset_server.add(material),
//             transform,
//             ..default()
//         });
//
//         count += 1;
//     }
//
//     if count > 0 {
//         info!("Rendered {} chunks", count);
//     }
// }
//
// fn derender_chunks(
//     mut commands: Commands,
//     center: Res<MapRenderCenter>,
//     chunks: Query<(Entity, &ChunkPosition3), (With<Chunk>, With<Handle<Mesh>>)>,
//     terrain: Res<TerrainSampler>,
// ) {
//     let render_radius_squared =
//         terrain.chunk_visibility_radius as i32 * terrain.chunk_visibility_radius as i32;
//
//     let center = center.center.to_ivec().xz();
//     for (chunk_entity, chunk_position) in chunks.iter() {
//         if is_chunk_in_range(center, chunk_position, render_radius_squared) {
//             continue;
//         }
//
//         commands.entity(chunk_entity).remove::<Handle<Mesh>>();
//         commands
//             .entity(chunk_entity)
//             .remove::<Handle<StandardMaterial>>();
//     }
// }
//
// fn is_chunk_in_range(
//     center: IVec2,
//     chunk_position: &ChunkPosition3,
//     render_radius_squared: i32,
// ) -> bool {
//     let distance_squared = center.distance_squared(chunk_position.to_ivec().xz());
//
//     distance_squared <= render_radius_squared
// }
//
// fn add_colliders_to_chunks(
//     mut commands: Commands,
//     chunks_query: Query<&Handle<Mesh>, Without<Collider>>,
//     chunk_manager: Res<ChunkManager>,
//     render_center: Res<MapRenderCenter>,
//     meshes: Res<Assets<Mesh>>,
// ) {
//     let center = render_center.center.to_ivec().xz();
//
//     for chunk_position in DIRECTIONS_2D
//         .into_iter()
//         .map(|d| ChunkPosition3::new(d.x + center.x, 0, d.y + center.y))
//         .chain([ChunkPosition3::new(center.x, 0, center.y)])
//     {
//         let chunk_entity_option = chunk_manager.chunks.get(&chunk_position.to_2d());
//
//         let Some(collider) = chunk_entity_option
//             .and_then(|e| chunks_query.get(*e).ok())
//             .and_then(|h| meshes.get(h))
//             .and_then(Collider::trimesh_from_mesh)
//         else {
//             continue;
//         };
//
//         commands
//             .entity(*chunk_entity_option.unwrap())
//             .insert(collider);
//     }
// }
//
// fn remove_colliders_from_chunks(
//     mut commands: Commands,
//     chunks_query: Query<(Entity, &ChunkPosition3), With<Collider>>,
//     render_center: Res<MapRenderCenter>,
// ) {
//     let center = render_center.center.to_ivec().xz();
//     for (entity, chunk_position) in chunks_query.iter().map(|(e, p)| (e, p.to_ivec().xz())) {
//         if chunk_position.distance_squared(center) > 2 {
//             commands.entity(entity).remove::<Collider>();
//         }
//     }
// }
//
// #[cfg(any(test, feature = "dev"))]
// impl TerrainSampler {
//     fn sample_estimate_normal(&self, pos: Vec2) -> Vec3 {
//         let dfdx = noise::estimate_dt1(pos.x, |x| self.sample(vec2(x, pos.y)).value);
//         let dfdy = noise::estimate_dt1(pos.y, |y| self.sample(vec2(pos.x, y)).value);
//
//         noise::Dt2(vec2(dfdx, dfdy)).get_normal()
//     }
//
//     pub fn chunk_sampler_estimate(
//         &self,
//         chunk_translation: Vec2,
//     ) -> impl Fn(f32, f32) -> (f32, [f32; 3]) + '_ {
//         move |x, y| {
//             let pos = chunk_translation + vec2(x, y);
//
//             let value = self.sample(pos).value;
//             let normal = self.sample_estimate_normal(pos);
//             (value, normal.into())
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use bevy::render::mesh::VertexAttributeValues;
//
//     use super::*;
//
//     fn get_terrain_config() -> TerrainSampler {
//         TerrainSampler::default()
//     }
//
//     fn extract_values(data: Option<&VertexAttributeValues>) -> Vec<Vec3> {
//         match data {
//             Some(VertexAttributeValues::Float32x3(d)) => {
//                 d.iter().map(|v| Vec3::new(v[0], v[1], v[2])).collect_vec()
//             }
//             _ => panic!("Expected Float32x3"),
//         }
//     }
//
//     #[test]
//     fn should_have_correct_normals() {
//         let epsilon = 0.05;
//         let max_error_epsilon = 0.1;
//         let min_success_percent = 95.0;
//         let terrain = get_terrain_config();
//
//         let generator = terrain.chunk_sampler(vec2(0.0, 0.0));
//         let generator_estimate = terrain.chunk_sampler_estimate(vec2(0.0, 0.0));
//
//         let mesh = utils::primitives::create_subdivided_plane(
//             terrain.chunk_subdivisions,
//             CHUNK_SIZE,
//             generator,
//         );
//         let auto_mesh = utils::primitives::create_subdivided_plane(
//             terrain.chunk_subdivisions,
//             CHUNK_SIZE,
//             generator_estimate,
//         );
//
//         let auto_mesh_positions = extract_values(auto_mesh.attribute(Mesh::ATTRIBUTE_POSITION));
//         let auto_mesh_normals = extract_values(auto_mesh.attribute(Mesh::ATTRIBUTE_NORMAL));
//
//         let mesh_positions = extract_values(mesh.attribute(Mesh::ATTRIBUTE_POSITION));
//         let mesh_normals = extract_values(mesh.attribute(Mesh::ATTRIBUTE_NORMAL));
//
//         assert_eq!(auto_mesh_positions.len(), mesh_positions.len());
//         assert_eq!(auto_mesh_normals.len(), mesh_normals.len());
//
//         let positions_zip = auto_mesh_positions
//             .into_iter()
//             .zip(mesh_positions)
//             .collect_vec();
//         let normals_zip = auto_mesh_normals
//             .into_iter()
//             .zip(mesh_normals)
//             .collect_vec();
//
//         let normals_errors = normals_zip
//             .clone()
//             .into_iter()
//             .map(|(a, b)| (a - b).abs())
//             .collect_vec();
//
//         let unacceptable_errors = normals_errors
//             .iter()
//             .enumerate()
//             .filter(|(_, v)| v.x > epsilon || v.y > epsilon || v.z > epsilon)
//             .collect_vec();
//
//         let max_error = normals_errors
//             .iter()
//             .max_by(|a, b| a.length().partial_cmp(&b.length()).unwrap())
//             .copied()
//             .unwrap_or_default();
//
//         let average_error = normals_errors.iter().sum::<Vec3>() / normals_errors.len() as f32;
//
//         let success_percent = (normals_errors.len() - unacceptable_errors.len()) as f32
//             / normals_errors.len() as f32
//             * 100.0;
//
//         println!("Failed normals: ");
//         for (i, error) in unacceptable_errors {
//             let auto_pos = positions_zip[i].0;
//             let (auto_normal, mesh_normal) = normals_zip[i];
//
//             println!("P{auto_pos:.3}\t{auto_normal:.3}!={mesh_normal}\tE: {error:.3}");
//         }
//
//         assert!(
//             success_percent > min_success_percent,
//             "SUCCESS: {success_percent:.2}% < {min_success_percent:.2}%"
//         );
//         assert!(
//             max_error.x < max_error_epsilon
//                 && max_error.y < max_error_epsilon
//                 && max_error.z < max_error_epsilon,
//             "MAX E: {max_error}, AVG E: {average_error}, MAX ACCEPTED: {max_error_epsilon}"
//         );
//     }
// }
