use bevy::prelude::*;

use crate::core::rsm::DraftStarted;

use super::events::{
    EndOfTurnTriggered, FinalBlowDealt, StartOfTurnTriggered, UnitAppeared, UnitDied,
};

pub fn on_unit_appeared(_trigger: On<UnitAppeared>) {
    todo!()
}

pub fn on_unit_died(_trigger: On<UnitDied>) {
    todo!()
}

pub fn on_final_blow_dealt(_trigger: On<FinalBlowDealt>) {
    todo!()
}

pub fn on_start_of_turn(_trigger: On<StartOfTurnTriggered>) {
    todo!()
}

pub fn on_end_of_turn(_trigger: On<EndOfTurnTriggered>) {
    todo!()
}

pub fn start_of_turn_dispatch_system(mut draft_started: MessageReader<DraftStarted>) {
    if draft_started.read().next().is_some() {
        todo!()
    }
}
