use crate::prelude::*;
use bevy::{
    ecs::system::SystemState,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{self, RenderGraph},
        render_resource::{ComputePassDescriptor, PipelineCache, ShaderStages, ShaderType},
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
    utils::hashbrown::HashMap,
};
use utils::BindLayoutBuilder;

#[derive(Debug, ShaderType, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct MapPosition {
    position: Vec3,
}

#[derive(Debug, ShaderType, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
#[repr(C)]
struct MapPointData {
    pub height: f32,
}

impl MapPosition {
    pub fn new(position: Vec3) -> Self {
        Self { position }
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

#[derive(Component)]
pub struct TestEntity;

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        insert_readback_channel(app, RenderBufferKey::MapPointData);

        app.add_plugins(ExtractResourcePlugin::<MapPointRequest>::default())
            .insert_resource(MapPointRequest::default())
            .add_systems(PreUpdate, receive)
            .add_systems(Update, test);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, prepare_bind_group.in_set(RenderSet::Prepare));
        render_app
            .world_mut()
            .resource_mut::<RenderGraph>()
            .add_node(MapNodeLabel, MapNode::new());
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

fn test(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    entities_query: Query<(Entity, &Name, Option<&MapPointResponse>), With<TestEntity>>,
    mut requests: ResMut<MapPointRequest>,
) {
    let entities = entities_query.iter().collect_vec();
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ];

    for (i, key) in digits.into_iter().enumerate() {
        if keys.just_pressed(key) {
            println!("\x1B[2J\x1B[1;1H"); // ANSI escape codes to clear terminal screen
            if let Some((entity, _, __)) = entities.get(i) {
                requests.send_request(
                    entity,
                    vec![MapPosition::new(Vec3::new(i as f32, i as f32, i as f32))],
                );
                requests.send_request(
                    entity,
                    vec![MapPosition::new(Vec3::new(i as f32, i as f32, i as f32))],
                );
            } else {
                commands.spawn((TestEntity, Name::from(format!("tester {}", i))));
            }
        }
    }

    if keys.just_pressed(KeyCode::Space) {
        for (_, name, response) in entities_query.iter() {
            if let Some(response) = response {
                info!("{}: {:?}", name, response);
            } else {
                warn!("{}: no response", name);
            }
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
    let Some(response) = wgsl
        .try_receive_vec::<MapPointData>(RenderBufferKey::MapPointData)
        .and_then(|v| {
            if v.len() == map_point_requests.len() {
                Some(v)
            } else {
                None
            }
        })
    else {
        return;
    };

    // info!(
    //     "\n{} requests: {:?}\n{} responses: {:?}",
    //     map_point_requests.positions.len(),
    //     map_point_requests.positions,
    //     response.len(),
    //     response
    // );

    let mut hash_map: HashMap<Entity, Vec<MapPointData>> = HashMap::with_capacity(response.len());
    for (entity, height) in map_point_requests.entities.iter().zip(response.iter()) {
        if hash_map.contains_key(entity) {
            hash_map.get_mut(entity).unwrap().push(*height);
        } else {
            hash_map.insert(*entity, vec![*height]);
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

struct MapNode {
    requests_count: usize,
}

impl MapNode {
    fn new() -> Self {
        Self { requests_count: 0 }
    }
}

impl render_graph::Node for MapNode {
    fn update(&mut self, world: &mut World) {
        let requests_count = world.resource::<MapPointRequest>().positions.len();
        self.requests_count = requests_count;

        if requests_count > 0 {
            let wgsl = &mut world.resource_mut::<RenderStateRender>();
            let Some(buffer) = wgsl.get_buffer_readback_mut(&RenderBufferKey::MapPointData) else {
                return;
            };
            buffer.mark_changed();
        }
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if self.requests_count == 0 {
            return Ok(());
        }

        {
            let wgsl = world.resource::<RenderStateRender>();
            let pipeline_cache = world.resource::<PipelineCache>();
            let Some(pipeline) = wgsl
                .get_pipeline(&RenderPipelineKey::MapPipeline)
                .and_then(|p| pipeline_cache.get_compute_pipeline(*p))
            else {
                return Ok(());
            };

            let Some(bind_group) = wgsl.get_bind_group(&RenderBindGroupKey::MapBindGroup) else {
                return Ok(());
            };

            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Map compute pass"),
                        ..default()
                    });

            pass.set_bind_group(0, bind_group, &[]);
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(self.requests_count as u32, 1, 1);
            drop(pass);

            // Copy the gpu accessible buffer to the cpu accessible buffer
            let encoder = render_context.command_encoder();
            info!("copying buffer");
            wgsl.copy_to_readback_buffer(encoder, &RenderBufferKey::MapPointData);

            Ok(())
        }
    }
}
