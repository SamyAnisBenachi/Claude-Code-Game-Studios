use bevy::prelude::*;
use lightyear::prelude::*;

use crate::core::rsm::advance_phase;
use crate::feature::objective::{
    initialize_objectives_on_draft_initial, HiddenObjectives, ObjectiveCounters, ObjectiveHp,
};

/// Objective System schedule labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectiveSystemSet {
    /// Session-entry objective initialization.
    Initialize,
}

/// Registers objective state resources, replication, and DRAFT_INITIAL setup.
pub struct ObjectivePlugin;

impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<ObjectiveHp>();

        app.init_resource::<HiddenObjectives>()
            .init_resource::<ObjectiveCounters>()
            .configure_sets(Update, ObjectiveSystemSet::Initialize.after(advance_phase))
            .add_systems(
                Update,
                initialize_objectives_on_draft_initial.in_set(ObjectiveSystemSet::Initialize),
            );
    }
}
