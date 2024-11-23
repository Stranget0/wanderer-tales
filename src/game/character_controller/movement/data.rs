use super::{CharacterAnimation, CollisionLayer};
use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Walk>()
        .register_type::<Jump>()
        .register_type::<Sprinting>()
        .register_type::<FloatHeight>();
}

pub(crate) trait CharacterControllerExt {
    fn walk(
        &mut self,
        walking: &mut Walk,
        sprinting: Option<&mut Sprinting>,
        looking_at: Option<&LookingAt>,
        float_height: &FloatHeight,
        rotation_speed: &RotationSpeed,
    );
    fn jump(&mut self, jump: &mut Jump);
    fn look_at(
        &mut self,
        walking: &Walk,
        jumping: &Jump,
        looking_at: &LookingAt,
        float_height: &FloatHeight,
        rotation_speed: &RotationSpeed,
    );
}

#[derive(Bundle)]
pub(crate) struct CharacterControllerBundle {
    pub(crate) walking: Walk,
    pub(crate) sprinting: Sprinting,
    pub(crate) jumping: Jump,
    pub(crate) collider: Collider,
    pub(crate) rigid_body: RigidBody,
    pub(crate) locked_axes: LockedAxes,
    pub(crate) collision_layers: CollisionLayers,
    pub(crate) tnua_sensor_shape: bevy_tnua_avian3d::TnuaAvian3dSensorShape,
    pub(crate) tnua_controller: TnuaControllerBundle,
    pub(crate) float_height: FloatHeight,
    pub(crate) rotation_speed: RotationSpeed,
    pub(super) animation_state: bevy_tnua::TnuaAnimatingState<CharacterAnimation>,
}

#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Walk {
    /// Top speed on the ground
    pub(crate) speed: f32,
    /// Direction in which we want to walk and turn this tick.
    pub(crate) direction: Option<Dir3>,
}

#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Jump {
    /// The full height of the jump, if the player does not release the button
    pub(crate) height: f32,
    /// Was jump requested this frame?
    pub(crate) requested: bool,
}

#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Sprinting {
    /// The speed multiplier when sprinting
    pub(crate) multiplier: f32,
    /// Was sprinting requested?
    pub(crate) requested: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
/// Must be larger than the height of the entity's center from the bottom of its
/// collider, or else the character will not float and Tnua will not work properly
pub(crate) struct FloatHeight(pub(crate) f32);

/// radians / second
#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct RotationSpeed(f32);

#[derive(Debug, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct LookingAt(Option<Dir3>);

impl CharacterControllerBundle {
    pub(crate) fn capsule(height: f32, radius: f32) -> Self {
        Self {
            collider: Collider::capsule(height, radius),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            collision_layers: CollisionLayers::new(
                [CollisionLayer::Character],
                [
                    CollisionLayer::Player,
                    CollisionLayer::Character,
                    CollisionLayer::Terrain,
                    CollisionLayer::Sensor,
                ],
            ),
            tnua_sensor_shape: bevy_tnua_avian3d::TnuaAvian3dSensorShape(Collider::capsule(
                height * 0.95,
                radius * 0.95,
            )),
            float_height: FloatHeight(height / 2. + 0.1),
            walking: default(),
            sprinting: default(),
            jumping: default(),
            rotation_speed: default(),
            tnua_controller: default(),
            animation_state: default(),
        }
    }
}

impl CharacterControllerExt for TnuaController {
    fn walk(
        &mut self,
        walking: &mut Walk,
        sprinting: Option<&mut Sprinting>,
        looking_at: Option<&LookingAt>,
        float_height: &FloatHeight,
        rotation_speed: &RotationSpeed,
    ) {
        let direction = walking.direction;
        let sprinting_multiplier = sprinting
            .filter(|s| s.requested)
            .map(|s| s.multiplier)
            .unwrap_or(1.);
        let speed = walking.speed * sprinting_multiplier;
        self.basis(TnuaBuiltinWalk {
            desired_velocity: direction.map(|d| d.as_vec3() * speed).unwrap_or_default(),
            desired_forward: looking_at
                .and_then(|l| l.horizontal().direction())
                .or_else(|| direction.map(|d| -d)),
            float_height: float_height.0,
            cling_distance: 0.1,
            turning_angvel: rotation_speed.radians_per_second(),
            ..Default::default()
        });
        walking.direction = None;
    }

    fn jump(&mut self, jump: &mut Jump) {
        self.action(TnuaBuiltinJump {
            height: jump.height,
            takeoff_extra_gravity: 10.0,
            ..Default::default()
        });
        jump.requested = false;
    }

    fn look_at(
        &mut self,
        walking: &Walk,
        jumping: &Jump,
        looking_at: &LookingAt,
        float_height: &FloatHeight,
        rotation_speed: &RotationSpeed,
    ) {
        let direction = looking_at.horizontal().direction();
        if walking.direction.is_some() || jumping.requested || direction.is_none() {
            return;
        }

        self.basis(TnuaBuiltinWalk {
            desired_forward: direction,
            float_height: float_height.0,
            cling_distance: 0.1,
            turning_angvel: rotation_speed.radians_per_second(),
            ..Default::default()
        });
    }
}

impl Default for Walk {
    fn default() -> Self {
        Self {
            speed: 1.,
            direction: None,
        }
    }
}

impl Default for Sprinting {
    fn default() -> Self {
        Self {
            multiplier: 1.5,
            requested: false,
        }
    }
}

impl Default for Jump {
    fn default() -> Self {
        Self {
            height: 1.0,
            requested: false,
        }
    }
}

impl RotationSpeed {
    pub(crate) fn radians_per_second(&self) -> f32 {
        self.0
    }
    pub(crate) fn degrees_per_second(&self) -> f32 {
        self.0.to_degrees()
    }
}

impl Default for RotationSpeed {
    fn default() -> Self {
        Self(360.0_f32.to_radians())
    }
}

impl LookingAt {
    pub(crate) fn new(direction: Dir3) -> Self {
        Self(Some(direction))
    }
    pub(crate) fn from_points(from: Vec3, to: Vec3) -> Result<Self, InvalidDirectionError> {
        let direction = Dir3::new(to - from)?;
        Ok(Self(Some(direction)))
    }
    pub(crate) fn horizontal(&self) -> Self {
        let dir = self
            .0
            .and_then(|dir| Dir3::from_xyz(dir.x, 0.0, dir.z).ok());

        Self(dir)
    }
    pub(crate) fn direction(&self) -> Option<Dir3> {
        self.0
    }
    pub(crate) fn is_some(&self) -> bool {
        self.0.is_some()
    }
    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
    }
}
