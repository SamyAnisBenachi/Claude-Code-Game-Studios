use bevy::prelude::*;

use crate::core::pool::CardPoolSet;
use crate::core::rsm::advance_phase;
use crate::core::session::SessionSystemSet;
use crate::feature::auction::auction_tick_system;

use super::hands::PlayerHands;
use super::messages::ShopRefreshTriggered;
use super::state::ShopStates;
use super::system::{card_acquisition_tick_system, CardAcquisitionSet};

pub struct CardAcquisitionPlugin;

impl Plugin for CardAcquisitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopStates>()
            .init_resource::<PlayerHands>()
            .add_message::<ShopRefreshTriggered>()
            .configure_sets(
                Update,
                CardAcquisitionSet::Tick
                    .after(CardPoolSet::Lifecycle)
                    .after(auction_tick_system)
                    .after(advance_phase),
            )
            .add_systems(
                Update,
                card_acquisition_tick_system
                    .in_set(CardAcquisitionSet::Tick)
                    .in_set(SessionSystemSet::LiveMessages),
            );
    }
}
