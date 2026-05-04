use bevy::prelude::*;

use crate::state::ClientState;
use crate::ui::shared::BoardLayout;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct CardAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

pub struct BoardRenderingPlugin;

impl Plugin for BoardRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>()
            .add_systems(
                OnEnter(ClientState::InSession),
                insert_board_rendering_session_resources,
            )
            .add_systems(
                OnExit(ClientState::InSession),
                remove_board_rendering_session_resources,
            );
    }
}

fn insert_board_rendering_session_resources(mut commands: Commands) {
    commands.insert_resource(BoardLayout::default());
    commands.insert_resource(CardAtlas::default());
}

fn remove_board_rendering_session_resources(mut commands: Commands) {
    commands.remove_resource::<BoardLayout>();
    commands.remove_resource::<CardAtlas>();
}
