use bevy::prelude::*;
mod noise;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(noise::plugin);
}
