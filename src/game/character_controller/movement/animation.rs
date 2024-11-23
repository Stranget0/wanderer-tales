use std::{fmt::Display, ops::Not, time::Duration};

use super::Walk;
use crate::prelude::*;
use bevy::animation::{ActiveAnimation, RepeatAnimation};
use bevy_tnua::{
    prelude::{TnuaBuiltinWalk, TnuaController},
    TnuaAnimatingState, TnuaAnimatingStateDirective,
};
use rand::seq::SliceRandom;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::Playing),
        load_animations.in_set(GameSet::RecordInput),
    )
    .add_systems(
        Update,
        (
            prepare_armature_for_animations
                .in_set(GameSet::RecordInput)
                .run_if(in_state(GameState::Playing)),
            handle_animations
                .in_set(GameSet::UpdateApply)
                .run_if(in_state(GameState::Playing)),
        ),
    )
    .add_systems(
        OnExit(GameState::Playing),
        unload_animations.in_set(GameSet::Cleanup),
    );
}

#[derive(Resource)]
struct CharacterAnimations {
    animations: hashbrown::HashMap<CharacterAnimation, AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum AnimationDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum CharacterAnimation {
    Walking(AnimationDirection),
    Running(AnimationDirection),
    Jumping,
    Idle(u8),
}

struct AnimationManager<'a, 'b, 'c> {
    animations: &'a CharacterAnimations,
    transitions: &'b mut AnimationTransitions,
    animation_player: &'c mut AnimationPlayer,
}

fn prepare_armature_for_animations(
    mut commands: Commands,
    mut armatures: Query<Entity, (With<AnimationPlayer>, Without<Handle<AnimationGraph>>)>,
    parent: Query<&Parent>,
    controller: Query<&TnuaController>,
    animations: Res<CharacterAnimations>,
) {
    for entity in armatures.iter_mut() {
        for ancestor in parent.iter_ancestors(entity) {
            if controller.contains(ancestor) {
                commands.entity(entity).insert((
                    animations.graph().clone_weak(),
                    AnimationTransitions::default(),
                ));
            }
        }
    }
}

fn load_animations(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CharacterAnimations::load(&asset_server));
}

fn unload_animations(mut commands: Commands) {
    commands.remove_resource::<CharacterAnimations>();
}

fn handle_animations(
    mut armatures: Query<(Entity, &mut AnimationTransitions, &mut AnimationPlayer)>,
    mut controllers: Query<(
        &TnuaController,
        &mut TnuaAnimatingState<CharacterAnimation>,
        &Walk,
    )>,
    parent: Query<&Parent>,
    animations: Res<CharacterAnimations>,
) {
    for (entity, mut transitions, mut animation_player) in armatures.iter_mut() {
        for ancestor in parent.iter_ancestors(entity) {
            let Ok((controller, mut animation_state, walk_settings)) =
                controllers.get_mut(ancestor)
            else {
                continue;
            };
            let Some((walk_basis, walk_basis_state)) =
                controller.concrete_basis::<TnuaBuiltinWalk>()
            else {
                continue;
            };

            let velocity = walk_basis_state.running_velocity;
            let desired_forward = walk_basis.desired_forward;
            let movement_speed = velocity.length();
            let walk_direction = desired_forward
                .map(|desired_forward| AnimationDirection::from_vectors(*desired_forward, velocity))
                .unwrap_or(AnimationDirection::Forward);

            let mut animation_manager = AnimationManager {
                transitions: &mut transitions,
                animation_player: &mut animation_player,
                animations: &animations,
            };

            let new_state = {
                if controller.is_airborne().unwrap_or(false) {
                    CharacterAnimation::Jumping
                } else if movement_speed < 0.01 {
                    if let Some(current_state) = animation_state.get().and_then(|state| {
                        state
                            .is_idle()
                            .then(|| {
                                animation_manager
                                    .is_animation_finished(state)
                                    .not()
                                    .then_some(*state)
                            })
                            .flatten()
                    }) {
                        current_state
                    } else {
                        CharacterAnimation::random_idle().unwrap()
                    }
                } else if movement_speed <= walk_settings.speed * 1.1 {
                    CharacterAnimation::Walking(walk_direction)
                } else {
                    CharacterAnimation::Running(walk_direction)
                }
            };

            match animation_state.update_by_value(new_state) {
                TnuaAnimatingStateDirective::Maintain { state } => {
                    if animation_manager.is_animation_finished(state) {
                        animation_manager.play(movement_speed, state, Some(state));
                    } else {
                        animation_manager.maintain_speed(movement_speed, state);
                    }
                }
                TnuaAnimatingStateDirective::Alter { old_state, state } => {
                    animation_manager.play(movement_speed, state, old_state.as_ref());
                }
            }
        }
    }
}

impl AnimationManager<'_, '_, '_> {
    fn play(
        &mut self,
        movement_speed: f32,
        state: &CharacterAnimation,
        old_state: Option<&CharacterAnimation>,
    ) {
        let Some(index) = self.animations.get_animation_index(state).copied() else {
            return;
        };

        let animation_speed = state.animation_speed(movement_speed);
        let repeat_mode = state.get_repeat_mode();

        self.transitions
            .play(
                self.animation_player,
                index,
                old_state
                    .map(|old| old.get_transition_duration(state))
                    .unwrap_or_default(),
            )
            .set_speed(animation_speed)
            .set_repeat(repeat_mode);
    }

    fn maintain_speed(&mut self, movement_speed: f32, state: &CharacterAnimation) {
        let animation_speed = state.animation_speed(movement_speed);

        if let Some(active_animation) = self
            .animations
            .get_animation_index(state)
            .and_then(|index| self.animation_player.animation_mut(*index))
        {
            active_animation.set_speed(animation_speed);
        }
    }

    fn animation(&self, animation: &CharacterAnimation) -> Option<&ActiveAnimation> {
        self.animations
            .get_animation_index(animation)
            .and_then(|index| self.animation_player.animation(*index))
    }

    fn is_animation_finished(&self, animation: &CharacterAnimation) -> bool {
        self.animation(animation)
            .map(|a| a.is_finished())
            .unwrap_or(true)
    }
}

impl CharacterAnimations {
    fn load(asset_server: &AssetServer) -> Self {
        let mut graph = AnimationGraph::default();
        let mut animations = hashbrown::HashMap::default();

        for animation in CharacterAnimation::all() {
            let animation_node =
                graph.add_clip(animation.get_or_load(asset_server), 1.0, graph.root);
            animations.insert(animation, animation_node);
        }

        let graph_handle = asset_server.add(graph);

        Self {
            animations,
            graph: graph_handle,
        }
    }

    fn graph(&self) -> &Handle<AnimationGraph> {
        &self.graph
    }

    fn get_animation_index(&self, animation: &CharacterAnimation) -> Option<&AnimationNodeIndex> {
        self.animations.get(animation)
    }

    fn get_animation_key(&self, index: AnimationNodeIndex) -> Option<CharacterAnimation> {
        self.animations
            .iter()
            .find_map(|(animation, i)| (*i == index).then_some(*animation))
    }
}

impl CharacterAnimation {
    const MAX_IDLE: u8 = 3;
    fn path(&self) -> String {
        match self {
            CharacterAnimation::Walking(direction) => match direction {
                AnimationDirection::Forward => "animations/soldier_walk_1.glb".to_string(),
                AnimationDirection::Backward => "animations/soldier_walk_back_1.glb".to_string(),
            },
            CharacterAnimation::Running(direction) => match direction {
                AnimationDirection::Forward => "animations/soldier_run_1.glb".to_string(),
                AnimationDirection::Backward => "animations/soldier_run_back_1.glb".to_string(),
            },
            CharacterAnimation::Jumping => "animations/soldier_jump_1.glb".to_string(),
            CharacterAnimation::Idle(num) => format!("animations/soldier_idle_{num}.glb"),
        }
    }

    pub fn load(self, asset_server: &AssetServer) -> Handle<AnimationClip> {
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(self.path()))
    }

    pub fn get(self, asset_server: &AssetServer) -> Option<Handle<AnimationClip>> {
        asset_server.get_handle(GltfAssetLabel::Animation(0).from_asset(self.path()))
    }

    pub fn get_or_load(self, asset_server: &AssetServer) -> Handle<AnimationClip> {
        self.get(asset_server)
            .unwrap_or_else(|| self.load(asset_server))
    }

    pub fn all_walking() -> Vec<CharacterAnimation> {
        vec![
            CharacterAnimation::Walking(AnimationDirection::Forward),
            CharacterAnimation::Walking(AnimationDirection::Backward),
        ]
    }

    pub fn all_running() -> Vec<CharacterAnimation> {
        vec![
            CharacterAnimation::Running(AnimationDirection::Forward),
            CharacterAnimation::Running(AnimationDirection::Backward),
        ]
    }

    pub fn all_idle() -> Vec<CharacterAnimation> {
        (1..=Self::MAX_IDLE).map(CharacterAnimation::Idle).collect()
    }

    pub fn random_idle() -> Option<CharacterAnimation> {
        Self::all_idle().choose(&mut rand::thread_rng()).copied()
    }

    pub fn all() -> Vec<CharacterAnimation> {
        Self::all_walking()
            .into_iter()
            .chain(Self::all_running())
            .chain(Self::all_idle())
            .chain([Self::Jumping])
            .collect()
    }

    pub fn is_running(&self) -> bool {
        matches!(self, CharacterAnimation::Running(_))
    }

    pub fn is_walking(&self) -> bool {
        matches!(self, CharacterAnimation::Walking(_))
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, CharacterAnimation::Idle(_))
    }

    pub fn is_speed_configurable(&self) -> bool {
        self.is_walking() || self.is_running()
    }

    pub fn animation_speed(&self, speed: f32) -> f32 {
        if self.is_speed_configurable() {
            speed
        } else {
            1.0
        }
    }

    pub fn get_repeat_mode(&self) -> RepeatAnimation {
        if self.is_walking() || self.is_running() {
            RepeatAnimation::Forever
        } else {
            RepeatAnimation::Count(1)
        }
    }
    pub fn get_transition_duration(&self, new_state: &CharacterAnimation) -> Duration {
        if new_state == &CharacterAnimation::Jumping {
            Duration::from_secs_f32(0.2)
        } else if self.is_walking() || self.is_running() {
            Duration::from_secs_f32(0.4)
        } else {
            Duration::from_secs_f32(0.5)
        }
    }
}

impl Display for AnimationDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationDirection::Forward => write!(f, "Forward"),
            AnimationDirection::Backward => write!(f, "Backward"),
        }
    }
}

impl AnimationDirection {
    pub fn from_vectors(a: Vec3, b: Vec3) -> Self {
        if a.dot(b) > 0.0 {
            AnimationDirection::Forward
        } else {
            AnimationDirection::Backward
        }
    }
}

impl Display for CharacterAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharacterAnimation::Walking(direction) => write!(f, "Walking {direction}"),
            CharacterAnimation::Running(direction) => write!(f, "Running {direction}"),
            CharacterAnimation::Idle(number) => write!(f, "Idle {number}"),
            CharacterAnimation::Jumping => write!(f, "Jumping"),
        }
    }
}
