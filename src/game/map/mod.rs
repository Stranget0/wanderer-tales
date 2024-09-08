use crate::prelude::*;
use libnoise::prelude::*;

use bevy::{
    color::palettes::tailwind,
    pbr::{wireframe::Wireframe, ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    utils::hashbrown::HashMap,
};

const CHUNK_SUBDIVISIONS: u32 = 2;
const CHUNK_DATA_LEN: u32 = CHUNK_SUBDIVISIONS * CHUNK_SUBDIVISIONS;

// World to chunk coordinates
const CHUNK_SIZE: f32 = 16.0;

// These are in chunks
const CHUNK_SPAWN_RADIUS: u8 = 64;
const CHUNK_VISIBILITY_RADIUS: u8 = 64;

#[derive(Component)]
pub struct ChunkOrigin;

#[derive(Component)]
struct Chunk;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
struct ChunkUnit(pub i32);
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
struct ChunkPosition3 {
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
struct ChunkPosition2 {
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
    chunks: HashMap<ChunkPosition2, Entity>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<MapRenderCenter>()
        .init_resource::<ChunkManager>()
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, MapMaterialExtension>,
        >::default())
        .add_systems(Startup, spawn_map_shader)
        .add_systems(
            Update,
            (
                spawn_chunks,
                register_chunks,
                despawn_unregister_chunks,
                render_chunks,
                update_map_render_center,
            ),
        );
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
    if !center.is_changed() {
        return;
    }
    let ChunkRect {
        from_x,
        to_x,
        from_y,
        to_y,
    } = ChunkRect::from_circle_outside(center.center, CHUNK_SPAWN_RADIUS.into());

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
    info!("Spawning {} chunks", bundles.len());
    commands.spawn_batch(bundles);
}

fn despawn_unregister_chunks(
    mut commands: Commands,
    center: Res<MapRenderCenter>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    if !center.is_changed() {
        return;
    }
    let chunk_rect = ChunkRect::from_circle_outside(center.center, CHUNK_SPAWN_RADIUS.into());

    let removed_chunks = chunk_manager
        .chunks
        .extract_if(|pos, _| !chunk_rect.contains(pos));

    let mut count = 0;
    for (_, chunk) in removed_chunks {
        count += 1;
        commands.entity(chunk).despawn();
    }
    info!("Despawned {} chunks", count);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ChunkRect {
    from_x: ChunkUnit,
    to_x: ChunkUnit,
    from_y: ChunkUnit,
    to_y: ChunkUnit,
}

impl ChunkRect {
    fn from_circle_outside(center: ChunkPosition3, chunk_radius: i32) -> Self {
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
) {
    if !center.is_changed() {
        return;
    }

    let center = center.center.to_ivec().xz();
    let render_radius_squared = CHUNK_VISIBILITY_RADIUS as i32 * CHUNK_VISIBILITY_RADIUS as i32;

    for (chunk_entity, chunk_position) in chunks.iter() {
        let distance_squared = center.distance_squared(chunk_position.to_ivec().xz());
        if distance_squared > render_radius_squared {
            continue;
        }
        let chunk_translation = chunk_position.to_2d().to_world_pos();
        let mesh =
            utils::primitives::create_subdivided_plane(CHUNK_SUBDIVISIONS, CHUNK_SIZE, |x, y| {
                let pos = chunk_translation + vec2(x, y);
                let height = utils::noise::value_noise_2d(pos / 100.0);
                (height.value * 10.0, height.get_normal().into())
            });
        let mesh_handle = asset_server.add(mesh);
        let material_handle = asset_server.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.0, 0.0, 0.1),
            ..default()
        });

        commands.entity(chunk_entity).insert(PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            transform: Transform::from_translation(chunk_position.to_world_pos()),
            ..default()
        });
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
struct MapMaterialExtension {}

impl MaterialExtension for MapMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        "shaders/map.wgsl".into()
    }
}

fn spawn_map_shader(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mesh = asset_server.add(utils::primitives::create_subdivided_plane(
        CHUNK_SUBDIVISIONS * CHUNK_SPAWN_RADIUS as u32 * 2,
        CHUNK_SIZE * CHUNK_SPAWN_RADIUS as f32 * 2.0,
        |x, y| (0.0, [0.0, 1.0, 0.0]),
    ));

    let material = asset_server.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: tailwind::GREEN_700.into(),
            ..default()
        },
        extension: MapMaterialExtension {},
    });
    commands.spawn(MaterialMeshBundle {
        mesh,
        material,
        ..default()
    });
}
