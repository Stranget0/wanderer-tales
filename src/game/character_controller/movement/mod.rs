mod data;
mod models;

use super::CollisionLayer;
use crate::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
pub(crate) use data::*;

// This plugin communicates with the Tnua character controller by propagating settings found in
/// the control components [`Walk`] and [`Jump`]. It also controls a state machine to determine which animations to play.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        TnuaControllerPlugin::default(),
        TnuaAvian3dPlugin::default(),
    ))
    .add_plugins((data::plugin, models::plugin))
    .add_systems(
        Update,
        (apply_looking_at, apply_jumping, apply_walking)
            .chain()
            .in_set(GameSet::Update),
    );
}

fn apply_walking(
    mut character_query: Query<(
        &mut TnuaController,
        &mut Walk,
        Option<&mut Sprinting>,
        &FloatHeight,
        &RotationSpeed,
    )>,
) {
    for (mut controller, mut walking, mut sprinting, float_height, rotation_speed) in
        character_query.iter_mut()
    {
        if walking.direction.is_some() {
            controller.walk(
                &mut walking,
                sprinting.as_deref_mut(),
                float_height,
                rotation_speed,
            );
        }
    }
}

fn apply_jumping(mut character_query: Query<(&mut TnuaController, &mut Jump)>) {
    for (mut controller, mut jump) in &mut character_query {
        if jump.requested {
            controller.jump(&mut jump);
        }
    }
}

fn apply_looking_at(
    mut character_query: Query<(
        &mut TnuaController,
        &LookingAt,
        &Walk,
        &Jump,
        &FloatHeight,
        &RotationSpeed,
    )>,
) {
    for (mut controller, looking_at, walking, jumping, float_height, rotation_speed) in
        &mut character_query
    {
        if looking_at.is_some() && !jumping.requested && walking.direction.is_none() {
            controller.look_at(walking, jumping, looking_at, float_height, rotation_speed);
        }
    }
}
