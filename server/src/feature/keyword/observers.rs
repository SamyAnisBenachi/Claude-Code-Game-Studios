use bevy::prelude::*;
use shared::card::{Keyword, SimpleKeyword};

use crate::core::board::UnitCardRef;
use crate::core::rsm::DraftStarted;
use crate::foundation::config::CardCatalog;

use super::events::{
    EndOfTurnTriggered, FinalBlowDealt, StartOfTurnTriggered, UnitAppeared, UnitDied,
};

pub fn on_unit_appeared(_trigger: On<UnitAppeared>) {
    todo!()
}

pub fn on_unit_died(
    trigger: On<UnitDied>,
    units: Query<&UnitCardRef>,
    card_catalog: Option<Res<CardCatalog>>,
) {
    let entity = trigger.entity;
    let Ok(card_ref) = units.get(entity) else {
        return;
    };
    let Some(card_catalog) = card_catalog else {
        return;
    };
    let Some(card) = card_catalog.cards.get(&card_ref.0) else {
        return;
    };
    if !card
        .keywords
        .iter()
        .any(|keyword| matches!(keyword, Keyword::Simple(SimpleKeyword::Death)))
    {
        return;
    }
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
