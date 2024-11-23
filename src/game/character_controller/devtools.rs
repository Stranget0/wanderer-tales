use bevy_tnua::prelude::{TnuaBuiltinWalk, TnuaController};

use crate::prelude::*;

use super::{LookingAt, Walk};

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, gizmo_walk_direction.in_set(GameSet::Update));
}

fn gizmo_walk_direction(
    mut gizmos: Gizmos,
    controllers: Query<(&Transform, &TnuaController, &Walk, &LookingAt)>,
) {
    for (transform, controller, walk, look_at) in controllers.iter() {
        let Some((walk_basis, walk_basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>()
        else {
            continue;
        };
        let Some(desired_forward) = walk_basis.desired_forward else {
            continue;
        };

        let velocity = walk_basis_state.running_velocity;
        gizmos.arrow(
            transform.translation,
            transform.translation + *desired_forward,
            tailwind::ORANGE_600,
        );

        gizmos.arrow(
            transform.translation,
            transform.translation + velocity,
            tailwind::RED_400,
        );

        gizmos.arrow(
            transform.translation,
            transform.translation + walk.direction.map(|d| *d).unwrap_or_default(),
            tailwind::GRAY_950,
        );

        gizmos.arrow(
            transform.translation,
            transform.translation
                + look_at
                    .horizontal()
                    .direction()
                    .map(|d| *d)
                    .unwrap_or_default(),
            tailwind::GRAY_50,
        );
    }
}
