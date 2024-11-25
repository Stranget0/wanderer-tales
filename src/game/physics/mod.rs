#[cfg(feature = "dev")]
pub mod devtools;

use crate::prelude::*;
use avian3d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default());
}

#[derive(PhysicsLayer)]
pub(crate) enum CollisionLayer {
    Player,
    Character,
    Terrain,
    CameraObstacle,
    Sensor,
}
