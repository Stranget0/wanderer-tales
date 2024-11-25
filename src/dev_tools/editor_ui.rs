use crate::{game, prelude::*};
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::window::PrimaryWindow;
use std::any::TypeId;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>()
        .init_resource::<DevUIEnabled>()
        .add_plugins((
            bevy_inspector_egui::DefaultInspectorConfigPlugin,
            bevy_inspector_egui::bevy_egui::EguiPlugin,
        ))
        .add_systems(
            Update,
            (
                toggle_dev_ui
                    .run_if(input_just_pressed(KeyCode::F10))
                    .in_set(GameSet::RecordInput),
                sync_control_locks
                    .run_if(resource_changed::<DevUIEnabled>)
                    .in_set(GameSet::Update),
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

pub trait EditorDock {
    fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui);
}

#[derive(Resource, Default)]
struct DevUIEnabled(pub bool);

#[derive(Resource)]
struct EditorState {
    pub(crate) state: egui_dock::DockState<EditorView>,
    pub(crate) selected_entities: bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
    pub(crate) selection: InspectorSelection,
    pub(crate) views_to_open: hashbrown::HashSet<EditorView>,
    pub(crate) windows_to_open: hashbrown::HashSet<WindowView>,
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

struct EditorTabs<'a> {
    pub world: &'a mut World,
    pub(crate) selected_entities:
        &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
    pub(crate) selection: &'a mut InspectorSelection,
    pub(crate) views_to_open: &'a mut hashbrown::HashSet<EditorView>,
    pub(crate) windows_to_open: &'a mut hashbrown::HashSet<WindowView>,
}

struct ResourceUI<'a> {
    pub(crate) selection: &'a mut InspectorSelection,
    pub(crate) views_to_open: &'a mut hashbrown::HashSet<EditorView>,
}

struct AssetUI<'a> {
    pub(crate) selection: &'a mut InspectorSelection,
    pub(crate) views_to_open: &'a mut hashbrown::HashSet<EditorView>,
}

struct HierarchyUI<'a> {
    pub(crate) selection: &'a mut InspectorSelection,
    pub(crate) selected_entities:
        &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
    pub(crate) views_to_open: &'a mut hashbrown::HashSet<EditorView>,
}

struct InspectorUI<'a> {
    pub(crate) selection: &'a mut InspectorSelection,
    pub(crate) selected_entities:
        &'a mut bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities,
}

struct FlagsUI;

struct GameUI;

fn dev_ui_enabled(enabled: Res<DevUIEnabled>) -> bool {
    enabled.0
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum EditorView {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
    TerrainGen,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum WindowView {
    Flags,
}

impl EditorView {
    pub(crate) fn clear_background(&self) -> bool {
        !matches!(self, EditorView::GameView)
    }

    pub(crate) fn closeable(&self) -> bool {
        !matches!(self, EditorView::GameView)
    }
    pub(crate) fn handle_split(&self, state: &mut egui_dock::DockState<EditorView>) {
        if state.find_tab(self).is_some() {
            return;
        }

        match self {
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
            _ => {}
        }
    }

    pub(crate) fn as_ui<'a>(
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
            EditorView::TerrainGen => Box::new(game::map::devtools::EditorTerrain {}),
            EditorView::GameView => Box::new(GameUI {}),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            EditorView::Hierarchy => "Hierarchy",
            EditorView::Resources => "Resources",
            EditorView::Assets => "Assets",
            EditorView::Inspector => "Inspector",
            EditorView::TerrainGen => "Terrain Gen",
            EditorView::GameView => "Game View",
        }
    }
}

impl WindowView {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            WindowView::Flags => "Flags",
        }
    }

    fn as_ui(&self) -> impl EditorDock {
        match self {
            WindowView::Flags => FlagsUI,
        }
    }
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
            windows_to_open: hashbrown::HashSet::default(),
        }
    }

    pub(crate) fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = EditorTabs {
            world,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            views_to_open: &mut self.views_to_open,
            windows_to_open: &mut self.windows_to_open,
        };

        egui_dock::DockArea::new(&mut self.state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show_add_buttons(true)
            .show_add_popup(true)
            .show(ctx, &mut tab_viewer);

        self.views_to_open.drain().for_each(|tab| {
            tab.handle_split(&mut self.state);
        });

        for window in self.windows_to_open.clone().into_iter() {
            let mut opened = true;
            let center = ctx.screen_rect().center() - egui::pos2(100.0, 100.0);
            let size = egui::vec2(200.0, 400.0);

            egui::Window::new(window.label())
                .open(&mut opened)
                .vscroll(true)
                .fade_in(true)
                .fade_out(true)
                .default_pos(center.to_pos2())
                .fixed_size(size)
                .show(ctx, |ui| window.as_ui().ui(world, ui));

            if !opened {
                self.windows_to_open.retain(|w| w != &window);
            }
        }
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
        let mut ctx = egui_context.get_mut();
        ui_state.ui(world, ctx)
    });
}

fn toggle_dev_ui(mut dev_ui_enabled: ResMut<DevUIEnabled>) {
    dev_ui_enabled.0 = !dev_ui_enabled.0;
}

fn sync_control_locks(
    mut control_locks: ResMut<game::ControlLocks>,
    dev_ui_enabled: Res<DevUIEnabled>,
) {
    if dev_ui_enabled.0 {
        control_locks.0.insert(game::ControlLock::EditorUI);
    } else {
        control_locks.0.remove(&game::ControlLock::EditorUI);
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
        let window_openable = vec![WindowView::Flags];
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

        for view in window_openable {
            if ui.button(view.label()).clicked() {
                self.windows_to_open.insert(view);
            }
        }

        for view in other_openable {
            if ui.button(view.label()).clicked() {
                self.views_to_open.insert(view);
            }
        }
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
                *self.selection = InspectorSelection::Resource(type_id, resource_name.to_string());
            }
        }
    }
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
                entities => bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                    world, entities, ui,
                ),
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

impl EditorDock for GameUI {
    fn ui(&mut self, _world: &mut World, _ui: &mut bevy_inspector_egui::egui::Ui) {}
}

impl EditorDock for FlagsUI {
    fn ui(&mut self, world: &mut World, ui: &mut bevy_inspector_egui::egui::Ui) {
        let mut flags_map = world.get_resource_mut::<DebugFlags>().unwrap();
        let mut has_changed = flags_map.has_changed;
        for (group, flags) in flags_map.groups.clone().into_iter() {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(group, |ui| {
                    ui_group(flags, &mut flags_map, ui, &mut has_changed);
                });
            });
        }

        flags_map.has_changed = has_changed;
    }
}

fn ui_group(
    flags: Vec<&str>,
    flags_map: &mut Mut<'_, DebugFlags>,
    ui: &mut egui::Ui,
    has_changed: &mut bool,
) {
    for flag in flags {
        let value = flags_map.flags.get_mut(flag).unwrap();
        if ui.checkbox(value, flag).changed() {
            *has_changed = true;
        }
    }
}
