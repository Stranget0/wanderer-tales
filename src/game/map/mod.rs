use crate::prelude::*;

use bevy::{
    ecs::system::SystemState,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{self, RenderGraph},
        render_resource::{PipelineCache, ShaderStages, ShaderType},
        renderer::RenderDevice,
        Render, RenderApp, RenderSet,
    },
    utils::hashbrown::HashMap,
};
use utils::{BindLayoutBuilder, RenderBurritoNodeTrait, RenderBurritoPassTrait};

const CHUNK_SUBDIVISIONS: u32 = 32;

#[derive(Debug, ShaderType, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct MapPosition {
    chunk_pos: IVec2,
}

#[derive(Debug, ShaderType, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct MapPointData {
    pub height: [f32; CHUNK_SUBDIVISIONS as usize],
}

impl MapPosition {
    pub fn new(chunk_pos: ChunkPos) -> Self {
        Self {
            chunk_pos: chunk_pos.0,
        }
    }
}

#[derive(Resource, ExtractResource, Clone, Debug, Default)]
struct MapPointRequest {
    positions: Vec<MapPosition>,
    entities: Vec<Entity>,
}

impl MapPointRequest {
    pub fn send_request(&mut self, entity: &Entity, positions: Vec<MapPosition>) {
        self.entities
            .extend(std::iter::repeat(*entity).take(positions.len()));
        self.positions.extend(positions);
        assert_eq!(self.positions.len(), self.entities.len());
    }

    pub fn clear(&mut self) {
        info!("clearing ALL request");
        self.positions.clear();
        self.entities.clear();
        // TODO: remove allocations?
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

#[derive(Component, Clone, Debug, Default)]
struct MapPointResponse {
    heights: Vec<MapPointData>,
}

impl MapPointResponse {
    pub fn new(heights: Vec<MapPointData>) -> Self {
        Self { heights }
    }
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        insert_readback_channel(app, RenderBufferKey::MapPointData);

        app.add_plugins(ExtractResourcePlugin::<MapPointRequest>::default())
            .insert_resource(MapPointRequest::default())
            .add_systems(PreUpdate, receive)
            .add_systems(Update, (spawn_chunks, despawn_chunks));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, prepare_bind_group.in_set(RenderSet::Prepare));
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(MapNodeLabel, RenderNode::new(MapNode));
    }

    fn finish(&self, app: &mut App) {
        let world = app.sub_app_mut(RenderApp).world_mut();

        let mut system_state: SystemState<(
            ResMut<RenderStateRender>,
            Res<RenderDevice>,
            Res<PipelineCache>,
            Res<AssetServer>,
        )> = SystemState::new(world);

        let (mut wgsl, render_device, pipeline_cache, asset_server) = system_state.get_mut(world);

        wgsl.insert_layout(
            RenderLayoutKey::MapLayout,
            BindLayoutBuilder::new(
                render_device.as_ref(),
                RenderLayoutKey::MapLayout,
                ShaderStages::COMPUTE,
            )
            .with_storage_slot::<Vec<MapPosition>>()
            .with_storage_slot::<Vec<MapPointData>>()
            .build(),
        );

        let pipeline_id = pipeline_cache.queue_compute_pipeline(
            wgsl.builder_pipeline(
                RenderPipelineKey::MapPipeline,
                asset_server.load("shaders/map.wgsl"),
                "main",
            )
            .with_layout(&RenderLayoutKey::MapLayout)
            .build(),
        );

        wgsl.insert_pipeline(RenderPipelineKey::MapPipeline, pipeline_id);
    }
}

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct ChunkPos(pub IVec2);
impl ChunkPos {
    fn new(x: i32, y: i32) -> Self {
        Self(IVec2::new(x, y))
    }
    fn from_world_pos(position: Vec2) -> Self {
        let x = (position.x / CHUNK_SIZE as f32) as i32;
        let y = (position.y / CHUNK_SIZE as f32) as i32;
        Self(IVec2::new(x, y))
    }

    fn to_world_pos(&self) -> Vec2 {
        Vec2::new(
            self.0.x as f32 * CHUNK_SIZE as f32,
            self.0.y as f32 * CHUNK_SIZE as f32,
        )
    }
}
impl std::fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}x{})", self.0.x, self.0.y)
    }
}

#[derive(Resource, Default)]
struct ChunkManager {
    chunks: HashMap<ChunkPos, Entity>,
}
impl ChunkManager {
    fn handle_position_add(&mut self, commands: &mut Commands, position: Vec3) {
        let camera_pos = ChunkPos::from_world_pos(position.xz());
        let mut new_chunks = Vec::new();
        for i in -CHUNK_RADIUS..CHUNK_RADIUS {
            for j in -CHUNK_RADIUS..CHUNK_RADIUS {
                let x = camera_pos.0.x + i;
                let y = camera_pos.0.y + j;
                let chunk_pos = ChunkPos::new(x, y);
                if self.chunks.contains_key(&chunk_pos) {
                    continue;
                }
                new_chunks.push(chunk_pos);
            }
        }

        let bundles = new_chunks
            .iter()
            .map(|chunk_pos| (Name::from(format!("chunk {}", chunk_pos))))
            .map(|bundle| commands.spawn(bundle).id())
            .collect_vec()
            .into_iter()
            .zip_eq(new_chunks);

        for (entity, pos) in bundles {
            self.chunks.insert(pos, entity);
        }
    }
    fn handle_position_sub(&mut self, commands: &mut Commands, position: Vec3) {
        let camera_pos = ChunkPos::from_world_pos(position.xz());
        let mut removed_chunks = Vec::new();
        for pos in self.chunks.keys() {
            let from = Vec2::new(pos.0.x as f32, pos.0.y as f32);
            let to = Vec2::new(camera_pos.0.x as f32, camera_pos.0.y as f32);
            if from.distance(to) > CHUNK_RADIUS as f32 {
                removed_chunks.push(pos.to_owned());
            }
        }

        for key in removed_chunks {
            let entity = self.chunks.remove(&key).unwrap();
            commands.entity(entity).despawn();
        }
    }
}

const CHUNK_SIZE: i32 = 16;
const CHUNK_RADIUS: i32 = 4;

fn spawn_chunks(
    mut commands: Commands,
    mut chunks: ResMut<ChunkManager>,
    camera_query: Query<(&Transform, &Camera), With<Camera3d>>,
) {
    let Some(camera_pos) = camera_query
        .iter()
        .max_by_key(|(_, camera)| camera.order)
        .map(|(transform, _)| transform.translation)
    else {
        return;
    };

    chunks.handle_position_add(&mut commands, camera_pos);
}

fn despawn_chunks(
    mut commands: Commands,
    mut chunks: ResMut<ChunkManager>,
    camera_query: Query<(&Transform, &Camera), With<Camera3d>>,
) {
    let Some(camera_pos) = camera_query
        .iter()
        .max_by_key(|(_, camera)| camera.order)
        .map(|(transform, _)| transform.translation)
    else {
        return;
    };

    chunks.handle_position_sub(&mut commands, camera_pos);
}

fn request_render_chunks(
    mut commands: Commands,
    chunks: Query<(Entity, Ref<ChunkPos>)>,
    map_point_requests: ResMut<MapPointRequest>,
) {
    for (entity, pos) in chunks.iter() {
        if !pos.is_changed() && !pos.is_added() {
            continue;
        }
    }
}

fn prepare_bind_group(
    render_device: Res<RenderDevice>,
    mut wgsl: ResMut<RenderStateRender>,
    requested_points_query: Res<MapPointRequest>,
) {
    let points = &requested_points_query.positions;
    if points.is_empty() {
        return;
    }

    info!("preparing bind group {points:?}");
    wgsl.start_create_buffers(render_device.as_ref())
        .create_storage_rw(RenderBufferKey::MapPointPositions, points)
        .create_empty_storage_readable(
            RenderBufferKey::MapPointData,
            (size_of::<MapPointData>() * points.len()) as u64,
        );

    wgsl.create_bind_group(
        RenderBindGroupKey::MapBindGroup,
        render_device.as_ref(),
        RenderLayoutKey::MapLayout,
        &[
            RenderBufferKey::MapPointPositions,
            RenderBufferKey::MapPointData,
        ],
    );
}

fn receive(
    mut commands: Commands,
    wgsl: Res<RenderStateMain>,
    mut map_point_requests: ResMut<MapPointRequest>,
) {
    let Some(response) = wgsl.try_receive_vec::<MapPointData>(RenderBufferKey::MapPointData) else {
        return;
    };

    info!(
        "\n{} requests: {:?}\n{} responses: {:?}",
        map_point_requests.positions.len(),
        map_point_requests.positions,
        response.len(),
        response
    );

    let mut hash_map: HashMap<Entity, Vec<MapPointData>> = HashMap::with_capacity(response.len());
    for (index, height) in response.iter().enumerate() {
        if index >= map_point_requests.positions.len() {
            warn!("received more responses than requests");
            break;
        }
        let entity = &map_point_requests.entities[index];
        match hash_map.get_mut(entity) {
            Some(results) => {
                results.push(*height);
            }
            None => {
                hash_map.insert(*entity, vec![*height]);
            }
        }
    }

    for (entity, response) in hash_map.into_iter() {
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(MapPointResponse::new(response));
        }
    }

    map_point_requests.clear();
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, render_graph::RenderLabel)]
struct MapNodeLabel;

enum MapNodePass {
    HeightPass,
}

struct MapNode;

impl RenderBurritoNodeTrait<RenderBufferKey, RenderBindGroupKey, RenderPipelineKey, MapNodePass>
    for MapNode
{
    fn passes(&self) -> &[MapNodePass] {
        &[MapNodePass::HeightPass]
    }
    fn label(&self) -> &str {
        "MapNode"
    }
    fn staging_buffers(&self, _: &World) -> &[RenderBufferKey] {
        &[RenderBufferKey::MapPointData]
    }

    fn should_run(&self, world: &World) -> bool {
        !world.resource::<MapPointRequest>().positions.is_empty()
    }
}

impl RenderBurritoPassTrait<RenderBufferKey, RenderBindGroupKey, RenderPipelineKey>
    for MapNodePass
{
    fn pipeline_key(&self, _: &World) -> &RenderPipelineKey {
        &RenderPipelineKey::MapPipeline
    }

    fn bind_group_key(&self, _: &World) -> &RenderBindGroupKey {
        &RenderBindGroupKey::MapBindGroup
    }

    fn workgroup_size(&self, world: &World) -> [u32; 3] {
        let x = world.resource::<MapPointRequest>().positions.len() as u32;
        [x, 1, 1]
    }
}
