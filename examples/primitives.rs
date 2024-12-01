use camera::*;
use wanderer_tales::prelude::*;

#[derive(Component)]
struct MainObject;

#[derive(Resource)]
struct Subdivisions(u32);

#[derive(Resource)]
struct Size(f32);

#[derive(Resource, Default)]
struct ShouldUpdate(bool);

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
enum UpdateState {
    #[default]
    Replace,
    Respawn,
    Insert,
}

fn main() {
    App::new()
        .add_plugins((
            wanderer_tales::default_plugins,
            minimal_dev_tools_plugin,
            camera::plugin,
            actions::plugin,
        ))
        .insert_state(GameState::Playing)
        .init_state::<UpdateState>()
        .insert_state(UpdateState::Replace)
        .insert_resource(ShouldUpdate(false))
        .insert_resource(Subdivisions(2))
        .insert_resource(Size(100.0))
        .add_systems(Startup, (spawn_random_plane, setup_lightning))
        .add_systems(
            Update,
            (
                mark_should_update.run_if(input_just_pressed(KeyCode::Space)),
                switch_state.run_if(input_just_pressed(KeyCode::KeyC)),
                replace_mesh.run_if(in_state(UpdateState::Replace).and_then(should_update)),
                insert_mesh.run_if(in_state(UpdateState::Insert).and_then(should_update)),
                (despawn_entities::<MainObject>, spawn_random_plane)
                    .chain()
                    .run_if(in_state(UpdateState::Respawn).and_then(should_update)),
                unmark_should_update.run_if(should_update),
            )
                .chain(),
        )
        .run();
}

fn setup_lightning(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::srgb(1.0, 1.0, 1.0),
            illuminance: 80.,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.)),
        ..default()
    });
}

fn should_update(should_update: Res<ShouldUpdate>) -> bool {
    should_update.0
}

fn unmark_should_update(mut should_update: ResMut<ShouldUpdate>) {
    should_update.0 = false;
}
fn mark_should_update(mut should_update: ResMut<ShouldUpdate>) {
    should_update.0 = true;
}

fn switch_state(state: Res<State<UpdateState>>, mut next_state: ResMut<NextState<UpdateState>>) {
    match state.get() {
        UpdateState::Replace => {
            info!("switching to respawning entity");
            next_state.set(UpdateState::Respawn)
        }
        UpdateState::Respawn => {
            info!("switching to replacing mesh");
            next_state.set(UpdateState::Replace)
        }
        UpdateState::Insert => {
            info!("switching to replacing mesh");
            next_state.set(UpdateState::Replace)
        }
    }
}

fn replace_mesh(
    mut query: Query<&mut Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    subdivisions: Res<Subdivisions>,
    size: Res<Size>,
) {
    for mut mesh_handle in query.iter_mut() {
        let mesh = random_mesh(subdivisions.0, size.0);
        *mesh_handle = meshes.add(mesh);
    }
}

fn insert_mesh(
    mut commands: Commands,
    query: Query<Entity, With<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    subdivisions: Res<Subdivisions>,
    size: Res<Size>,
) {
    for entity in query.iter() {
        let mesh = random_mesh(subdivisions.0, size.0);
        commands.entity(entity).insert(meshes.add(mesh));
    }
}

fn spawn_random_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    subdivisions: Res<Subdivisions>,
    size: Res<Size>,
) {
    let mesh = random_mesh(subdivisions.0, size.0);
    commands.spawn((
        Name::new("Plane"),
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                ..default()
            }),
            ..default()
        },
        CameraFocus,
        MainObject,
        CameraOrbitTarget { zoom: 10.0 },
        CameraRotationSpeed(45.0_f32.to_radians()),
        CameraRotationController::default(),
        CameraAction::input_bundle(),
    ));
}

fn random_mesh(subdivisions: u32, size: f32) -> Mesh {
    let offset_x = rand::random::<f32>() * 1000.0;
    let offset_y = rand::random::<f32>() * 1000.0;

    utils::primitives::create_subdivided_plane_smooth(subdivisions, size, |x, y| {
        let pos = vec2(x + offset_x, y + offset_y) / 10.0;
        let value = (pos.x.sin() + pos.y.cos()) * 10.0;
        (value, [0.0, 0.0, 0.0])
    })
}
