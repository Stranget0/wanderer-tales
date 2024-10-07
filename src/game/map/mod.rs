use crate::prelude::*;

use bevy::utils::hashbrown::HashMap;
use utils::noise::{self, Dt2, NoiseHasher, PcgHasher};

#[cfg(feature = "dev")]
pub mod map_devtools;

const CHUNK_SUBDIVISIONS: u32 = 32;

// World to chunk coordinates
const CHUNK_SIZE: f32 = 64.0;

// These are in chunks
const CHUNK_SPAWN_RADIUS: u8 = 100;
const CHUNK_VISIBILITY_RADIUS: u8 = CHUNK_SPAWN_RADIUS;

// size amplitude erosion
const MAP_GENERATOR_WEIGHTS: [(f32, f32, f32); 6] = [
    (10000.0, 1000.0, 100.0),
    (5000.0, 700.0, 40.0),
    (2500.0, 200.0, 10.0),
    (500.0, 150.0, 10.0),
    (100.0, 100.0, 10.0),
    (50.0, 10.0, 1.0),
];

#[derive(Component)]
pub struct ChunkOrigin;

#[derive(Component)]
pub struct Chunk;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct ChunkUnit(pub i32);
impl ChunkUnit {
    pub fn to_world_unit(self) -> i32 {
        self.0 * CHUNK_SIZE as i32
    }
    pub fn from_world_unit(v: f32) -> i32 {
        (v / CHUNK_SIZE) as i32
    }
}
#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
#[repr(C)]
pub struct ChunkPosition3 {
    pub x: ChunkUnit,
    pub y: ChunkUnit,
    pub z: ChunkUnit,
}

impl ChunkPosition3 {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x: ChunkUnit(x),
            y: ChunkUnit(y),
            z: ChunkUnit(z),
        }
    }

    pub fn to_vec(self) -> Vec3 {
        vec3(
            self.x.0 as f32 * CHUNK_SIZE,
            self.y.0 as f32 * CHUNK_SIZE,
            self.z.0 as f32 * CHUNK_SIZE,
        )
    }
    pub fn to_ivec(self) -> IVec3 {
        ivec3(self.x.0, self.y.0, self.z.0)
    }
    pub fn to_world_pos(self) -> Vec3 {
        vec3(
            self.x.to_world_unit() as f32,
            self.y.to_world_unit() as f32,
            self.z.to_world_unit() as f32,
        )
    }
    pub fn from_world_pos(pos: Vec3) -> Self {
        let x = ChunkUnit::from_world_unit(pos.x);
        let y = ChunkUnit::from_world_unit(pos.y);
        let z = ChunkUnit::from_world_unit(pos.z);
        Self::new(x, y, z)
    }
    pub fn to_2d(self) -> ChunkPosition2 {
        ChunkPosition2::new(self.x.0, self.z.0)
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub struct ChunkPosition2 {
    pub x: ChunkUnit,
    pub y: ChunkUnit,
}
impl ChunkPosition2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x: ChunkUnit(x),
            y: ChunkUnit(y),
        }
    }

    pub fn to_vec(self) -> Vec2 {
        vec2(self.x.0 as f32 * CHUNK_SIZE, self.y.0 as f32 * CHUNK_SIZE)
    }
    pub fn to_ivec(self) -> IVec2 {
        ivec2(self.x.0, self.y.0)
    }
    pub fn to_world_pos(self) -> Vec2 {
        vec2(self.x.to_world_unit() as f32, self.y.to_world_unit() as f32)
    }
}

fn chunk_position_3(x: i32, y: i32, z: i32) -> ChunkPosition3 {
    ChunkPosition3::new(x, y, z)
}
fn chunk_position_2(x: i32, y: i32) -> ChunkPosition2 {
    ChunkPosition2::new(x, y)
}

#[derive(Resource, Default)]
pub struct MapRenderCenter {
    center: ChunkPosition3,
}
impl MapRenderCenter {
    pub fn to_world_pos(&self) -> Vec3 {
        self.center.to_vec()
    }
}

#[derive(Resource, Default)]
struct ChunkManager {
    pub chunks: HashMap<ChunkPosition2, Entity>,
}

#[derive(Resource, Default)]
pub struct MapSeed(pub u32);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum MapSystemSets {
    ChunkMutate,
    ChunkRegister,
    ChunkRender,

    ChunkReload,
    Input,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<MapRenderCenter>()
        .init_resource::<ChunkManager>()
        .init_resource::<MapSeed>()
        .configure_sets(
            Startup,
            (
                MapSystemSets::ChunkMutate,
                MapSystemSets::ChunkRegister,
                MapSystemSets::ChunkRender,
            )
                .chain(),
        )
        .configure_sets(
            Update,
            (
                #[cfg(feature = "dev")]
                MapSystemSets::ChunkReload.run_if(map_devtools::seed_changed),
                MapSystemSets::ChunkMutate.run_if(render_center_changed),
                MapSystemSets::ChunkRegister,
                MapSystemSets::ChunkRender,
            )
                .chain(),
        )
        .add_systems(
            Startup,
            (
                spawn_chunks.in_set(MapSystemSets::ChunkMutate),
                render_chunks.in_set(MapSystemSets::ChunkRender),
                register_chunks.in_set(MapSystemSets::ChunkRegister),
            ),
        )
        .add_systems(
            Update,
            (
                update_map_render_center.before(MapSystemSets::ChunkMutate),
                register_chunks.in_set(MapSystemSets::ChunkRegister),
                (
                    spawn_chunks.in_set(MapSystemSets::ChunkMutate),
                    despawn_unregister_out_of_range_chunks.in_set(MapSystemSets::ChunkMutate),
                    render_chunks.in_set(MapSystemSets::ChunkRender),
                    derender_chunks.in_set(MapSystemSets::ChunkRender),
                )
                    .run_if(render_center_changed),
            ),
        );
}

pub fn map_generator(chunk_translation: Vec2, seed: u32) -> impl Fn(f32, f32) -> (f32, [f32; 3]) {
    move |x, y| {
        let pos = chunk_translation + vec2(x, y);

        sample_terrain(pos, seed).to_mesh_input()
    }
}

fn sample_terrain(pos: Vec2, seed: u32) -> noise::ValueDt2 {
    let (size, amplitude, erosion) = MAP_GENERATOR_WEIGHTS[0];
    let mut hasher = PcgHasher::from_seed(seed);
    let layer_1 = (utils::noise::perlin_noise_2d(pos, 1.0 / size, &hasher) / 2.0 + 0.5) * amplitude;

    let mut erosion_factor = layer_1.dt_length();
    let mut terrain = layer_1.to_dt2() / (1.0 + erosion * erosion_factor);

    for (size, amplitude, erosion) in MAP_GENERATOR_WEIGHTS.into_iter().skip(1) {
        hasher = hasher.with_next_seed();
        let layer =
            (utils::noise::perlin_noise_2d(pos, 1.0 / size, &hasher) / 2.0 + 0.5) * amplitude;

        erosion_factor = erosion_factor + layer.dt_length();

        terrain = terrain + (layer.to_dt2() / (1.0 + erosion * erosion_factor));
    }

    terrain
}

fn update_map_render_center(
    query: Query<&Transform, With<ChunkOrigin>>,
    mut map_render_center: ResMut<MapRenderCenter>,
) {
    let Ok(transform) = query.get_single() else {
        return;
    };

    let position = transform.translation;
    let chunk_position = ChunkPosition3::from_world_pos(position);
    if map_render_center.center != chunk_position {
        *map_render_center = MapRenderCenter {
            center: chunk_position,
        };
        info!(
            "Map render center ({}, {})",
            chunk_position.x.0, chunk_position.z.0
        );
    }
}

fn spawn_chunks(
    mut commands: Commands,
    center: Res<MapRenderCenter>,
    chunk_manager: Res<ChunkManager>,
) {
    let ChunkRect {
        from_x,
        to_x,
        from_y,
        to_y,
    } = ChunkRect::from_circle_outside(center.center.to_2d(), CHUNK_SPAWN_RADIUS.into());

    let mut bundles = Vec::new();
    for x in from_x.0..=to_x.0 {
        for y in from_y.0..=to_y.0 {
            let chunk_position = chunk_position_3(x, 0, y);
            if chunk_manager.chunks.contains_key(&chunk_position.to_2d()) {
                continue;
            }

            bundles.push((
                Chunk,
                Name::new(format!("Chunk {}x{}", x, y)),
                chunk_position,
            ));
        }
    }
    if !bundles.is_empty() {
        info!("Spawning {} chunks", bundles.len());
    }
    commands.spawn_batch(bundles);
}

fn render_center_changed(center: Res<MapRenderCenter>) -> bool {
    center.is_changed()
}

fn despawn_unregister_out_of_range_chunks(
    mut commands: Commands,
    center: Res<MapRenderCenter>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    let chunk_rect =
        ChunkRect::from_circle_outside(center.center.to_2d(), CHUNK_SPAWN_RADIUS.into());

    let removed_chunks = chunk_manager
        .chunks
        .extract_if(|pos, _| !chunk_rect.contains(pos));

    let mut count = 0;
    for (_, chunk) in removed_chunks {
        count += 1;
        commands.entity(chunk).despawn();
    }

    if count > 0 {
        info!("Despawned {} chunks", count);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ChunkRect {
    from_x: ChunkUnit,
    to_x: ChunkUnit,
    from_y: ChunkUnit,
    to_y: ChunkUnit,
}

impl ChunkRect {
    fn from_circle_outside(center: ChunkPosition2, chunk_radius: i32) -> Self {
        let center = center.to_ivec();
        let from_x = center.x - chunk_radius;
        let to_x = center.x + chunk_radius;

        let from_y = center.y - chunk_radius;
        let to_y = center.y + chunk_radius;

        Self {
            from_x: ChunkUnit(from_x),
            to_x: ChunkUnit(to_x),
            from_y: ChunkUnit(from_y),
            to_y: ChunkUnit(to_y),
        }
    }
    fn contains(&self, chunk_position: &ChunkPosition2) -> bool {
        self.from_x <= chunk_position.x
            && chunk_position.x <= self.to_x
            && self.from_y <= chunk_position.y
            && chunk_position.y <= self.to_y
    }
}

fn register_chunks(
    mut chunk_manager: ResMut<ChunkManager>,
    chunks: Query<(Entity, &ChunkPosition3), Added<Chunk>>,
) {
    for (chunk, chunk_position) in chunks.iter() {
        chunk_manager.chunks.insert(chunk_position.to_2d(), chunk);
    }
}

fn render_chunks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    center: Res<MapRenderCenter>,
    chunks: Query<(Entity, &ChunkPosition3), (With<Chunk>, Without<Handle<Mesh>>)>,

    seed: Res<MapSeed>,
) {
    let center = center.center.to_ivec().xz();
    let render_radius_squared = CHUNK_VISIBILITY_RADIUS as i32 * CHUNK_VISIBILITY_RADIUS as i32;

    let mut count = 0;

    let mut highest_point = -1.0;
    let mut lowest_point = 1.0;
    for (chunk_entity, chunk_position) in chunks.iter() {
        if !is_chunk_in_range(center, chunk_position, render_radius_squared) {
            continue;
        }
        let chunk_translation = chunk_position.to_2d().to_world_pos();
        let mesh = utils::primitives::create_subdivided_plane(
            CHUNK_SUBDIVISIONS,
            CHUNK_SIZE,
            map_generator(chunk_translation, seed.0),
        );

        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            for v in positions {
                let y = v[1];
                if y > highest_point {
                    highest_point = y;
                }
                if y < lowest_point {
                    lowest_point = y;
                }
            }
        }

        let transform = Transform::from_translation(chunk_position.to_world_pos());
        let material = StandardMaterial {
            base_color: tailwind::GRAY_400.into(),
            perceptual_roughness: 0.9,

            ..default()
        };

        commands.entity(chunk_entity).insert((MaterialMeshBundle {
            mesh: asset_server.add(mesh),
            material: asset_server.add(material),
            transform,
            ..default()
        },));

        count += 1;
    }

    info!("Highest point: {highest_point}, lowest point: {lowest_point}");

    if count > 0 {
        info!("Rendered {} chunks", count);
    }
}

fn derender_chunks(
    mut commands: Commands,
    center: Res<MapRenderCenter>,
    chunks: Query<(Entity, &ChunkPosition3), (With<Chunk>, With<Handle<Mesh>>)>,
) {
    let render_radius_squared = CHUNK_VISIBILITY_RADIUS as i32 * CHUNK_VISIBILITY_RADIUS as i32;
    let center = center.center.to_ivec().xz();
    for (chunk_entity, chunk_position) in chunks.iter() {
        if is_chunk_in_range(center, chunk_position, render_radius_squared) {
            continue;
        }

        commands.entity(chunk_entity).remove::<Handle<Mesh>>();
        commands
            .entity(chunk_entity)
            .remove::<Handle<StandardMaterial>>();
        commands.entity(chunk_entity).remove::<DebugNormals>();
    }
}

fn estimate_map_normal(pos: Vec2, seed: u32) -> Vec3 {
    let dfdx = noise::estimate_derivative(pos.x, |x| sample_terrain(vec2(x, pos.y), seed).value);
    let dfdy = noise::estimate_derivative(pos.y, |y| sample_terrain(vec2(pos.x, y), seed).value);

    Dt2(vec2(dfdx, dfdy)).get_normal()
}

fn is_chunk_in_range(
    center: IVec2,
    chunk_position: &ChunkPosition3,
    render_radius_squared: i32,
) -> bool {
    let distance_squared = center.distance_squared(chunk_position.to_ivec().xz());

    distance_squared <= render_radius_squared
}

#[cfg(test)]
mod tests {
    use bevy::render::mesh::VertexAttributeValues;

    use super::*;

    fn extract_values(data: Option<&VertexAttributeValues>) -> Vec<Vec3> {
        match data {
            Some(VertexAttributeValues::Float32x3(d)) => {
                d.iter().map(|v| Vec3::new(v[0], v[1], v[2])).collect_vec()
            }
            _ => panic!("Expected Float32x3"),
        }
    }

    #[test]
    fn should_have_correct_normals() {
        let epsilon = 0.05;
        let max_error_epsilon = 0.1;
        let min_success_percent = 95.0;

        let generator = map_generator(vec2(0.0, 0.0), 0);

        let auto_mesh = utils::primitives::create_subdivided_plane_smooth(
            CHUNK_SUBDIVISIONS,
            CHUNK_SIZE,
            |x, y| (generator(x, y).0, estimate_map_normal(vec2(x, y), 0).into()),
        );
        let mesh =
            utils::primitives::create_subdivided_plane(CHUNK_SUBDIVISIONS, CHUNK_SIZE, generator);

        let auto_mesh_positions = extract_values(auto_mesh.attribute(Mesh::ATTRIBUTE_POSITION));
        let auto_mesh_normals = extract_values(auto_mesh.attribute(Mesh::ATTRIBUTE_NORMAL));

        let mesh_positions = extract_values(mesh.attribute(Mesh::ATTRIBUTE_POSITION));
        let mesh_normals = extract_values(mesh.attribute(Mesh::ATTRIBUTE_NORMAL));

        assert_eq!(auto_mesh_positions.len(), mesh_positions.len());
        assert_eq!(auto_mesh_normals.len(), mesh_normals.len());

        let positions_zip = auto_mesh_positions
            .into_iter()
            .zip(mesh_positions)
            .collect_vec();
        let normals_zip = auto_mesh_normals
            .into_iter()
            .zip(mesh_normals)
            .collect_vec();

        let normals_errors = normals_zip
            .clone()
            .into_iter()
            .map(|(a, b)| (a - b).abs())
            .collect_vec();

        let unacceptable_errors = normals_errors
            .iter()
            .enumerate()
            .filter(|(_, v)| v.x > epsilon || v.y > epsilon || v.z > epsilon)
            .collect_vec();

        let max_error = normals_errors
            .iter()
            .max_by(|a, b| a.length().partial_cmp(&b.length()).unwrap())
            .copied()
            .unwrap_or_default();

        let average_error = normals_errors.iter().sum::<Vec3>() / normals_errors.len() as f32;

        let success_percent = (normals_errors.len() - unacceptable_errors.len()) as f32
            / normals_errors.len() as f32
            * 100.0;

        println!("Failed normals: ");
        for (i, error) in unacceptable_errors {
            let auto_pos = positions_zip[i].0;
            let (auto_normal, mesh_normal) = normals_zip[i];

            println!("P{auto_pos:.3}\t{auto_normal:.3}!={mesh_normal}\tE: {error:.3}");
        }

        assert!(
            success_percent > min_success_percent,
            "SUCCESS: {success_percent:.2}% < {min_success_percent:.2}%"
        );
        assert!(
            max_error.x < max_error_epsilon
                && max_error.y < max_error_epsilon
                && max_error.z < max_error_epsilon,
            "MAX E: {max_error}, AVG E: {average_error}, MAX ACCEPTED: {max_error_epsilon}"
        );
    }
}
