use crate::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;

/// Configures [`Actionlike`]s, the components that hold all player input.
pub fn plugin(app: &mut App) {
    app.register_type::<PlayerAction>()
        .register_type::<CameraAction>()
        .add_plugins((
            InputManagerPlugin::<PlayerAction>::default(),
            InputManagerPlugin::<CameraAction>::default(),
        ))
        .add_systems(
            Update,
            (
                (
                    disable_actions::<CameraAction>
                        .run_if(actions_dependencies_changed::<CameraAction>()),
                    disable_actions::<PlayerAction>
                        .run_if(actions_dependencies_changed::<PlayerAction>()),
                )
                    .run_if(controls_locked)
                    .in_set(InputManagerSystem::ManualControl),
                (
                    enable_actions::<CameraAction>
                        .run_if(actions_dependencies_changed::<CameraAction>()),
                    enable_actions::<PlayerAction>
                        .run_if(actions_dependencies_changed::<PlayerAction>()),
                )
                    .run_if(not(controls_locked))
                    .in_set(InputManagerSystem::ManualControl),
            ),
        );
}

fn actions_dependencies_changed<A: Actionlike>() -> impl Condition<()> {
    resource_changed::<ControlLocks>.or_else(components_added::<ActionState<A>>)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Actionlike, Reflect, Default)]
pub enum PlayerAction {
    #[default]
    #[actionlike(DualAxis)]
    Move,
    Sprint,
    Jump,
    Interact,
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum CameraAction {
    #[actionlike(DualAxis)]
    Orbit,
    #[actionlike(Axis)]
    Zoom,
}

impl PlayerAction {
    pub fn input_bundle() -> InputManagerBundle<PlayerAction> {
        InputManagerBundle {
            input_map: InputMap::new([
                (PlayerAction::Jump, KeyCode::Space),
                (PlayerAction::Sprint, KeyCode::ShiftLeft),
                (PlayerAction::Interact, KeyCode::KeyE),
            ])
            .with_dual_axis(PlayerAction::Move, KeyboardVirtualDPad::WASD),
            ..default()
        }
    }
}

impl CameraAction {
    pub fn input_bundle() -> InputManagerBundle<CameraAction> {
        InputManagerBundle {
            input_map: InputMap::default()
                .with_dual_axis(CameraAction::Orbit, MouseMove::default())
                .with_axis(CameraAction::Zoom, MouseScrollAxis::Y),
            ..default()
        }
    }
}

fn disable_actions<T: Actionlike>(mut actions_query: Query<&mut ActionState<T>>) {
    for mut actions in actions_query.iter_mut() {
        actions.disable();
    }
}
fn enable_actions<T: Actionlike>(mut actions_query: Query<&mut ActionState<T>>) {
    for mut actions in actions_query.iter_mut() {
        actions.enable();
    }
}

pub(crate) trait DualAxisDataExt {
    fn max_normalized(self) -> Option<Vec2>;
}

impl DualAxisDataExt for Vec2 {
    fn max_normalized(self) -> Option<Vec2> {
        let len_squared = self.length_squared();
        if len_squared > 1.0 {
            Some(self.normalize())
        } else if len_squared < 1e-5 {
            None
        } else {
            Some(self)
        }
    }
}
