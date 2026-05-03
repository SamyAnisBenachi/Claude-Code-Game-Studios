use bevy::prelude::*;
use lightyear::prelude::*;

use crate::core::rsm::advance_phase;
use crate::feature::objective::{
    deliver_objective_identities_on_ready, initialize_objectives_on_draft_initial,
    HiddenObjectives, ObjectiveCounters, ObjectiveHp, ObjectiveIdentitiesReady,
    ObjectiveNetworkOutbox, PendingObjectiveEvents,
};

/// Objective System schedule labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectiveSystemSet {
    /// Session-entry objective initialization.
    Initialize,
    /// Owner-only hidden objective identity delivery.
    IdentityDelivery,
}

/// Registers objective state resources, replication, and DRAFT_INITIAL setup.
pub struct ObjectivePlugin;

impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<ObjectiveHp>();

        app.add_message::<ObjectiveIdentitiesReady>()
            .init_resource::<HiddenObjectives>()
            .init_resource::<ObjectiveCounters>()
            .init_resource::<PendingObjectiveEvents>()
            .init_resource::<ObjectiveNetworkOutbox>()
            .configure_sets(
                Update,
                (
                    ObjectiveSystemSet::Initialize.after(advance_phase),
                    ObjectiveSystemSet::IdentityDelivery.after(ObjectiveSystemSet::Initialize),
                ),
            )
            .add_systems(
                Update,
                initialize_objectives_on_draft_initial.in_set(ObjectiveSystemSet::Initialize),
            )
            .add_systems(
                Update,
                deliver_objective_identities_on_ready.in_set(ObjectiveSystemSet::IdentityDelivery),
            );
    }
}
