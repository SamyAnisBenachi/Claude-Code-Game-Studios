//! Client-side dedupe state for late / duplicate reliable S2C messages.
//!
//! Story: `S13-LATE-MSG-DEDUPE-001`. PROMPT 803 §3 DC-6 / §4 Lane C / §5 Should
//! row 1.
//!
//! The reliable Lightyear channel is mostly duplicate-safe at the transport
//! layer, but the server reconnect flow
//! (`server/src/core/session/reconnect.rs:198-233` and `:836-893`) can re-send
//! the same authoritative state — including `S2CGameOver`, `S2CClassLocked`,
//! and `S2CPlacementReveal` — after a client reconnect. The drains for those
//! messages have user-visible side effects (result-screen entry, lobby state
//! mutation, board reveal animation) and previously had no dedupe guard.
//!
//! This module provides a small per-drain dedupe ring that follows the
//! `C2SAcknowledgeResult` precedent at
//! `tests/integration/session/result_acknowledgement_contract_test.rs:91-96`:
//! on duplicate detection, the drain logs DEBUG and returns early without
//! side effect.
//!
//! # Scope
//!
//! - Session-lifetime scope: cleared on `OnExit(ClientState::InSession)` via
//!   [`reset_client_idempotency_on_session_exit_system`]. The reconnect
//!   flow does not exit `InSession`, so the set is preserved across reconnect
//!   replay (AC5).
//! - Bounded size per drain: [`DEDUPE_BOUND`] = 32 keys (AC6). Oldest key
//!   is evicted on overflow.
//!
//! # Authority boundary
//!
//! The dedupe state is part of the read-only client projection. It does not
//! mutate authoritative state and does not introduce optimistic client-side
//! authority. ADR-002 / ADR-008 / ADR-011 binding.

use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use shared::card::ClassId;
use shared::protocol::{
    GameOverReason, PlayTarget, S2CClassLocked, S2CGameOver, S2CPlacementReveal,
};

use crate::state::ClientState;

/// Per-drain dedupe ring upper bound. 32 is well above the worst-case
/// reconnect-replay burst the server can emit per drain (one snapshot,
/// one phase change, one game-over, one class lock, one placement reveal
/// per round with at most a few rounds of buffered tail).
pub const DEDUPE_BOUND: usize = 32;

/// Aggregates the per-drain dedupe rings handed out to each `Update`-stage
/// receive system.
#[derive(Resource, Default, Debug, Clone)]
pub struct ClientIdempotencyState {
    pub game_over: DedupeRing<GameOverDedupeKey>,
    pub class_locked: DedupeRing<ClassLockedDedupeKey>,
    pub placement_reveal: DedupeRing<PlacementRevealDedupeKey>,
}

impl ClientIdempotencyState {
    pub fn clear_for_session_exit(&mut self) {
        self.game_over.clear();
        self.class_locked.clear();
        self.placement_reveal.clear();
    }
}

/// Bounded FIFO ring of seen keys with O(1) lookup. Insertion past the bound
/// evicts the oldest key.
#[derive(Debug, Clone)]
pub struct DedupeRing<K: Clone + Eq + Hash> {
    queue: VecDeque<K>,
    set: HashSet<K>,
    bound: usize,
}

impl<K: Clone + Eq + Hash> Default for DedupeRing<K> {
    fn default() -> Self {
        Self::with_bound(DEDUPE_BOUND)
    }
}

impl<K: Clone + Eq + Hash> DedupeRing<K> {
    pub fn with_bound(bound: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(bound),
            set: HashSet::with_capacity(bound),
            bound,
        }
    }

    /// Returns `true` if `key` is fresh (was inserted) and `false` if it is a
    /// duplicate (already in the ring; not inserted again).
    pub fn check_and_insert(&mut self, key: K) -> bool {
        if self.set.contains(&key) {
            return false;
        }
        if self.queue.len() >= self.bound {
            if let Some(oldest) = self.queue.pop_front() {
                self.set.remove(&oldest);
            }
        }
        self.queue.push_back(key.clone());
        self.set.insert(key);
        true
    }

    pub fn contains(&self, key: &K) -> bool {
        self.set.contains(key)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.set.clear();
    }
}

/// Canonical dedupe key for `S2CGameOver`. The reliable channel guarantees
/// byte-identity of replayed messages, so reducing the message to its
/// `(round, reason, loser)` tuple is sufficient and avoids a full content
/// hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameOverDedupeKey {
    pub round: u32,
    pub reason_index: u8,
    /// 0 = `None`; otherwise the wrapped `PlayerId.0`. `PlayerId` reserves
    /// non-zero ids in the lobby allocator
    /// (`server/src/core/session/lobby.rs`), so collision with a real id is
    /// impossible.
    pub loser_raw: u64,
}

impl GameOverDedupeKey {
    pub fn from_message(msg: &S2CGameOver) -> Self {
        Self {
            round: msg.round,
            reason_index: game_over_reason_index(msg.reason),
            loser_raw: msg.loser.map(|p| p.0).unwrap_or(0),
        }
    }
}

fn game_over_reason_index(reason: GameOverReason) -> u8 {
    match reason {
        GameOverReason::ObjectivesDestroyed => 0,
        GameOverReason::Disconnect => 1,
        GameOverReason::Draw => 2,
        GameOverReason::ResolutionTimeout => 3,
    }
}

/// Canonical dedupe key for `S2CClassLocked`. The lobby allows at most one
/// confirmed class per session and the message body is a single `ClassId`,
/// so the `ClassId` itself is the key. Lobby exits via `OnExit(InSession)`
/// reset clear this set so a fresh game starts with no carry-over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassLockedDedupeKey {
    pub class_id: ClassId,
}

impl ClassLockedDedupeKey {
    pub fn from_message(msg: &S2CClassLocked) -> Self {
        Self {
            class_id: msg.class_id,
        }
    }
}

/// Canonical dedupe key for `S2CPlacementReveal`. `S2CPlacementReveal` does
/// not carry an explicit round field; the round is sourced from
/// [`crate::state::CurrentClientPhase`] at drain time. The digest is a
/// process-stable `u64` derived from the placement vector — `DefaultHasher`
/// (`SipHasher13`) is fixed-seed at construction and identical across two
/// invocations on the same content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacementRevealDedupeKey {
    pub round: u32,
    pub digest: u64,
}

impl PlacementRevealDedupeKey {
    pub fn from_message(round: u32, msg: &S2CPlacementReveal) -> Self {
        Self {
            round,
            digest: digest_placement_reveal(msg),
        }
    }
}

fn digest_placement_reveal(msg: &S2CPlacementReveal) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    msg.placements.len().hash(&mut hasher);
    for placement in &msg.placements {
        placement.owner_id.hash(&mut hasher);
        placement.card_id.0.hash(&mut hasher);
        match &placement.target {
            PlayTarget::BoardCell { lane, cell } => {
                0u8.hash(&mut hasher);
                lane.hash(&mut hasher);
                cell.hash(&mut hasher);
            }
            PlayTarget::TargetUnit { lane, unit_id } => {
                1u8.hash(&mut hasher);
                lane.hash(&mut hasher);
                unit_id.hash(&mut hasher);
            }
            PlayTarget::TargetObj { player_id, lane } => {
                2u8.hash(&mut hasher);
                player_id.hash(&mut hasher);
                lane.hash(&mut hasher);
            }
            PlayTarget::LaneWide { lane } => {
                3u8.hash(&mut hasher);
                lane.hash(&mut hasher);
            }
            PlayTarget::Instant => {
                4u8.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// `OnExit(InSession)` system: clear all dedupe rings so a returning lobby
/// session starts with no carry-over.
pub fn reset_client_idempotency_on_session_exit_system(
    mut idempotency: ResMut<ClientIdempotencyState>,
) {
    idempotency.clear_for_session_exit();
}

/// Bevy plugin that owns the dedupe resource lifecycle. Adding it once at
/// app build time wires the `OnExit(InSession)` clear and ensures every
/// drain that imports `Res<ClientIdempotencyState>` shares the same instance.
pub struct ClientIdempotencyPlugin;

impl Plugin for ClientIdempotencyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClientIdempotencyState>().add_systems(
            OnExit(ClientState::InSession),
            reset_client_idempotency_on_session_exit_system,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::card::CardId;
    use shared::protocol::{PlacedCardReveal, PlayTarget};
    use shared::session::PlayerId;

    fn placement(owner: u64, card: u32, lane: u8, cell: u8) -> PlacedCardReveal {
        PlacedCardReveal {
            owner_id: PlayerId(owner),
            card_id: CardId(card),
            target: PlayTarget::BoardCell { lane, cell },
        }
    }

    #[test]
    fn dedupe_ring_inserts_unique_returns_true() {
        let mut ring = DedupeRing::<u32>::with_bound(4);
        assert!(ring.check_and_insert(1));
        assert!(ring.check_and_insert(2));
        assert_eq!(ring.len(), 2);
    }

    #[test]
    fn dedupe_ring_duplicate_returns_false_and_does_not_grow() {
        let mut ring = DedupeRing::<u32>::with_bound(4);
        assert!(ring.check_and_insert(7));
        assert!(!ring.check_and_insert(7));
        assert!(!ring.check_and_insert(7));
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn dedupe_ring_evicts_oldest_when_bound_exceeded() {
        let mut ring = DedupeRing::<u32>::with_bound(2);
        assert!(ring.check_and_insert(1));
        assert!(ring.check_and_insert(2));
        assert!(ring.check_and_insert(3));
        assert!(!ring.contains(&1), "oldest key 1 must have been evicted");
        assert!(ring.contains(&2));
        assert!(ring.contains(&3));
        assert_eq!(ring.len(), 2);
    }

    #[test]
    fn dedupe_ring_clear_empties_state() {
        let mut ring = DedupeRing::<u32>::with_bound(4);
        ring.check_and_insert(1);
        ring.check_and_insert(2);
        ring.clear();
        assert!(ring.is_empty());
        assert!(
            ring.check_and_insert(1),
            "post-clear must accept previously seen key"
        );
    }

    #[test]
    fn game_over_key_distinguishes_round_reason_loser() {
        let base = S2CGameOver {
            loser: Some(PlayerId(7)),
            round: 3,
            reason: GameOverReason::ObjectivesDestroyed,
        };
        let same = base.clone();
        let diff_round = S2CGameOver {
            round: 4,
            ..base.clone()
        };
        let diff_reason = S2CGameOver {
            reason: GameOverReason::Disconnect,
            ..base.clone()
        };
        let diff_loser = S2CGameOver {
            loser: Some(PlayerId(9)),
            ..base.clone()
        };

        let key = GameOverDedupeKey::from_message(&base);
        assert_eq!(key, GameOverDedupeKey::from_message(&same));
        assert_ne!(key, GameOverDedupeKey::from_message(&diff_round));
        assert_ne!(key, GameOverDedupeKey::from_message(&diff_reason));
        assert_ne!(key, GameOverDedupeKey::from_message(&diff_loser));
    }

    #[test]
    fn placement_reveal_key_is_stable_for_identical_payload() {
        let msg = S2CPlacementReveal {
            placements: vec![placement(1, 10, 1, 0), placement(2, 11, 3, 4)],
        };
        let again = msg.clone();
        let k1 = PlacementRevealDedupeKey::from_message(5, &msg);
        let k2 = PlacementRevealDedupeKey::from_message(5, &again);
        assert_eq!(k1, k2);
    }

    #[test]
    fn placement_reveal_key_changes_with_round_or_payload() {
        let msg = S2CPlacementReveal {
            placements: vec![placement(1, 10, 1, 0)],
        };
        let other_round = PlacementRevealDedupeKey::from_message(6, &msg);
        let same_round = PlacementRevealDedupeKey::from_message(5, &msg);
        assert_ne!(other_round, same_round);

        let mutated = S2CPlacementReveal {
            placements: vec![placement(1, 10, 2, 0)],
        };
        assert_ne!(
            PlacementRevealDedupeKey::from_message(5, &msg),
            PlacementRevealDedupeKey::from_message(5, &mutated)
        );
    }
}
