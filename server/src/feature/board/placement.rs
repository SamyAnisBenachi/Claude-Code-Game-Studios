use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use lightyear::prelude::{NetworkTarget, Replicate, Server, ServerMultiMessageSender};
use shared::card::{CardData, CardId, CardType};
use shared::protocol::{
    PlacedCardReveal, PlacedCardSubmit, PlayTarget, ReliableChannel, S2CPlacementReveal,
};
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
    pub placements: Vec<AcceptedPlacement>,
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
    pub placements: Vec<PlacedCardSubmit>,
}

/// Internal signal emitted when a player destroys an opposing fake objective.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FakeObjectiveDestroyed {
    pub destroyed_by: PlayerId,
}

/// Internal signal emitted after reveal enqueue and entity spawn complete.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct PlacementCommitted {
    pub round_number: u32,
    pub committed_placements: HashMap<PlayerId, Vec<AcceptedPlacement>>,
}

/// Server-internal placement after sender identity has been resolved.
///
/// This intentionally differs from both protocol payloads. C2S submit entries
/// do not carry trusted ownership, while S2C reveal entries must omit mana
/// spend fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedPlacement {
    pub owner_id: PlayerId,
    pub card_id: CardId,
    pub target: PlayTarget,
    pub current_mana_spend: u32,
    pub reserve_mana_spend: u32,
}

impl AcceptedPlacement {
    fn from_submit(owner_id: PlayerId, placement: PlacedCardSubmit) -> Self {
        Self {
            owner_id,
            card_id: placement.card_id,
            target: placement.target,
            current_mana_spend: placement.current_mana_spend,
            reserve_mana_spend: placement.reserve_mana_spend,
        }
    }

    pub fn reveal(&self) -> PlacedCardReveal {
        PlacedCardReveal {
            owner_id: self.owner_id,
            card_id: self.card_id,
            target: self.target.clone(),
        }
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
    ManaDeducted,
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
    DuplicateCardId,
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

/// Expands each player's spawn range after fake objective destruction.
pub fn update_spawn_range(
    mut destroyed: MessageReader<FakeObjectiveDestroyed>,
    session: Option<Res<SessionConfig>>,
    mut spawn_ranges: ResMut<SpawnRangeState>,
) {
    let Some(session) = session.as_deref() else {
        for _ in destroyed.read() {}
        return;
    };

    for event in destroyed.read() {
        let Some(index) = spawn_range_index_for(event.destroyed_by, session) else {
            continue;
        };
        spawn_ranges.fakes_destroyed[index] = spawn_ranges.fakes_destroyed[index]
            .saturating_add(1)
            .min(MAX_FAKE_OBJECTIVES_DESTROYED);
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
    placements: Vec<PlacedCardSubmit>,
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
            placements: placements
                .into_iter()
                .map(|placement| AcceptedPlacement::from_submit(player, placement))
                .collect(),
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
    server: Query<&Server>,
    mut sender: Option<ServerMultiMessageSender>,
    mut trace: ResMut<PlacementCommitTrace>,
    mut committed: MessageWriter<PlacementCommitted>,
) {
    let Some(round_number) = resolution_entered.read().last().map(|event| event.round) else {
        return;
    };

    let Some(catalog) = catalog.as_deref() else {
        return;
    };
    let (Ok(server), Some(sender)) = (server.single(), sender.as_mut()) else {
        return;
    };

    let committed_sequence = collect_committed_placements(&pending);
    let committed_placements = committed_sequence
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    if !committed_placements.is_empty() {
        let Some(economies) = economies.as_deref_mut() else {
            return;
        };
        if !deduct_committed_mana(&committed_placements, catalog, economies) {
            return;
        }
        trace.push(PlacementCommitTraceEntry::ManaDeducted);
    }

    let reveal = S2CPlacementReveal {
        placements: committed_sequence
            .iter()
            .map(|(_, placements)| placements)
            .flat_map(|placements| placements.iter().map(AcceptedPlacement::reveal))
            .collect(),
    };

    if sender
        .send::<S2CPlacementReveal, ReliableChannel>(&reveal, server, &NetworkTarget::All)
        .is_err()
    {
        return;
    }
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
    placements: &[PlacedCardSubmit],
    session: &SessionConfig,
    board_config: &BoardConfig,
    spawn_ranges: &SpawnRangeState,
    occupancy: &BoardOccupancy,
    catalog: &CardCatalog,
    economies: &PlayerEconomies,
    hands: Option<&PlayerHands>,
) -> Option<PlacementSubmissionResult> {
    let mut staged_occupancy = occupancy.clone();
    let Some(economy) = economies.0.get(&player) else {
        return Some(PlacementSubmissionResult::MissingEconomy);
    };
    let Some(hand) = hands.and_then(|hands| hands.hands.get(&player)) else {
        return Some(PlacementSubmissionResult::CardNotInHand);
    };
    let mut submitted_cards = HashSet::new();
    let mut total_cost = 0_u32;
    let mut total_current_mana_spend = 0_u32;
    let mut total_reserve_mana_spend = 0_u32;

    for placement in placements {
        if !submitted_cards.insert(placement.card_id) {
            return Some(PlacementSubmissionResult::DuplicateCardId);
        }

        if !hand.contains(&placement.card_id) {
            return Some(PlacementSubmissionResult::CardNotInHand);
        }

        let Some(card) = catalog.cards.get(&placement.card_id) else {
            return Some(PlacementSubmissionResult::CardMissingFromCatalog);
        };

        if let Some(result) = validate_placement_target(
            player,
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
        let Some(next_total_cost) = checked_submission_total(total_cost, card.cost) else {
            return Some(PlacementSubmissionResult::InsufficientMana);
        };
        let Some(next_total_current_mana_spend) =
            checked_submission_total(total_current_mana_spend, placement.current_mana_spend)
        else {
            return Some(PlacementSubmissionResult::InsufficientMana);
        };
        let Some(next_total_reserve_mana_spend) =
            checked_submission_total(total_reserve_mana_spend, placement.reserve_mana_spend)
        else {
            return Some(PlacementSubmissionResult::InsufficientMana);
        };
        total_cost = next_total_cost;
        total_current_mana_spend = next_total_current_mana_spend;
        total_reserve_mana_spend = next_total_reserve_mana_spend;

        if economy_api::validate_explicit_mana_split(
            economy,
            card.cost,
            placement.current_mana_spend,
            placement.reserve_mana_spend,
        )
        .is_err()
        {
            return Some(PlacementSubmissionResult::InsufficientMana);
        }
    }

    if economy_api::validate_explicit_mana_split(
        economy,
        total_cost,
        total_current_mana_spend,
        total_reserve_mana_spend,
    )
    .is_err()
    {
        return Some(PlacementSubmissionResult::InsufficientMana);
    }

    None
}

fn checked_submission_total(current: u32, addend: u32) -> Option<u32> {
    current.checked_add(addend)
}

fn validate_placement_target(
    player: PlayerId,
    placement: &PlacedCardSubmit,
    card: &CardData,
    session: &SessionConfig,
    board_config: &BoardConfig,
    spawn_ranges: &SpawnRangeState,
    occupancy: &BoardOccupancy,
) -> Option<PlacementSubmissionResult> {
    if !play_target_in_bounds(&placement.target, session, board_config) {
        return Some(PlacementSubmissionResult::InvalidTarget);
    }

    match (card.card_type, &placement.target) {
        (CardType::Minion, PlayTarget::BoardCell { lane, cell }) => {
            let Some(fakes_destroyed) = fakes_destroyed_for(spawn_ranges, player, session) else {
                return Some(PlacementSubmissionResult::SpawnRangeRejected);
            };
            if !validate_spawn_range(*cell, player, fakes_destroyed, session, board_config) {
                return Some(PlacementSubmissionResult::SpawnRangeRejected);
            }
            if !is_minion_slot_available(occupancy, player, *lane, session) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Trap, PlayTarget::BoardCell { lane, cell }) => {
            if !is_trap_slot_available(occupancy, player, *lane, *cell) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Structure, PlayTarget::BoardCell { lane, cell }) => {
            if !is_structure_slot_available(occupancy, player, *lane, *cell) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Field, PlayTarget::LaneWide { lane }) => {
            if !is_field_slot_available(occupancy, player, *lane) {
                return Some(PlacementSubmissionResult::OccupancyRejected);
            }
        }
        (CardType::Spell | CardType::Order | CardType::DoubleFace, _) => {}
        _ => return Some(PlacementSubmissionResult::InvalidTarget),
    }

    None
}

fn play_target_in_bounds(
    target: &PlayTarget,
    session: &SessionConfig,
    board_config: &BoardConfig,
) -> bool {
    match target {
        PlayTarget::BoardCell { lane, cell } => valid_lane_cell(*lane, *cell, board_config),
        PlayTarget::TargetUnit { lane, .. } | PlayTarget::LaneWide { lane } => {
            *lane >= 1 && *lane <= board_config.lane_count
        }
        PlayTarget::TargetObj { player_id, lane } => {
            session.team_map.contains_key(player_id)
                && *lane >= 1
                && *lane <= board_config.lane_count
        }
        PlayTarget::Instant => true,
    }
}

fn fakes_destroyed_for(
    spawn_ranges: &SpawnRangeState,
    player: PlayerId,
    session: &SessionConfig,
) -> Option<u8> {
    spawn_range_index_for(player, session).map(|index| spawn_ranges.fakes_destroyed[index])
}

fn spawn_range_index_for(player: PlayerId, session: &SessionConfig) -> Option<usize> {
    match session.team_map.get(&player).copied() {
        Some(PLAYER_A_TEAM_ID) => Some(0),
        Some(PLAYER_B_TEAM_ID) => Some(1),
        _ => None,
    }
}

fn stage_occupancy(
    occupancy: &mut BoardOccupancy,
    player: PlayerId,
    placement: &PlacedCardSubmit,
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

fn collect_committed_placements(
    pending: &PendingPlacements,
) -> Vec<(PlayerId, Vec<AcceptedPlacement>)> {
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

pub fn deduct_committed_mana(
    committed: &HashMap<PlayerId, Vec<AcceptedPlacement>>,
    catalog: &CardCatalog,
    economies: &mut PlayerEconomies,
) -> bool {
    let mut players = committed.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);

    for player in &players {
        let Some(placements) = committed.get(player) else {
            continue;
        };
        let mut total_cost = 0_u32;
        let mut total_current_mana_spend = 0_u32;
        let mut total_reserve_mana_spend = 0_u32;

        for placement in placements {
            let Some(card) = catalog.cards.get(&placement.card_id) else {
                return false;
            };
            let Some(next_cost) = total_cost.checked_add(card.cost) else {
                return false;
            };
            let Some(next_current) =
                total_current_mana_spend.checked_add(placement.current_mana_spend)
            else {
                return false;
            };
            let Some(next_reserve) =
                total_reserve_mana_spend.checked_add(placement.reserve_mana_spend)
            else {
                return false;
            };
            total_cost = next_cost;
            total_current_mana_spend = next_current;
            total_reserve_mana_spend = next_reserve;
        }

        let Some(economy) = economies.0.get(player) else {
            return false;
        };
        if economy_api::validate_explicit_mana_split(
            economy,
            total_cost,
            total_current_mana_spend,
            total_reserve_mana_spend,
        )
        .is_err()
        {
            return false;
        }
    }

    for player in players {
        let Some(placements) = committed.get(&player) else {
            continue;
        };
        let Some(economy) = economies.0.get_mut(&player) else {
            return false;
        };
        for placement in placements {
            economy_api::apply_explicit_mana_split(
                economy,
                placement.current_mana_spend,
                placement.reserve_mana_spend,
            );
        }
    }

    true
}

fn spawn_committed_placement(
    commands: &mut Commands,
    grid: &mut BoardGrid,
    occupancy: &mut BoardOccupancy,
    trace: &mut PlacementCommitTrace,
    placement: &AcceptedPlacement,
    catalog: &CardCatalog,
) {
    let Some(card) = catalog.cards.get(&placement.card_id) else {
        return;
    };

    match &placement.target {
        PlayTarget::BoardCell { lane, cell } => {
            let entity = commands
                .spawn((
                    UnitCardRef(placement.card_id),
                    UnitOwner(placement.owner_id),
                    UnitStats::new(card.hp, card.atk, card.mp, card.ar),
                    BoardPosition {
                        lane: *lane,
                        cell: *cell,
                    },
                    Replicate::to_clients(NetworkTarget::All),
                ))
                .id();
            if let Some((lane_index, cell_index)) =
                grid_indices(*lane, *cell, &BoardConfig::default())
            {
                grid.lanes[lane_index][cell_index] = Some(BoardCell::new(entity));
            }
            apply_spawned_occupancy(
                occupancy,
                placement.owner_id,
                *lane,
                *cell,
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
            occupancy.fields.insert((placement.owner_id, *lane), entity);
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
