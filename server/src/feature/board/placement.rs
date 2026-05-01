use std::collections::HashMap;

use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Replicate};
use shared::card::{CardData, CardType};
use shared::protocol::{PlacedCard, PlayTarget, S2CPlacementReveal};
use shared::session::PlayerId;

use crate::core::board::{BoardPosition, UnitCardRef, UnitOwner, UnitStats};
use crate::core::economy::{api as economy_api, PlayerEconomies};
use crate::core::rsm::{
    PlacementPhaseEntered, PlacementSubmitted, ResolutionPhaseEntered, RoundPhase, RoundState,
};
use crate::core::session::SessionConfig;
use crate::feature::acquisition::PlayerHands;
use crate::feature::board::{
    BoardCell, BoardConfig, BoardGrid, BoardOccupancy, LaneId, SpawnRangeState,
};
use crate::foundation::config::CardCatalog;

const PLAYER_A_TEAM_ID: u8 = 0;
const PLAYER_B_TEAM_ID: u8 = 1;
const MAX_FAKE_OBJECTIVES_DESTROYED: u8 = 2;

/// Server-side buffer of accepted placement submissions for the active phase.
///
/// This is plain Rust data. Pending placement cards are never represented as
/// ECS entities, so Lightyear cannot replicate them before the reveal boundary.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PendingPlacements {
    /// One final accepted submission per player.
    pub submissions: HashMap<PlayerId, PlayerSubmission>,
}

/// Accepted placement batch for one player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSubmission {
    /// Validated placements in submission order.
    pub placements: Vec<PlacedCard>,
    /// Server time at submission receipt. Tests may use `0.0`.
    pub submitted_at: std::time::Duration,
    /// True once the first valid submission is accepted for this phase.
    pub is_final: bool,
}

/// Internal server message after sender identity has been resolved.
///
/// The shared `C2SSubmitPlacement` payload does not carry trusted sender
/// identity. Network code should resolve the sender to `player` before writing
/// this message.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct PlacementSubmissionReceived {
    pub player: PlayerId,
    pub placements: Vec<PlacedCard>,
}

/// Internal signal emitted after reveal enqueue and entity spawn complete.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct PlacementCommitted {
    pub round_number: u32,
    pub committed_placements: HashMap<PlayerId, Vec<PlacedCard>>,
}

/// Testable stand-in for the future Lightyear network dispatch layer.
///
/// Messages pushed here represent `S2CPlacementReveal` enqueued on the reliable
/// channel. The close system pushes before it spawns any ECS unit entity.
#[derive(Resource, Default, Debug, Clone)]
pub struct PlacementRevealOutbox {
    messages: Vec<S2CPlacementReveal>,
}

impl PlacementRevealOutbox {
    /// Append a placement reveal in enqueue order.
    pub fn push(&mut self, message: S2CPlacementReveal) {
        self.messages.push(message);
    }

    /// Read enqueued reveal messages.
    pub fn messages(&self) -> &[S2CPlacementReveal] {
        &self.messages
    }

    /// Clear all recorded reveal messages.
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

/// Instrumentation for the reveal-before-spawn invariant.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PlacementCommitTrace {
    entries: Vec<PlacementCommitTraceEntry>,
}

impl PlacementCommitTrace {
    /// Record one close-phase action.
    pub fn push(&mut self, entry: PlacementCommitTraceEntry) {
        self.entries.push(entry);
    }

    /// Read recorded close-phase actions.
    pub fn entries(&self) -> &[PlacementCommitTraceEntry] {
        &self.entries
    }

    /// Clear all recorded actions.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Ordered trace entries for `close_placement_phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementCommitTraceEntry {
    PlacementRevealEnqueued,
    UnitSpawned { entity: Entity },
    PlacementCommittedWritten,
}

/// Outcome of processing a resolved placement submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementSubmissionResult {
    Accepted,
    DiscardedWrongPhase,
    DuplicateFinalSubmission,
    UnknownPlayer,
    MissingCatalog,
    MissingEconomy,
    CardMissingFromCatalog,
    CardNotInHand,
    InvalidTarget,
    SpawnRangeRejected,
    OccupancyRejected,
    InsufficientMana,
    OwnerMismatch,
}

/// Returns whether a card type must pass spawn-range validation.
///
/// GDD Formula F2 applies to Minion placements only. Structures and Traps
/// bypass spawn range and continue to later placement checks.
pub fn requires_spawn_range_validation(card_type: CardType) -> bool {
    matches!(card_type, CardType::Minion)
}

/// Implements GDD Formula F2 for Minion placement cells.
///
/// The caller supplies `fakes_destroyed` from `SpawnRangeState` at validation
/// time. Unknown players or invalid team assignments reject silently.
pub fn validate_spawn_range(
    target_cell: u8,
    player: PlayerId,
    fakes_destroyed: u8,
    session_config: &SessionConfig,
    board_config: &BoardConfig,
) -> bool {
    let expansion = fakes_destroyed.min(MAX_FAKE_OBJECTIVES_DESTROYED);

    match session_config.team_map.get(&player).copied() {
        Some(PLAYER_A_TEAM_ID) => {
            let min_cell = board_config.player_a_spawn_cell;
            let max_cell = min_cell
                .saturating_add(expansion)
                .min(board_config.cell_max);

            target_cell >= min_cell && target_cell <= max_cell
        }
        Some(PLAYER_B_TEAM_ID) => {
            let max_cell = board_config.player_b_spawn_cell;
            let min_cell = max_cell
                .saturating_sub(expansion)
                .max(board_config.cell_min);

            target_cell >= min_cell && target_cell <= max_cell
        }
        _ => false,
    }
}

/// Returns whether `player` has an open Minion slot in `lane`.
///
/// Occupancy is per-player. Team capacity is derived from the current session's
/// team map so 1v1 has one slot per team lane and 2v2 has two.
pub fn is_minion_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    session_config: &SessionConfig,
) -> bool {
    let Some(team) = session_config.team_map.get(&player).copied() else {
        return false;
    };

    if occupancy.minion_slots.contains_key(&(player, lane)) {
        return false;
    }

    let team_capacity = session_config
        .team_map
        .values()
        .filter(|candidate| **candidate == team)
        .count();

    let team_count = occupancy
        .minion_slots
        .keys()
        .filter(|(slot_player, slot_lane)| {
            *slot_lane == lane && session_config.team_map.get(slot_player).copied() == Some(team)
        })
        .count();

    team_count < team_capacity
}

/// Returns whether `player` may place a Trap at `(lane, cell)`.
pub fn is_trap_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
) -> bool {
    !occupancy.traps.contains_key(&(player, lane, cell))
}

/// Returns whether `player` may place a Structure at `(lane, cell)`.
pub fn is_structure_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
) -> bool {
    !occupancy.structures.contains_key(&(player, lane, cell))
}

/// Returns whether `player` may place a Field in `lane`.
pub fn is_field_slot_available(occupancy: &BoardOccupancy, player: PlayerId, lane: LaneId) -> bool {
    !occupancy.fields.contains_key(&(player, lane))
}

/// Clears the pending-placement buffer on each PLACEMENT phase entry.
pub fn placement_buffer_open(
    mut phase_entered: MessageReader<PlacementPhaseEntered>,
    mut pending: ResMut<PendingPlacements>,
) {
    if phase_entered.read().next().is_some() {
        pending.submissions.clear();
    }
}

/// Accepts valid placement submissions into the server-only pending buffer.
#[allow(clippy::too_many_arguments)]
pub fn handle_placement_submission(
    mut submissions: MessageReader<PlacementSubmissionReceived>,
    round_state: Option<Res<RoundState>>,
    session: Option<Res<SessionConfig>>,
    board_config: Res<BoardConfig>,
    spawn_ranges: Res<SpawnRangeState>,
    occupancy: Res<BoardOccupancy>,
    catalog: Option<Res<CardCatalog>>,
    economies: Option<Res<PlayerEconomies>>,
    hands: Option<Res<PlayerHands>>,
    mut pending: ResMut<PendingPlacements>,
    mut submitted: MessageWriter<PlacementSubmitted>,
) {
    let phase = round_state.as_deref().map(|state| state.phase);
    let session = session.as_deref();
    let catalog = catalog.as_deref();
    let economies = economies.as_deref();
    let hands = hands.as_deref();

    for submission in submissions.read() {
        if process_placement_submission(
            &mut pending,
            submission.player,
            submission.placements.clone(),
            phase,
            session,
            &board_config,
            &spawn_ranges,
            &occupancy,
            catalog,
            economies,
            hands,
        ) == PlacementSubmissionResult::Accepted
        {
            submitted.write(PlacementSubmitted {
                player: submission.player,
            });
        }
    }
}

/// Validates and records a single player placement batch.
#[allow(clippy::too_many_arguments)]
pub fn process_placement_submission(
    pending: &mut PendingPlacements,
    player: PlayerId,
    placements: Vec<PlacedCard>,
    phase: Option<RoundPhase>,
    session: Option<&SessionConfig>,
    board_config: &BoardConfig,
    spawn_ranges: &SpawnRangeState,
    occupancy: &BoardOccupancy,
    catalog: Option<&CardCatalog>,
    economies: Option<&PlayerEconomies>,
    hands: Option<&PlayerHands>,
) -> PlacementSubmissionResult {
    if phase != Some(RoundPhase::Placement) {
        return PlacementSubmissionResult::DiscardedWrongPhase;
    }

    let Some(session) = session else {
        return PlacementSubmissionResult::UnknownPlayer;
    };
    if !session.team_map.contains_key(&player) {
        return PlacementSubmissionResult::UnknownPlayer;
    }

    if pending
        .submissions
        .get(&player)
        .is_some_and(|submission| submission.is_final)
    {
        return PlacementSubmissionResult::DuplicateFinalSubmission;
    }

    let Some(catalog) = catalog else {
        return PlacementSubmissionResult::MissingCatalog;
    };
    let Some(economies) = economies else {
        return PlacementSubmissionResult::MissingEconomy;
    };

    if let Some(result) = validate_submission_batch(
        player,
        &placements,
        session,
        board_config,
        spawn_ranges,
        occupancy,
        catalog,
        economies,
        hands,
    ) {
        return result;
    }

    pending.submissions.insert(
        player,
        PlayerSubmission {
            placements,
            submitted_at: std::time::Duration::ZERO,
            is_final: true,
        },
    );

    PlacementSubmissionResult::Accepted
}

/// Commits buffered placements when RESOLUTION begins.
#[allow(clippy::too_many_arguments)]
pub fn close_placement_phase(
    mut commands: Commands,
    mut resolution_entered: MessageReader<ResolutionPhaseEntered>,
    mut pending: ResMut<PendingPlacements>,
    mut grid: ResMut<BoardGrid>,
    mut occupancy: ResMut<BoardOccupancy>,
    mut economies: Option<ResMut<PlayerEconomies>>,
    catalog: Option<Res<CardCatalog>>,
    mut outbox: ResMut<PlacementRevealOutbox>,
    mut trace: ResMut<PlacementCommitTrace>,
    mut committed: MessageWriter<PlacementCommitted>,
) {
    let Some(round_number) = resolution_entered.read().last().map(|event| event.round) else {
        return;
    };

    let Some(catalog) = catalog.as_deref() else {
        return;
    };

    let committed_sequence = collect_committed_placements(&pending);
    let committed_placements = committed_sequence
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    if let Some(economies) = economies.as_deref_mut() {
        deduct_committed_mana(&committed_placements, catalog, economies);
    }

    let reveal = S2CPlacementReveal {
        placements: committed_sequence
            .iter()
            .map(|(_, placements)| placements)
            .flat_map(|placements| placements.iter().cloned())
            .collect(),
    };

    outbox.push(reveal);
    trace.push(PlacementCommitTraceEntry::PlacementRevealEnqueued);

    for (_, placements) in &committed_sequence {
        for placement in placements {
            spawn_committed_placement(
                &mut commands,
                &mut grid,
                &mut occupancy,
                &mut trace,
                placement,
                catalog,
            );
        }
    }

    committed.write(PlacementCommitted {
        round_number,
        committed_placements,
    });
    trace.push(PlacementCommitTraceEntry::PlacementCommittedWritten);
    pending.submissions.clear();
}

/// Returns all spawned unit entities visible at one board cell.
pub fn get_units_at_cell(grid: &BoardGrid, lane: LaneId, cell: u8) -> Vec<Entity> {
    grid_indices(lane, cell, &BoardConfig::default())
        .and_then(|(lane_index, cell_index)| grid.lanes[lane_index][cell_index])
        .map(|board_cell| vec![board_cell.entity])
        .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn validate_submission_batch(
    player: PlayerId,
    placements: &[PlacedCard],
    session: &SessionConfig,
    board_config: &BoardConfig,
    spawn_ranges: &SpawnRangeState,
    occupancy: &BoardOccupancy,
    catalog: &CardCatalog,
    economies: &PlayerEconomies,
    hands: Option<&PlayerHands>,
) -> Option<PlacementSubmissionResult> {
    let mut staged_occupancy = occupancy.clone();
    let mut total_cost = 0_u32;

    for placement in placements {
        if placement.owner_id != player {
            return Some(PlacementSubmissionResult::OwnerMismatch);
        }
        if hands
            .and_then(|hands| hands.hands.get(&player))
            .is_some_and(|hand| !hand.contains(&placement.card_id))
        {
            return Some(PlacementSubmissionResult::CardNotInHand);
        }

        let Some(card) = catalog.cards.get(&placement.card_id) else {
            return Some(PlacementSubmissionResult::CardMissingFromCatalog);
        };

        if let Some(result) = validate_placement_target(
            placement,
            card,
            session,
            board_config,
            spawn_ranges,
            &staged_occupancy,
        ) {
            return Some(result);
        }

        stage_occupancy(&mut staged_occupancy, player, placement, card.card_type);
        total_cost = total_cost.saturating_add(card.cost);
    }

    let Some(economy) = economies.0.get(&player) else {
        return Some(PlacementSubmissionResult::MissingEconomy);
    };
    if economy_api::validate_spend(economy, total_cost, false).is_err() {
        return Some(PlacementSubmissionResult::InsufficientMana);
    }

    None
}

fn validate_placement_target(
    placement: &PlacedCard,
    card: &CardData,
    session: &SessionConfig,
    board_config: &BoardConfig,
    spawn_ranges: &SpawnRangeState,
    occupancy: &BoardOccupancy,
) -> Option<PlacementSubmissionResult> {
    match (card.card_type, &placement.target) {
        (CardType::Minion, PlayTarget::BoardCell { lane, cell }) => {
            if !valid_lane_cell(*lane, *cell, board_config) {
                return Some(PlacementSubmissionResult::InvalidTarget);
            }
            let Some(fakes_destroyed) =
                fakes_destroyed_for(spawn_ranges, placement.owner_id, session)
            else {
                return Some(PlacementSubmissionResult::SpawnRangeRejected);
            };
            if !validate_spawn_range(
                *cell,
                placement.owner_id,
                fakes_destroyed,
                session,
                board_config,
            ) {
                return Some(PlacementSubmissionResult::SpawnRangeRejected);
            }
            if !is_minion_slot_available(occupancy, placement.owner_id, *lane, session) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Trap, PlayTarget::BoardCell { lane, cell }) => {
            if !valid_lane_cell(*lane, *cell, board_config) {
                return Some(PlacementSubmissionResult::InvalidTarget);
            }
            if !is_trap_slot_available(occupancy, placement.owner_id, *lane, *cell) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Structure, PlayTarget::BoardCell { lane, cell }) => {
            if !valid_lane_cell(*lane, *cell, board_config) {
                return Some(PlacementSubmissionResult::InvalidTarget);
            }
            if !is_structure_slot_available(occupancy, placement.owner_id, *lane, *cell) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Field, PlayTarget::LaneWide { lane }) => {
            if *lane < 1 || *lane > board_config.lane_count {
                return Some(PlacementSubmissionResult::InvalidTarget);
            }
            if !is_field_slot_available(occupancy, placement.owner_id, *lane) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Spell | CardType::Order | CardType::DoubleFace, _) => {}
        _ => return Some(PlacementSubmissionResult::InvalidTarget),
    }

    None
}

fn fakes_destroyed_for(
    spawn_ranges: &SpawnRangeState,
    player: PlayerId,
    session: &SessionConfig,
) -> Option<u8> {
    match session.team_map.get(&player).copied() {
        Some(PLAYER_A_TEAM_ID) => Some(spawn_ranges.fakes_destroyed[0]),
        Some(PLAYER_B_TEAM_ID) => Some(spawn_ranges.fakes_destroyed[1]),
        _ => None,
    }
}

fn stage_occupancy(
    occupancy: &mut BoardOccupancy,
    player: PlayerId,
    placement: &PlacedCard,
    card_type: CardType,
) {
    match (card_type, placement.target.clone()) {
        (CardType::Minion, PlayTarget::BoardCell { lane, .. }) => {
            occupancy
                .minion_slots
                .insert((player, lane), Entity::PLACEHOLDER);
        }
        (CardType::Trap, PlayTarget::BoardCell { lane, cell }) => {
            occupancy
                .traps
                .insert((player, lane, cell), Entity::PLACEHOLDER);
        }
        (CardType::Structure, PlayTarget::BoardCell { lane, cell }) => {
            occupancy
                .structures
                .insert((player, lane, cell), Entity::PLACEHOLDER);
        }
        (CardType::Field, PlayTarget::LaneWide { lane }) => {
            occupancy.fields.insert((player, lane), Entity::PLACEHOLDER);
        }
        _ => {}
    }
}

fn collect_committed_placements(pending: &PendingPlacements) -> Vec<(PlayerId, Vec<PlacedCard>)> {
    let mut players = pending.submissions.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);

    players
        .into_iter()
        .filter_map(|player| {
            pending
                .submissions
                .get(&player)
                .map(|submission| (player, submission.placements.clone()))
        })
        .collect()
}

fn deduct_committed_mana(
    committed: &HashMap<PlayerId, Vec<PlacedCard>>,
    catalog: &CardCatalog,
    economies: &mut PlayerEconomies,
) {
    let mut players = committed.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);

    for player in players {
        let cost = committed
            .get(&player)
            .into_iter()
            .flat_map(|placements| placements.iter())
            .filter_map(|placement| catalog.cards.get(&placement.card_id))
            .map(|card| card.cost)
            .sum::<u32>();

        let Some(economy) = economies.0.get_mut(&player) else {
            continue;
        };
        if economy_api::validate_spend(economy, cost, false).is_ok() {
            economy_api::apply_spend(economy, cost, false);
        }
    }
}

fn spawn_committed_placement(
    commands: &mut Commands,
    grid: &mut BoardGrid,
    occupancy: &mut BoardOccupancy,
    trace: &mut PlacementCommitTrace,
    placement: &PlacedCard,
    catalog: &CardCatalog,
) {
    let Some(card) = catalog.cards.get(&placement.card_id) else {
        return;
    };

    match placement.target {
        PlayTarget::BoardCell { lane, cell } => {
            let entity = commands
                .spawn((
                    UnitCardRef(placement.card_id),
                    UnitOwner(placement.owner_id),
                    UnitStats::new(card.hp, card.atk, card.mp, card.ar),
                    BoardPosition { lane, cell },
                    Replicate::to_clients(NetworkTarget::All),
                ))
                .id();
            if let Some((lane_index, cell_index)) =
                grid_indices(lane, cell, &BoardConfig::default())
            {
                grid.lanes[lane_index][cell_index] = Some(BoardCell::new(entity));
            }
            apply_spawned_occupancy(
                occupancy,
                placement.owner_id,
                lane,
                cell,
                card.card_type,
                entity,
            );
            trace.push(PlacementCommitTraceEntry::UnitSpawned { entity });
        }
        PlayTarget::LaneWide { lane } => {
            let entity = commands
                .spawn((
                    UnitCardRef(placement.card_id),
                    UnitOwner(placement.owner_id),
                    Replicate::to_clients(NetworkTarget::All),
                ))
                .id();
            occupancy.fields.insert((placement.owner_id, lane), entity);
            trace.push(PlacementCommitTraceEntry::UnitSpawned { entity });
        }
        PlayTarget::Instant | PlayTarget::TargetUnit { .. } | PlayTarget::TargetObj { .. } => {}
    }
}

fn apply_spawned_occupancy(
    occupancy: &mut BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
    card_type: CardType,
    entity: Entity,
) {
    match card_type {
        CardType::Minion => {
            occupancy.minion_slots.insert((player, lane), entity);
        }
        CardType::Trap => {
            occupancy.traps.insert((player, lane, cell), entity);
        }
        CardType::Structure => {
            occupancy.structures.insert((player, lane, cell), entity);
        }
        CardType::Field => {
            occupancy.fields.insert((player, lane), entity);
        }
        CardType::Spell | CardType::Order | CardType::DoubleFace => {}
    }
}

fn valid_lane_cell(lane: LaneId, cell: u8, board_config: &BoardConfig) -> bool {
    lane >= 1
        && lane <= board_config.lane_count
        && cell >= board_config.cell_min
        && cell <= board_config.cell_max
}

fn grid_indices(lane: LaneId, cell: u8, board_config: &BoardConfig) -> Option<(usize, usize)> {
    if !valid_lane_cell(lane, cell, board_config) {
        return None;
    }

    Some((usize::from(lane - 1), usize::from(cell - 1)))
}
