use bevy::prelude::{Entity, Timer, World};
use shared::card::{CardId, ClassId};
use shared::protocol::{
    BoardSnapshot, FieldBoardState, ObjectiveSnapshot, OpponentObjectiveSnapshot,
    PlacementTimerMultiplier, PlayerSnapshot, PrismBoardState, RoundPhase as ProtocolRoundPhase,
    S2CGameSnapshot, SeedBoardState, SinistroState, StructureBoardState, TrapBoardState,
};
use shared::session::PlayerId;

use crate::core::board::{
    build_unit_board_states, BoardPosition, ClassTokenKind, ObjectiveAttachment, SeedMarker,
    SeedOwner, UnitCardRef, UnitOwner, UnitStats,
};
use crate::core::economy::{PlayerEconomies, PlayerEconomy};
use crate::core::pool::{PlayerPool, PlayerPools};
use crate::core::rsm::{RoundPhase, RoundState};
use crate::core::session::{PlayerSessions, SessionConfig};
use crate::feature::acquisition::{PlayerHands, ShopStates};
use crate::feature::auction::{auction_snapshot, AuctionState};
use crate::feature::board::{spawn_range_cells_for_player, BoardOccupancy, SpawnRangeState};
use crate::feature::objective::{
    HiddenObjectives, ObjectiveHp, ObjectiveSlot, OBJECTIVE_LANE_COUNT,
};
use crate::feature::prism::{PrismState, PRISM_LANE_COUNT};
use crate::foundation::config::GameConfig;

const DEFAULT_SINISTRO_DAMAGE: u8 = 1;

/// Builds a reconnect snapshot from authoritative server state.
///
/// The snapshot builder is intentionally a wide read: reconnect is rare, and
/// assembling one authoritative payload here keeps HUD/client rebuilds from
/// inventing local state or waiting for later incremental messages.
pub fn build_game_snapshot(
    recipient_player_id: PlayerId,
    world: &mut World,
) -> Option<S2CGameSnapshot> {
    let players = session_players(world)?;
    let round_phase = world.get_resource::<RoundState>().map(|state| state.phase);
    let round_number = world
        .get_resource::<RoundState>()
        .map(|state| state.round_number)
        .unwrap_or(0);
    let phase = round_phase
        .map(protocol_round_phase)
        .unwrap_or(ProtocolRoundPhase::Handshaking);
    let timer_remaining_ms = snapshot_timer_remaining_ms(world);
    let placement_timer_multiplier_effective = world
        .get_resource::<SessionConfig>()
        .map(|config| config.placement_timer_multiplier_effective)
        .unwrap_or(PlacementTimerMultiplier::X1);

    let player_snapshots = players
        .iter()
        .copied()
        .map(|player| build_player_snapshot(world, recipient_player_id, player, &players))
        .collect::<Vec<_>>();

    let board = build_board_snapshot(recipient_player_id, world, &players);
    let auction_state = if phase == ProtocolRoundPhase::DraftAuction {
        world
            .get_resource::<AuctionState>()
            .and_then(auction_snapshot)
    } else {
        None
    };

    Some(S2CGameSnapshot {
        protocol_version: protocol_version(world),
        recipient_player_id,
        round_number,
        phase,
        timer_remaining_ms,
        placement_timer_multiplier_effective,
        players: player_snapshots,
        board,
        auction_state,
        active_sang_meprise_reveals: crate::core::session::reconnect::active_sang_meprise_reveals(
            world,
            recipient_player_id,
        ),
    })
}

/// Backwards-compatible name used by existing GSS/class tests.
pub fn build_snapshot(player_id: PlayerId, world: &mut World) -> Option<S2CGameSnapshot> {
    build_game_snapshot(player_id, world)
}

fn build_player_snapshot(
    world: &mut World,
    recipient_player_id: PlayerId,
    player_id: PlayerId,
    players: &[PlayerId],
) -> PlayerSnapshot {
    let is_recipient = player_id == recipient_player_id;
    let economy = economy_for_player(world, player_id);

    PlayerSnapshot {
        player_id,
        class_id: class_for_player(world, player_id),
        gold: economy.map(|economy| economy.gold).unwrap_or(0),
        reserved_gold: economy.map(|economy| economy.reserved_gold).unwrap_or(0),
        current_mana: economy.map(|economy| economy.current_mana).unwrap_or(0),
        reserve_mana: economy.map(|economy| economy.reserve_mana).unwrap_or(0),
        spawn_range_cells: spawn_range_cells(world, player_id),
        mana_cap: economy
            .map(|economy| u32_to_u8(economy.mana_cap))
            .unwrap_or_else(|| default_mana_cap(world)),
        submitted: submitted_for_player(world, player_id),
        hand: is_recipient
            .then(|| hand_for_player(world, player_id))
            .unwrap_or_default(),
        shop_slots: is_recipient
            .then(|| shop_slots_for_player(world, player_id))
            .unwrap_or_default(),
        pool_snapshot: is_recipient
            .then(|| pool_snapshot_for_player(world, player_id))
            .unwrap_or_default(),
        objectives: objective_snapshots_for_player(world, player_id, is_recipient),
        opponent_objectives: is_recipient
            .then(|| opponent_objectives_for_recipient(world, recipient_player_id, players))
            .unwrap_or_default(),
    }
}

fn build_board_snapshot(
    recipient_player_id: PlayerId,
    world: &mut World,
    players: &[PlayerId],
) -> BoardSnapshot {
    BoardSnapshot {
        units: build_unit_board_states(world),
        traps: trap_board_states(recipient_player_id, world),
        structures: structure_board_states(world),
        fields: field_board_states(world),
        prisms: prism_board_states(world, players),
        seeds: seed_board_states(world),
        sinistros: sinistro_states(world),
    }
}

fn session_players(world: &World) -> Option<Vec<PlayerId>> {
    if let Some(config) = world.get_resource::<SessionConfig>() {
        let players = config.players().collect::<Vec<_>>();
        if !players.is_empty() {
            return Some(players);
        }
    }

    let sessions = world.get_resource::<PlayerSessions>()?;
    let mut players = sessions.players.keys().copied().collect::<Vec<_>>();
    players.sort_by_key(|player| player.0);
    (!players.is_empty()).then_some(players)
}

fn class_for_player(world: &World, player_id: PlayerId) -> ClassId {
    world
        .get_resource::<SessionConfig>()
        .and_then(|config| config.class_map.get(&player_id).copied())
        .or_else(|| {
            world.get_resource::<PlayerSessions>().and_then(|sessions| {
                sessions
                    .players
                    .get(&player_id)
                    .map(|session| session.class)
            })
        })
        .unwrap_or(ClassId::Neutral)
}

fn economy_for_player(world: &World, player_id: PlayerId) -> Option<&PlayerEconomy> {
    world
        .get_resource::<PlayerEconomies>()
        .and_then(|economies| economies.0.get(&player_id))
}

fn hand_for_player(world: &World, player_id: PlayerId) -> Vec<CardId> {
    world
        .get_resource::<PlayerHands>()
        .and_then(|hands| hands.hands.get(&player_id).cloned())
        .unwrap_or_default()
}

fn shop_slots_for_player(world: &World, player_id: PlayerId) -> Vec<Option<CardId>> {
    if let Some(slots) = world
        .get_resource::<ShopStates>()
        .and_then(|shops| shops.players.get(&player_id))
        .map(|shop| shop.current_slots.to_vec())
    {
        return slots;
    }

    pool_for_player(world, player_id)
        .map(|pool| pool.shop_slots.clone())
        .unwrap_or_default()
}

fn pool_snapshot_for_player(world: &World, player_id: PlayerId) -> Vec<(CardId, u8)> {
    let mut snapshot = pool_for_player(world, player_id)
        .map(|pool| {
            pool.copies_remaining
                .iter()
                .map(|(card_id, copies)| (*card_id, u32_to_u8(*copies)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    snapshot.sort_by_key(|(card_id, _)| card_id.0);
    snapshot
}

fn pool_for_player(world: &World, player_id: PlayerId) -> Option<&PlayerPool> {
    world
        .get_resource::<PlayerPools>()
        .and_then(|pools| pools.pools.get(&player_id))
}

fn spawn_range_cells(world: &World, player_id: PlayerId) -> u8 {
    let Some(session) = world.get_resource::<SessionConfig>() else {
        return 1;
    };

    world
        .get_resource::<SpawnRangeState>()
        .and_then(|spawn_ranges| spawn_range_cells_for_player(spawn_ranges, player_id, session))
        .unwrap_or(1)
}

fn submitted_for_player(world: &World, player_id: PlayerId) -> bool {
    world
        .get_resource::<RoundState>()
        .map(|state| state.submissions_received.contains(&player_id))
        .unwrap_or(false)
}

fn objective_snapshots_for_player(
    world: &mut World,
    player_id: PlayerId,
    reveal_identity: bool,
) -> Vec<ObjectiveSnapshot> {
    objective_rows(world, player_id)
        .into_iter()
        .map(|row| ObjectiveSnapshot {
            lane: row.lane,
            hp: row.hp,
            is_real: reveal_identity
                && hidden_objective_is_fake(world, player_id, row.lane)
                    .map(|is_fake| !is_fake)
                    .unwrap_or(false),
            is_destroyed: row.is_destroyed,
        })
        .collect()
}

fn opponent_objectives_for_recipient(
    world: &mut World,
    recipient_player_id: PlayerId,
    players: &[PlayerId],
) -> Vec<OpponentObjectiveSnapshot> {
    let mut snapshots = Vec::new();

    for opponent in players.iter().copied() {
        if opponent == recipient_player_id {
            continue;
        }

        for row in objective_rows(world, opponent) {
            snapshots.push(OpponentObjectiveSnapshot {
                lane: row.lane,
                hp: row.hp,
                is_destroyed: row.is_destroyed,
                was_fake: row
                    .is_destroyed
                    .then(|| hidden_objective_is_fake(world, opponent, row.lane))
                    .flatten(),
            });
        }
    }

    snapshots
}

#[derive(Clone, Copy)]
struct ObjectiveRow {
    lane: u8,
    hp: u8,
    is_destroyed: bool,
}

fn objective_rows(world: &mut World, player_id: PlayerId) -> Vec<ObjectiveRow> {
    let default_hp = default_objective_hp(world);
    let mut rows = (1..=OBJECTIVE_LANE_COUNT)
        .map(|lane| ObjectiveRow {
            lane,
            hp: default_hp,
            is_destroyed: false,
        })
        .collect::<Vec<_>>();

    let mut query = world.query::<(&ObjectiveSlot, Option<&ObjectiveHp>)>();
    for (slot, hp) in query.iter(world) {
        if slot.player != player_id {
            continue;
        }

        let lane_index = usize::from(slot.lane.saturating_sub(1));
        if let Some(row) = rows.get_mut(lane_index) {
            row.hp = hp.map(|hp| u32_to_u8(hp.hp)).unwrap_or(default_hp);
            row.is_destroyed = slot.destroyed || row.hp == 0;
        }
    }

    rows
}

fn hidden_objective_is_fake(world: &World, player_id: PlayerId, lane: u8) -> Option<bool> {
    world
        .get_resource::<HiddenObjectives>()
        .and_then(|hidden| hidden.identities.get(&(player_id, lane)).copied())
}

fn trap_board_states(recipient_player_id: PlayerId, world: &World) -> Vec<TrapBoardState> {
    let mut traps = world
        .get_resource::<BoardOccupancy>()
        .map(|occupancy| {
            occupancy
                .traps
                .iter()
                .map(|((owner, lane, cell), entity)| TrapBoardState {
                    trap_id: entity_id(*entity),
                    owner: *owner,
                    lane: *lane,
                    cell: *cell,
                    card_id: (*owner == recipient_player_id)
                        .then(|| card_id(world, *entity))
                        .flatten(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    traps.sort_by_key(|trap| (trap.owner.0, trap.lane, trap.cell, trap.trap_id));
    traps
}

fn structure_board_states(world: &World) -> Vec<StructureBoardState> {
    let mut structures = world
        .get_resource::<BoardOccupancy>()
        .map(|occupancy| {
            occupancy
                .structures
                .iter()
                .map(|((owner, lane, cell), entity)| {
                    let stats = world.get::<UnitStats>(*entity);
                    let hp = stats.map(|stats| stats.hp).unwrap_or(0);
                    StructureBoardState {
                        structure_id: entity_id(*entity),
                        card_id: card_id(world, *entity),
                        owner: *owner,
                        lane: *lane,
                        cell: *cell,
                        max_hp: hp,
                        current_hp: hp,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    structures.sort_by_key(|structure| {
        (
            structure.owner.0,
            structure.lane,
            structure.cell,
            structure.structure_id,
        )
    });
    structures
}

fn field_board_states(world: &World) -> Vec<FieldBoardState> {
    let mut fields = world
        .get_resource::<BoardOccupancy>()
        .map(|occupancy| {
            occupancy
                .fields
                .iter()
                .map(|((owner, lane), entity)| FieldBoardState {
                    field_id: entity_id(*entity),
                    card_id: card_id(world, *entity),
                    owner: *owner,
                    lane: *lane,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    fields.sort_by_key(|field| (field.owner.0, field.lane, field.field_id));
    fields
}

fn prism_board_states(world: &World, players: &[PlayerId]) -> Vec<PrismBoardState> {
    let Some(prisms) = world.get_resource::<PrismState>() else {
        return Vec::new();
    };

    players
        .iter()
        .copied()
        .enumerate()
        .flat_map(|(player_index, player_id)| {
            (0..PRISM_LANE_COUNT).map(move |lane_index| PrismBoardState {
                player_id,
                lane: u8::try_from(lane_index + 1).unwrap_or(u8::MAX),
                collected: prisms
                    .collected
                    .get(player_index)
                    .and_then(|lanes| lanes.get(lane_index))
                    .copied()
                    .unwrap_or(false),
            })
        })
        .collect()
}

fn seed_board_states(world: &mut World) -> Vec<SeedBoardState> {
    let mut query = world.query::<(&SeedOwner, &BoardPosition, &SeedMarker)>();
    let mut seeds = query
        .iter(world)
        .map(|(owner, position, _)| SeedBoardState {
            owner: owner.0,
            lane: position.lane,
            cell: position.cell,
        })
        .collect::<Vec<_>>();
    seeds.sort_by_key(|seed| (seed.owner.0, seed.lane, seed.cell));
    seeds
}

fn sinistro_states(world: &mut World) -> Vec<SinistroState> {
    let mut query = world.query::<(&UnitOwner, &ObjectiveAttachment, &ClassTokenKind)>();
    let mut sinistros = query
        .iter(world)
        .filter_map(|(owner, attachment, kind)| {
            (*kind == ClassTokenKind::Sinistro).then_some(SinistroState {
                owner: owner.0,
                lane: attachment.lane,
                damage: DEFAULT_SINISTRO_DAMAGE,
            })
        })
        .collect::<Vec<_>>();
    sinistros.sort_by_key(|sinistro| (sinistro.owner.0, sinistro.lane));
    sinistros
}

fn card_id(world: &World, entity: Entity) -> Option<CardId> {
    world.get::<UnitCardRef>(entity).map(|card| card.0)
}

fn entity_id(entity: Entity) -> u64 {
    entity.to_bits()
}

fn protocol_round_phase(phase: RoundPhase) -> ProtocolRoundPhase {
    match phase {
        RoundPhase::Lobby => ProtocolRoundPhase::Lobby,
        RoundPhase::DraftInitial => ProtocolRoundPhase::DraftInitial,
        RoundPhase::DraftAuction => ProtocolRoundPhase::DraftAuction,
        RoundPhase::DraftShop => ProtocolRoundPhase::DraftShop,
        RoundPhase::Placement => ProtocolRoundPhase::Placement,
        RoundPhase::Resolution => ProtocolRoundPhase::Resolution,
        RoundPhase::GameOver => ProtocolRoundPhase::GameOver,
    }
}

fn snapshot_timer_remaining_ms(world: &World) -> Option<u32> {
    let round_state = world.get_resource::<RoundState>()?;
    let timer = match round_state.phase {
        RoundPhase::Lobby | RoundPhase::GameOver => None,
        RoundPhase::DraftInitial => round_state.draft_initial_timer.as_ref(),
        RoundPhase::DraftShop => round_state.draft_shop_timer.as_ref(),
        RoundPhase::Placement => round_state.placement_timer.as_ref(),
        RoundPhase::Resolution => round_state.resolution_safety_timer.as_ref(),
        RoundPhase::DraftAuction => {
            return world
                .get_resource::<AuctionState>()
                .map(|state| state.timer_remaining_ms);
        }
    };

    timer.map(timer_remaining_ms)
}

fn timer_remaining_ms(timer: &Timer) -> u32 {
    u32::try_from(timer.remaining().as_millis()).unwrap_or(u32::MAX)
}

fn protocol_version(world: &World) -> u32 {
    world
        .get_resource::<GameConfig>()
        .map(|config| config.protocol_version)
        .unwrap_or_else(|| shared::config::GameConfig::default().protocol_version)
}

fn default_mana_cap(world: &World) -> u8 {
    u32_to_u8(
        world
            .get_resource::<GameConfig>()
            .map(|config| config.mana_cap)
            .unwrap_or_else(|| shared::config::GameConfig::default().mana_cap),
    )
}

fn default_objective_hp(world: &World) -> u8 {
    u32_to_u8(
        world
            .get_resource::<GameConfig>()
            .map(|config| config.objective_hp)
            .unwrap_or_else(|| shared::config::GameConfig::default().objective_hp),
    )
}

fn u32_to_u8(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}
