use bevy::prelude::*;
use lightyear::prelude::*;

use crate::core::rsm::{advance_phase, rsm_input_reader};
use crate::feature::objective::{
    broadcast_objective_events, deliver_objective_identities_on_ready,
    initialize_objectives_on_draft_initial, objective_resolution_ready, HiddenObjectives,
    ObjectiveCounters, ObjectiveDestroyed, ObjectiveHp, ObjectiveIdentitiesReady,
    ObjectiveNetworkOutbox, ObjectiveResolutionState, PendingObjectiveEvents,
};

/// Objective System schedule labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectiveSystemSet {
    /// Session-entry objective initialization.
    Initialize,
    /// Owner-only hidden objective identity delivery.
    IdentityDelivery,
    /// RESOLUTION entry subscriber.
    ResolutionReady,
    /// RESOLUTION-end objective reveal broadcast.
    ResolutionBroadcast,
}

/// Registers objective state resources, replication, and DRAFT_INITIAL setup.
pub struct ObjectivePlugin;

impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<ObjectiveHp>();

        app.add_message::<ObjectiveIdentitiesReady>()
            .add_message::<ObjectiveDestroyed>()
            .init_resource::<HiddenObjectives>()
            .init_resource::<ObjectiveCounters>()
            .init_resource::<PendingObjectiveEvents>()
            .init_resource::<ObjectiveNetworkOutbox>()
            .init_resource::<ObjectiveResolutionState>()
            .configure_sets(
                Update,
                (
                    ObjectiveSystemSet::ResolutionBroadcast
                        .after(rsm_input_reader)
                        .before(advance_phase),
                    ObjectiveSystemSet::Initialize.after(advance_phase),
                    ObjectiveSystemSet::ResolutionReady.after(advance_phase),
                    ObjectiveSystemSet::IdentityDelivery.after(ObjectiveSystemSet::Initialize),
                ),
            )
            .add_systems(
                Update,
                broadcast_objective_events.in_set(ObjectiveSystemSet::ResolutionBroadcast),
            )
            .add_systems(
                Update,
                initialize_objectives_on_draft_initial.in_set(ObjectiveSystemSet::Initialize),
            )
            .add_systems(
                Update,
                objective_resolution_ready.in_set(ObjectiveSystemSet::ResolutionReady),
            )
            .add_systems(
                Update,
                deliver_objective_identities_on_ready.in_set(ObjectiveSystemSet::IdentityDelivery),
            );
    }
}
