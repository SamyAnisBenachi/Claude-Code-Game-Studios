use bevy::prelude::*;
use lightyear::prelude::*;

use crate::core::rsm::advance_phase;
use crate::core::session::SessionSystemSet;
use crate::feature::prism::{
    cleanup_prism_session, initialize_prism_session, resolve_prism_draws, PrismCollected,
    PrismNetworkOutbox, PrismPresence,
};

/// Prism System schedule labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrismSystemSet {
    Lifecycle,
    Resolve,
}

/// Registers Prism session state, public presence replication, and resolver scaffold.
pub struct PrismPlugin;

impl Plugin for PrismPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<PrismPresence>();

        app.init_resource::<PrismNetworkOutbox>()
            .add_message::<PrismCollected>()
            .add_message::<crate::core::rsm::DraftStarted>()
            .configure_sets(
                Update,
                (
                    PrismSystemSet::Lifecycle.after(advance_phase),
                    PrismSystemSet::Resolve.after(PrismSystemSet::Lifecycle),
                ),
            )
            .add_systems(
                Update,
                (initialize_prism_session, cleanup_prism_session)
                    .chain()
                    .in_set(PrismSystemSet::Lifecycle),
            )
            .add_systems(
                Update,
                resolve_prism_draws
                    .in_set(PrismSystemSet::Resolve)
                    .in_set(SessionSystemSet::LiveMessages),
            );
    }
}
