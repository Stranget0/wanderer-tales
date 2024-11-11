mod movement;
mod physics;

pub(crate) use movement::*;
use physics::CollisionLayer;

pub(super) fn plugin(app: &mut bevy::prelude::App) {
    app.add_plugins((movement::plugin, physics::plugin));
}
