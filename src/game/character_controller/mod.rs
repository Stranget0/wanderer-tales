#[cfg(feature = "dev")]
pub(crate) mod devtools;
mod movement;

use crate::prelude::*;
pub(crate) use movement::*;

pub(super) fn plugin(app: &mut bevy::prelude::App) {
    app.add_plugins(movement::plugin);
}

pub(crate) fn spawn_character(
    commands: &mut Commands,
    asset_server: &AssetServer,
    model: &CharacterModel,
    with_bundle: impl Bundle,
) {
    let mesh_entity = commands
        .spawn(model.get_or_load(asset_server))
        .insert(SpatialBundle::default())
        .insert(*model)
        .id();

    commands
        .spawn((
            StateScoped(GameState::Playing),
            CharacterControllerBundle::capsule(model.height(), model.radius()),
            with_bundle,
        ))
        .add_child(mesh_entity);
}
