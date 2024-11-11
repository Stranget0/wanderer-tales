//! A credits screen that can be accessed from the title screen.

use crate::{
    game::{assets::SoundtrackKey, audio::soundtrack::PlaySoundtrack},
    prelude::*,
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Credits), enter_credits);
    app.add_systems(OnExit(GameState::Credits), exit_credits);

    app.add_systems(
        Update,
        handle_credits_action.run_if(in_state(GameState::Credits)),
    );
    app.register_type::<CreditsAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum CreditsAction {
    Back,
}

fn enter_credits(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(GameState::Credits))
        .with_children(|children| {
            children.header("Made by");
            children.label("Alice - Foo");
            children.label("Bob - Bar");

            children.header("Assets");
            children.label("Bevy logo - All rights reserved by the Bevy Foundation. Permission granted for splash screen use when unmodified.");
            children.label("Ducky sprite - CC0 by Caz Creates Games");
            children.label("Music - CC BY 3.0 by Kevin MacLeod");

            children.button("Back").insert(CreditsAction::Back);
        });

    commands.trigger(PlaySoundtrack::Key(SoundtrackKey::Credits));
}

fn exit_credits(mut commands: Commands) {
    commands.trigger(PlaySoundtrack::Disable);
}

fn handle_credits_action(
    mut next_screen: ResMut<NextState<GameState>>,
    mut button_query: InteractionQuery<&CreditsAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                CreditsAction::Back => next_screen.set(GameState::Title),
            }
        }
    }
}
