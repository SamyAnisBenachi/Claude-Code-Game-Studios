use bevy::prelude::*;

use crate::core::rsm::advance_phase;
use crate::feature::board::{
    apply_attract_displacements, apply_change_lane_displacements, apply_repel_displacements,
    close_placement_phase, handle_placement_submission, placement_buffer_open, update_spawn_range,
    AttractDisplacement, BoardConfig, BoardGrid, BoardOccupancy, ChangeLaneDisplacement,
    FakeObjectiveDestroyed, PendingPlacements, PlacementCommitTrace, PlacementCommitted,
    PlacementSubmissionReceived, PrismState, RepelDisplacement, SpawnRangeState, TrapTrigger,
    UnitAtObjective,
};

/// Board/Lane system ordering labels.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardSystemSet {
    PlacementBufferOpen,
    PlacementSubmission,
    PlacementClose,
    SpawnRangeUpdate,
    Displacement,
}

/// Registers server-only board resources.
pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardGrid>()
            .init_resource::<BoardOccupancy>()
            .init_resource::<SpawnRangeState>()
            .init_resource::<PrismState>()
            .init_resource::<PendingPlacements>()
            .init_resource::<PlacementCommitTrace>()
            .insert_resource(BoardConfig::default())
            .add_message::<PlacementSubmissionReceived>()
            .add_message::<PlacementCommitted>()
            .add_message::<FakeObjectiveDestroyed>()
            .add_message::<RepelDisplacement>()
            .add_message::<AttractDisplacement>()
            .add_message::<ChangeLaneDisplacement>()
            .add_message::<TrapTrigger>()
            .add_message::<UnitAtObjective>()
            .add_message::<crate::core::rsm::PlacementPhaseEntered>()
            .add_message::<crate::core::rsm::PlacementSubmitted>()
            .add_message::<crate::core::rsm::ResolutionPhaseEntered>()
            .configure_sets(
                Update,
                (
                    BoardSystemSet::PlacementBufferOpen,
                    BoardSystemSet::PlacementSubmission,
                    BoardSystemSet::PlacementClose,
                    BoardSystemSet::SpawnRangeUpdate,
                    BoardSystemSet::Displacement,
                )
                    .chain()
                    .after(advance_phase),
            )
            .add_systems(
                Update,
                placement_buffer_open.in_set(BoardSystemSet::PlacementBufferOpen),
            )
            .add_systems(
                Update,
                handle_placement_submission.in_set(BoardSystemSet::PlacementSubmission),
            )
            .add_systems(
                Update,
                close_placement_phase.in_set(BoardSystemSet::PlacementClose),
            )
            .add_systems(
                Update,
                update_spawn_range.in_set(BoardSystemSet::SpawnRangeUpdate),
            )
            .add_systems(
                Update,
                (
                    apply_repel_displacements,
                    apply_attract_displacements,
                    apply_change_lane_displacements,
                )
                    .in_set(BoardSystemSet::Displacement),
            );
    }
}
