use super::{LookingAt, Walk};
use crate::prelude::*;
use bevy_tnua::prelude::{TnuaBuiltinWalk, TnuaController};
use debug_flags::*;

enum Flags {
    CharacterControllerGizmo,
}

impl DebugFlagsExt for Flags {
    fn group(&self) -> &'static str {
        "Character controller"
    }
    fn as_str(&self) -> &'static str {
        match self {
            Flags::CharacterControllerGizmo => "Character controller gizmo",
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    register_debug_flags(app, vec![Flags::CharacterControllerGizmo]);

    app.add_systems(
        Update,
        gizmo_walk_direction
            .run_if(debug_flag_enabled(&Flags::CharacterControllerGizmo))
            .in_set(GameSet::PostUpdate),
    );
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
