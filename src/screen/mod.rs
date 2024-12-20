//! The game's main screen states and transitions between them.

mod credits;
mod loading;
mod playing;
mod splash;
mod title;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_state(GameState::Splash);
    app.enable_state_scoped_entities::<GameState>();

    app.add_plugins((
        splash::plugin,
        loading::plugin,
        title::plugin,
        credits::plugin,
        playing::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum GameState {
    #[default]
    Splash,
    Loading,
    Title,
    Credits,
    Playing,
}

mod prelude {
    pub use super::GameState;
    pub use crate::game::{assets::SoundtrackKey, audio::soundtrack::PlaySoundtrack};
}
