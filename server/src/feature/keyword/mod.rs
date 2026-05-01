// KW-001 exposes interfaces before downstream keyword behavior stories use them.
#![allow(dead_code, unused_imports)]

pub mod components;
pub mod effects;
pub mod events;
pub mod movement;
pub mod observers;
pub mod resources;
pub mod state_eval;

use bevy::prelude::*;

use observers::{
    on_end_of_turn, on_final_blow_dealt, on_start_of_turn, on_unit_appeared, on_unit_died,
    start_of_turn_dispatch_system,
};

pub use components::{EnteredPlayRound, UnitKeywordState};
pub use events::{
    EndOfTurnTriggered, FinalBlowDealt, KeywordTriggered, StartOfTurnTriggered, UnitAppeared,
    UnitDied,
};
pub use resources::ChainDeathBuffer;

pub struct KeywordPlugin;

impl Plugin for KeywordPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_unit_appeared)
            .add_observer(on_unit_died)
            .add_observer(on_final_blow_dealt)
            .add_observer(on_start_of_turn)
            .add_observer(on_end_of_turn)
            .init_resource::<resources::ChainDeathBuffer>()
            .add_message::<events::KeywordTriggered>()
            .add_systems(Update, start_of_turn_dispatch_system);
    }
}
