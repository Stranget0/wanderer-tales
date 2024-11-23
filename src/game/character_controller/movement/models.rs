use super::*;
use crate::prelude::*;
use avian3d::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy_tnua::controller::TnuaController;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        prepare_models_of_controllers.run_if(in_state(GameState::Playing)),
    );
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharacterModel {
    KnightPlaceholder,
}

fn prepare_models_of_controllers(
    mut commands: Commands,
    controllers: Query<(Entity, &Transform, &FloatHeight), (Added<TnuaController>, With<Collider>)>,
    mut transforms: Query<&mut Transform, Without<Collider>>,
    children_query: Query<&Children>,
    meshes: Query<&Handle<Mesh>>,
) {
    for (entity, transform, float_height) in controllers.iter() {
        // Shift models down because Tnua will make controllers float,
        // but our models definitely should not be floating!
        let offset = (float_height.0 / transform.scale.y) * 2.;
        let children = children_query.get(entity).unwrap();
        for child in children.iter() {
            if let Ok(mut model_transform) = transforms.get_mut(*child) {
                model_transform.translation.y -= offset;
            }
        }

        // Frustum culling is erroneous for animated models because the AABB can be too small
        for entity in children_query.iter_descendants(entity) {
            if meshes.contains(entity) {
                commands.entity(entity).insert(NoFrustumCulling);
            }
        }
    }
}

impl CharacterModel {
    pub fn path(self) -> &'static str {
        match self {
            CharacterModel::KnightPlaceholder => "meshes/soldier_placeholder.glb",
        }
    }

    pub fn load(self, asset_server: &AssetServer) -> Handle<Scene> {
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(self.path()))
    }

    pub fn get(&self, asset_server: &AssetServer) -> Option<Handle<Scene>> {
        asset_server.get_handle(self.path())
    }

    pub fn get_or_load(&self, asset_server: &AssetServer) -> Handle<Scene> {
        self.get(asset_server)
            .unwrap_or_else(|| self.load(asset_server))
    }

    pub fn height(&self) -> f32 {
        match self {
            CharacterModel::KnightPlaceholder => 1.72,
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            CharacterModel::KnightPlaceholder => 0.4,
        }
    }
}
