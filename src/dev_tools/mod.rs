//! Development tools for the game. This plugin is only enabled in dev builds.
pub mod data;

pub use data::*;

#[cfg(feature = "dev")]
pub(super) use plugin::plugin;

#[cfg(feature = "dev")]
pub mod plugin {
    use std::any::TypeId;

    use super::*;
    use crate::{game, prelude::*};
    use crate::{game::CameraOrbit, screen::Screen};
    use bevy::asset::{ReflectAsset, UntypedAssetId};
    use bevy::pbr::ExtendedMaterial;
    use bevy::window::PrimaryWindow;
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

    #[derive(Resource, Default)]
    pub struct DevUIEnabled(pub bool);

    pub fn dev_ui_enabled(enabled: Res<DevUIEnabled>) -> bool {
        enabled.0
    }

    pub fn plugin(app: &mut App) {
        app
            .add_plugins(
                (
                    MaterialPlugin::<
                        ExtendedMaterial<StandardMaterial, DebugNormalsMaterialExtension>,
                    >::default(),
                    game::devtools::map_devtools_plugin,
                    bevy_inspector_egui::DefaultInspectorConfigPlugin,
                    bevy_inspector_egui::bevy_egui::EguiPlugin,
                ),
            )
            .init_resource::<DevUIEnabled>()
            .insert_resource(WireframeConfig {
                default_color: Color::srgb(1.0, 1.0, 1.0),
                ..Default::default()
            })
            .insert_resource(EditorState::new())
            // Print state transitions in dev builds
            .add_systems(Update, log_transitions::<Screen>)
            .add_systems(
                Update,
                (
                    // add_forward_gizmo,
                    // add_world_gizmo,
                    toggle_dev_ui
                        .run_if(input_just_pressed(KeyCode::F10))
                        .in_set(AppSet::RecordInput),
                    sync_camera_locks
                        .run_if(resource_changed::<DevUIEnabled>)
                        .in_set(AppSet::Update),
                    // add_camera_debug,
                    log_shader_load,
                    // draw_debug_normals,
                    // game::devtools::change_map_seed.run_if(input_just_pressed(KeyCode::Numpad0)),
                    // toggle_debug_normals.run_if(input_just_pressed(KeyCode::Numpad1)),
                    // game::devtools::toggle_debug_chunks
                    //     .run_if(input_just_pressed(KeyCode::Numpad2)),
                ),
            )
            .add_systems(
                PostUpdate,
                show_ui_system
                    .run_if(dev_ui_enabled)
                    .before(bevy_inspector_egui::bevy_egui::EguiSet::ProcessOutput)
                    .before(bevy_inspector_egui::bevy_egui::systems::end_pass_system)
                    .before(bevy::transform::TransformSystem::TransformPropagate),
            );
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EditorView {
        GameView,
        Hierarchy,
        Resources,
        Assets,
        Inspector,
        TerrainGen,
    }

    impl EditorView {
        fn clear_background(&self) -> bool {
            match self {
                EditorView::GameView => false,
                _ => true,
            }
        }

        fn closeable(&self) -> bool {
            match self {
                EditorView::GameView => false,
                _ => true,
            }
        }
        fn handle_split(&self, state: &mut egui_dock::DockState<EditorView>) {
            if state.find_tab(self).is_some() {
                return;
            }

            match self {
                EditorView::GameView => {}
                EditorView::Hierarchy => {
                    let tree = state.main_surface_mut();
                    tree.split_left(egui_dock::NodeIndex::root(), 0.2, vec![*self]);
                }
                EditorView::Resources => {
                    let tree = state.main_surface_mut();
                    tree.split_below(egui_dock::NodeIndex::root(), 0.7, vec![*self]);
                }
                EditorView::Assets => {
                    let tree = state.main_surface_mut();
                    tree.split_below(egui_dock::NodeIndex::root(), 0.7, vec![*self]);
                }
                EditorView::Inspector => {
                    let tree = state.main_surface_mut();
                    tree.split_right(egui_dock::NodeIndex::root(), 0.8, vec![*self]);
                }
                EditorView::TerrainGen => {
                    let tree = state.main_surface_mut();
                    tree.split_right(egui_dock::NodeIndex::root(), 0.8, vec![*self]);
                }
            }
        }

        fn as_ui<'a>(
            &self,
            views_to_open: &'a mut hashbrown::HashSet<EditorView>,
            selection: &'a mut InspectorSelection,
            selected_entities: &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
        ) -> Box<dyn EditorDock + 'a> {
            match self {
                EditorView::Hierarchy => Box::new(HierarchyUI {
                    selection,
                    selected_entities,
                    views_to_open,
                }),
                EditorView::Resources => Box::new(ResourceUI {
                    selection,
                    views_to_open,
                }),
                EditorView::Assets => Box::new(AssetUI {
                    selection,
                    views_to_open,
                }),
                EditorView::Inspector => Box::new(InspectorUI {
                    selection,
                    selected_entities,
                }),
                EditorView::TerrainGen => Box::new(game::devtools::EditorTerrain {}),
                EditorView::GameView => Box::new(GameUI {}),
            }
        }

        pub fn label(&self) -> String {
            match self {
                EditorView::Hierarchy => "Hierarchy",
                EditorView::Resources => "Resources",
                EditorView::Assets => "Assets",
                EditorView::Inspector => "Inspector",
                EditorView::TerrainGen => "Terrain Gen",
                EditorView::GameView => "Game View",
            }
            .to_string()
        }
    }

    pub struct EditorTabs<'a> {
        pub world: &'a mut World,
        selected_entities: &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
        selection: &'a mut InspectorSelection,
        views_to_open: &'a mut hashbrown::HashSet<EditorView>,
    }

    #[derive(Eq, PartialEq)]
    enum InspectorSelection {
        Entities,
        Resource(TypeId, String),
        Asset(TypeId, String, UntypedAssetId),
    }

    #[derive(Resource)]
    pub(crate) struct EditorState {
        state: egui_dock::DockState<EditorView>,
        selected_entities: bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
        selection: InspectorSelection,
        views_to_open: hashbrown::HashSet<EditorView>,
    }

    pub trait EditorDock {
        fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui);
    }

    impl EditorState {
        pub fn new() -> Self {
            let state = egui_dock::DockState::new(vec![EditorView::GameView]);

            Self {
                state,
                selected_entities:
                    bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities::default(),
                selection: InspectorSelection::Entities,
                views_to_open: hashbrown::HashSet::default(),
            }
        }

        fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
            let mut tab_viewer = EditorTabs {
                world,
                selected_entities: &mut self.selected_entities,
                selection: &mut self.selection,
                views_to_open: &mut self.views_to_open,
            };
            egui_dock::DockArea::new(&mut self.state)
                .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
                .show_add_buttons(true)
                .show_add_popup(true)
                .show(ctx, &mut tab_viewer);

            self.views_to_open.drain().for_each(|tab| {
                tab.handle_split(&mut self.state);
            });
        }
    }

    impl Default for EditorState {
        fn default() -> Self {
            Self::new()
        }
    }

    fn show_ui_system(world: &mut World) {
        let Ok(egui_context) = world
            .query_filtered::<&mut bevy_inspector_egui::bevy_egui::EguiContext, With<PrimaryWindow>>()
            .get_single(world)
        else {
            return;
        };
        let mut egui_context = egui_context.clone();

        world.resource_scope::<EditorState, _>(|world, mut ui_state| {
            ui_state.ui(world, egui_context.get_mut())
        });
    }

    fn toggle_dev_ui(mut dev_ui_enabled: ResMut<DevUIEnabled>) {
        dev_ui_enabled.0 = !dev_ui_enabled.0;
    }

    fn sync_camera_locks(
        mut camera_locks: ResMut<game::CameraLocks>,
        dev_ui_enabled: Res<DevUIEnabled>,
    ) {
        if dev_ui_enabled.0 {
            camera_locks.0.insert(game::CameraLock::EditorUI);
        } else {
            camera_locks.0.remove(&game::CameraLock::EditorUI);
        }
    }

    impl egui_dock::TabViewer for EditorTabs<'_> {
        type Tab = EditorView;

        fn ui(&mut self, ui: &mut egui_dock::egui::Ui, view: &mut Self::Tab) {
            view.as_ui(self.views_to_open, self.selection, self.selected_entities)
                .ui(self.world, ui);
        }

        fn add_popup(
            &mut self,
            ui: &mut egui::Ui,
            _surface: egui_dock::SurfaceIndex,
            _node: egui_dock::NodeIndex,
        ) {
            ui.set_min_width(120.0);
            ui.style_mut().visuals.button_frame = false;

            let top_openable = vec![EditorView::TerrainGen];
            let other_openable = vec![
                EditorView::Hierarchy,
                EditorView::Resources,
                EditorView::Assets,
                EditorView::Inspector,
            ];

            for view in top_openable {
                if ui.button(view.label()).clicked() {
                    self.views_to_open.insert(view);
                }
            }

            ui.separator();
            ui.spacing();

            ui.group(|ui| {
                for view in other_openable {
                    if ui.button(view.label()).clicked() {
                        self.views_to_open.insert(view);
                    }
                }
            });
        }

        fn clear_background(&self, tab: &Self::Tab) -> bool {
            tab.clear_background()
        }

        fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
            tab.closeable()
        }

        fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
            format!("{window:?}").into()
        }
    }
    struct ResourceUI<'a> {
        selection: &'a mut InspectorSelection,
        views_to_open: &'a mut hashbrown::HashSet<EditorView>,
    }

    impl EditorDock for ResourceUI<'_> {
        fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
            let type_registry = world.resource::<AppTypeRegistry>().0.clone();
            let type_registry = type_registry.read();

            let mut resources: Vec<_> = type_registry
                .iter()
                .filter(|registration| registration.data::<ReflectResource>().is_some())
                .map(|registration| {
                    (
                        registration.type_info().type_path_table().short_path(),
                        registration.type_id(),
                    )
                })
                .collect();
            resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

            for (resource_name, type_id) in resources {
                let selected = match *self.selection {
                    InspectorSelection::Resource(selected, _) => selected == type_id,
                    _ => false,
                };

                if ui.selectable_label(selected, resource_name).clicked() {
                    self.views_to_open.insert(EditorView::Inspector);
                    *self.selection =
                        InspectorSelection::Resource(type_id, resource_name.to_string());
                }
            }
        }
    }

    struct AssetUI<'a> {
        selection: &'a mut InspectorSelection,
        views_to_open: &'a mut hashbrown::HashSet<EditorView>,
    }

    impl EditorDock for AssetUI<'_> {
        fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
            let type_registry = world.resource::<AppTypeRegistry>().0.clone();
            let type_registry = type_registry.read();

            let mut assets: Vec<_> = type_registry
                .iter()
                .filter_map(|registration| {
                    let reflect_asset = registration.data::<ReflectAsset>()?;
                    Some((
                        registration.type_info().type_path_table().short_path(),
                        registration.type_id(),
                        reflect_asset,
                    ))
                })
                .collect();
            assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

            for (asset_name, asset_type_id, reflect_asset) in assets {
                let handles: Vec<_> = reflect_asset.ids(world).collect();

                ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
                    for handle in handles {
                        let selected = match *self.selection {
                            InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                            _ => false,
                        };

                        if ui
                            .selectable_label(selected, format!("{:?}", handle))
                            .clicked()
                        {
                            self.views_to_open.insert(EditorView::Inspector);
                            *self.selection = InspectorSelection::Asset(
                                asset_type_id,
                                asset_name.to_string(),
                                handle,
                            );
                        }
                    }
                });
            }
        }
    }

    struct HierarchyUI<'a> {
        selection: &'a mut InspectorSelection,
        selected_entities: &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
        views_to_open: &'a mut hashbrown::HashSet<EditorView>,
    }

    impl EditorDock for HierarchyUI<'_> {
        fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
            let selected = bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                world,
                ui,
                self.selected_entities,
            );
            if selected {
                self.views_to_open.insert(EditorView::Inspector);
                *self.selection = InspectorSelection::Entities;
            }
        }
    }

    struct InspectorUI<'a> {
        selection: &'a mut InspectorSelection,
        selected_entities: &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
    }

    impl EditorDock for InspectorUI<'_> {
        fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
            let type_registry = world.resource::<AppTypeRegistry>().0.clone();
            let type_registry = type_registry.read();
            // let ui_state = world.resource::<EditorState>();
            // let selection = ui_state.selection;

            match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => bevy_inspector_egui::bevy_inspector::ui_for_entity_with_children(
                        world, entity, ui,
                    ),
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        )
                    }
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector_egui::bevy_inspector::by_type_id::ui_for_resource(
                        world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector_egui::bevy_inspector::by_type_id::ui_for_asset(
                        world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            }
        }
    }

    struct GameUI;

    impl EditorDock for GameUI {
        fn ui(&mut self, _world: &mut World, _ui: &mut bevy_inspector_egui::egui::Ui) {}
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
