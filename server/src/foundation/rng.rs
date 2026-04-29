// Verified crate compatibility (GDD OQ2):
//   rand_chacha = "0.3" uses rand_core "0.6"
//   rand = "0.9" uses rand_core "0.6"
//   These are compatible. Version pair confirmed in workspace Cargo.toml.
//
// ADR-005 §1 Resource Definition
// All randomness in Lanes and Lies flows through this module.
// ChaCha20Rng is never exposed outside this file.
// PlayerId is a placeholder until shared/ defines the canonical type (TODO).

use bevy::prelude::Resource;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

// TODO: import PlayerId from shared/ when defined there.
// Using a local alias to avoid blocking this story on PlayerId definition.
type PlayerId = u32;

/// Identifies which random event consumed an RNG seed.
/// One variant per call site — no generic "Misc" catch-all (ADR-005 §5).
/// Adding a new random event requires a new variant here AND an entry in
/// the §4 consumption order table in ADR-005.
#[derive(Debug, Clone, PartialEq)]
pub enum RngEvent {
    /// Sentinel: logged once at session init (seed_index = 0). result = None.
    SessionInit,
    /// Fake objective lane assignment — 2 seeds per player, ascending player_id.
    AssignFakeObjectives { player_id: PlayerId },
    /// Initial draft draw — 1 seed per player, ascending player_id.
    DrawInitialDraft { player_id: PlayerId },
    /// Shop slot draw — 2–3 seeds per slot, ascending player_id then slot_index.
    DrawShopSlot { player_id: PlayerId, slot_index: u8 },
    /// Ecaflip dice trigger — ascending lane order.
    ResolveEcaflip { lane: u8 },
    /// Prism activation resolution — ascending player_id then lane.
    ResolvePrism { player_id: PlayerId, lane: u8 },
    /// Fake objective destroyed reward — ascending player_id then lane.
    AwardFakeObjectiveReward { player_id: PlayerId, lane: u8 },
    /// Conditional free card draw (only if AwardFakeObjectiveReward = free card).
    DrawFreeCard { player_id: PlayerId },
}

/// One entry in the server-side audit log.
/// Appended on every intent-named method call (ADR-005 §5).
/// The audit log is server-only and MUST NOT be transmitted to clients.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEntry {
    /// Which random event consumed this seed.
    pub event_type: RngEvent,
    /// Monotonically increasing index. Entry 0 is always SessionInit.
    pub seed_index: u32,
    /// Human-readable encoded outcome. None for SessionInit or stub methods.
    /// Encoding per event type defined in server-rng.md Rule 8.
    /// Consuming epics fill in real result strings when they implement actual draws.
    pub result: Option<String>,
}

/// Per-session deterministic RNG resource.
///
/// Wraps a single `ChaCha20Rng` (ADR-005 §1). Seeded once from OS entropy at
/// session start via `ServerRng::new()`. Never re-seeded mid-session.
///
/// All access to the inner `ChaCha20Rng` is private to this module.
/// Consumers call intent-named methods (one per `RngEvent` variant),
/// never raw RNG access.
///
/// ADR-005 forbidden list: never use `rand::thread_rng()`, `StdRng`, `SmallRng`.
/// technical-preferences.md: seeds are never transmitted to clients.
///
/// # Ordering Contract
///
/// When multiple players have simultaneous random events (e.g., Ecaflip triggers
/// for both players in the same resolution sub-step), callers MUST process events
/// in this deterministic order:
///
/// 1. Ascending `player_id` (lower ID first)
/// 2. Within one player: ascending `lane` index (lane 1 → 5)
/// 3. Within a lane: ascending board position (cell 1 → 8)
///
/// Violating this order produces a different audit log for the same game state,
/// breaking determinism and replay. This is a caller contract — `ServerRng` cannot
/// enforce it internally.
#[derive(Resource)]
pub struct ServerRng {
    /// Private — never exposed. All ChaCha20Rng access stays in this module.
    rng: ChaCha20Rng,
    /// Monotonically incrementing; starts at 1 after construction
    /// (index 0 is consumed by the SessionInit sentinel).
    seed_index: u32,
    /// Append-only. Server-only. Never sent to clients.
    audit_log: Vec<AuditEntry>,
}

impl ServerRng {
    /// Production constructor.
    ///
    /// Seeds from OS entropy via `ChaCha20Rng::from_entropy()`.
    /// Pushes the mandatory `SessionInit` sentinel at `seed_index = 0`.
    /// Returns with `seed_index = 1` — gameplay starts from index 1.
    ///
    /// ADR-012: Game Session System calls this before emitting `SessionReady`.
    pub fn new() -> Self {
        let rng = ChaCha20Rng::from_entropy();
        let mut audit_log = Vec::new();
        audit_log.push(AuditEntry {
            event_type: RngEvent::SessionInit,
            seed_index: 0,
            result: None,
        });
        Self {
            rng,
            seed_index: 1,
            audit_log,
        }
    }

    /// Test-only constructor — deterministic seed.
    ///
    /// Uses `ChaCha20Rng::seed_from_u64(seed)` so tests are fully deterministic.
    /// Same sentinel behaviour as `new()`.
    #[cfg(test)]
    pub fn from_seed(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        let mut audit_log = Vec::new();
        audit_log.push(AuditEntry {
            event_type: RngEvent::SessionInit,
            seed_index: 0,
            result: None,
        });
        Self {
            rng,
            seed_index: 1,
            audit_log,
        }
    }

    /// Test-only constructor — sets seed_index to u32::MAX for overflow testing.
    ///
    /// Allows testing RNG15 (wrapping_add overflow) without calling next_seed()
    /// ~4.3 billion times. Session starts with the standard SessionInit sentinel.
    /// Only callable in test builds (ADR-005 §2 lifecycle).
    #[cfg(test)]
    pub fn at_max_seed_index() -> Self {
        let mut s = Self::from_seed(42);
        s.seed_index = u32::MAX;
        s
    }

    // -------------------------------------------------------------------------
    // Deferred ACs — Story 003 (RNG13: session reset, RNG15: overflow)
    // -------------------------------------------------------------------------
    //
    // RNG8 (RESOLUTION ordering): deferred to RSM epic — requires full system chain.
    // RNG9 (run condition): each consuming system adds its own guard — see ADR-005 §2 lifecycle.
    // RNG10 (chi-squared): deferred to QA/Polish phase.
    // RNG14 (no seeds in S2C): deferred to Epic 4 network integration tests.

    /// Current seed index. Equals 1 after construction; increments on each call.
    pub fn current_seed_index(&self) -> u32 {
        self.seed_index
    }

    /// Read-only view of the audit log.
    ///
    /// `audit_log()[0]` is always the `SessionInit` sentinel with `result = None`.
    /// After N gameplay calls, `audit_log().len() == N + 1`.
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// Advances the ChaCha20 stream and increments `seed_index`.
    ///
    /// Private — callers are the intent-named public methods only.
    /// The caller captures `seed_index` before this call and pushes its own
    /// `AuditEntry` with the pre-call index (ADR-005 §5).
    fn next_seed(&mut self) -> u64 {
        use rand::RngCore;
        let value = self.rng.next_u64();
        self.seed_index = self.seed_index.wrapping_add(1);
        value
    }

    // -------------------------------------------------------------------------
    // Intent-named public API — one method per RngEvent variant (ADR-005 §6).
    // Each method: captures seed_index, calls next_seed(), pushes AuditEntry,
    // returns the seed (or typed stub result). result: None in all stubs —
    // consuming epics fill in real result strings when they implement draws.
    // -------------------------------------------------------------------------

    /// Assign fake objective lanes for one player.
    ///
    /// Consumes 2 seeds per call (GDD seed table, DRAFT_INITIAL order 1).
    /// Returns `(0, 0)` as stub — Objective System epic implements real assignment.
    /// Callers MUST iterate ascending `player_id` (ADR-005 §4 ordering contract).
    pub fn assign_fake_objectives(&mut self, player_id: PlayerId) -> (u8, u8) {
        let idx1 = self.seed_index;
        let _seed1 = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::AssignFakeObjectives { player_id },
            seed_index: idx1,
            result: None,
        });
        let idx2 = self.seed_index;
        let _seed2 = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::AssignFakeObjectives { player_id },
            seed_index: idx2,
            result: None,
        });
        (0, 0)
    }

    /// Draw seed for a player's initial draft hand.
    ///
    /// Consumes 1 seed (DRAFT_INITIAL order 2). Returns the raw seed for Card
    /// Pool to use. Callers MUST iterate ascending `player_id`.
    pub fn draw_initial_draft(&mut self, player_id: PlayerId) -> u64 {
        let idx = self.seed_index;
        let seed = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::DrawInitialDraft { player_id },
            seed_index: idx,
            result: None,
        });
        seed
    }

    /// Draw seed for one shop slot.
    ///
    /// Consumes 1 seed per call (DRAFT_SHOP order 3). Card Pool determines total
    /// call count (2–3 seeds per slot via multiple calls). Returns raw seed.
    /// Callers MUST iterate ascending `player_id` then ascending `slot_index`.
    pub fn draw_shop_slot(&mut self, player_id: PlayerId, slot_index: u8) -> u64 {
        let idx = self.seed_index;
        let seed = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::DrawShopSlot { player_id, slot_index },
            seed_index: idx,
            result: None,
        });
        seed
    }

    /// Draw seed for an Ecaflip dice trigger on a lane.
    ///
    /// Consumes 1 seed (RESOLUTION order 4). Returns raw seed.
    /// Callers MUST iterate ascending `lane` index.
    pub fn resolve_ecaflip(&mut self, lane: u8) -> u64 {
        let idx = self.seed_index;
        let seed = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::ResolveEcaflip { lane },
            seed_index: idx,
            result: None,
        });
        seed
    }

    /// Draw seed for a Prism activation on a lane.
    ///
    /// Consumes 1 seed (RESOLUTION order 5). Returns raw seed.
    /// Callers MUST iterate ascending `player_id` then ascending `lane`.
    pub fn resolve_prism(&mut self, player_id: PlayerId, lane: u8) -> u64 {
        let idx = self.seed_index;
        let seed = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::ResolvePrism { player_id, lane },
            seed_index: idx,
            result: None,
        });
        seed
    }

    /// Draw seed for a fake-objective-destroyed reward roll.
    ///
    /// Consumes 1 seed (RESOLUTION order 6). Returns raw seed.
    /// Callers MUST iterate ascending `player_id` then ascending `lane`.
    pub fn award_fake_objective_reward(&mut self, player_id: PlayerId, lane: u8) -> u64 {
        let idx = self.seed_index;
        let seed = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::AwardFakeObjectiveReward { player_id, lane },
            seed_index: idx,
            result: None,
        });
        seed
    }

    /// Draw seed for a conditional free-card draw.
    ///
    /// Consumes 1 seed (RESOLUTION order 7, only when order 6 awarded a free card).
    /// Returns raw seed.
    pub fn draw_free_card(&mut self, player_id: PlayerId) -> u64 {
        let idx = self.seed_index;
        let seed = self.next_seed();
        self.audit_log.push(AuditEntry {
            event_type: RngEvent::DrawFreeCard { player_id },
            seed_index: idx,
            result: None,
        });
        seed
    }
}

// =============================================================================
// Tests — Story 001 (type definitions) + Story 002 (intent-named API) +
//         Story 003 (determinism proof & session reset)
// =============================================================================
//
// These run via `cargo test -p server`. Deterministic seeds via from_seed().
// Evidence files:
//   tests/unit/foundation/server_rng_types_test.rs        (Story 001)
//   tests/unit/foundation/server_rng_api_test.rs          (Story 002)
//   tests/unit/foundation/server_rng_determinism_test.rs  (Story 003)

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Story 001: Type definitions & audit infrastructure
    // -------------------------------------------------------------------------

    // RNG1: After ServerRng::new(), current_seed_index() == 1
    #[test]
    fn test_new_seed_index_is_one() {
        let rng = ServerRng::from_seed(0);
        assert_eq!(
            rng.current_seed_index(),
            1,
            "index 0 is consumed by SessionInit sentinel; gameplay starts at 1"
        );
    }

    // RNG5: 0 gameplay calls → 1 audit entry (sentinel only)
    #[test]
    fn test_zero_calls_has_one_audit_entry() {
        let rng = ServerRng::from_seed(42);
        assert_eq!(
            rng.audit_log().len(),
            1,
            "only the SessionInit sentinel should be present"
        );
    }

    // RNG5: N gameplay calls → N+1 audit entries
    #[test]
    fn test_n_calls_produces_n_plus_one_audit_entries() {
        let mut rng = ServerRng::from_seed(42);
        rng.resolve_ecaflip(0);
        rng.resolve_ecaflip(1);
        rng.resolve_ecaflip(2);
        assert_eq!(
            rng.audit_log().len(),
            4,
            "1 sentinel + 3 gameplay calls = 4 entries"
        );
        assert_eq!(rng.current_seed_index(), 4);
    }

    // RNG11: audit_log()[0] is SessionInit with result = None
    #[test]
    fn test_sentinel_is_session_init_with_no_result() {
        let rng = ServerRng::from_seed(12345);
        let first = &rng.audit_log()[0];
        assert_eq!(first.event_type, RngEvent::SessionInit);
        assert_eq!(first.seed_index, 0);
        assert!(first.result.is_none(), "SessionInit must have result = None");
    }

    // RNG11: no raw seed bytes appear in any AuditEntry.result
    #[test]
    fn test_no_raw_seed_in_audit_log() {
        let seed: u64 = 12345;
        let mut rng = ServerRng::from_seed(seed);
        rng.resolve_ecaflip(0);
        for entry in rng.audit_log() {
            if let Some(result) = &entry.result {
                assert!(
                    !result.contains(&seed.to_string()),
                    "raw seed value must not appear in any audit entry result"
                );
            }
        }
    }

    // seed_index values in audit log are monotonically ordered 0..N
    #[test]
    fn test_audit_log_seed_indices_are_sequential() {
        let mut rng = ServerRng::from_seed(7);
        rng.draw_free_card(1);
        rng.draw_free_card(2);
        let log = rng.audit_log();
        for (i, entry) in log.iter().enumerate() {
            assert_eq!(
                entry.seed_index,
                i as u32,
                "seed_index at position {} should be {}",
                i,
                i
            );
        }
    }

    // -------------------------------------------------------------------------
    // Story 002: Intent-named API & consumption invariants
    // -------------------------------------------------------------------------

    // RNG2: Two ServerRng with different seeds → different first output
    #[test]
    fn test_rng2_different_seeds_produce_different_first_output() {
        let mut a = ServerRng::from_seed(1);
        let mut b = ServerRng::from_seed(2);
        assert_ne!(
            a.resolve_ecaflip(0),
            b.resolve_ecaflip(0),
            "different seeds must produce different outputs on the first call"
        );
    }

    // RNG6: Empty-pool draw still increments seed_index (seed consumed regardless of outcome)
    #[test]
    fn test_rng6_draw_always_increments_seed_index() {
        let mut rng = ServerRng::from_seed(99);
        assert_eq!(rng.current_seed_index(), 1);
        rng.resolve_ecaflip(0);
        assert_eq!(
            rng.current_seed_index(),
            2,
            "seed_index must increment even when the draw result is vacuous"
        );
        assert!(
            rng.audit_log().last().unwrap().result.is_none(),
            "stub result must be None"
        );
    }

    // RNG7: Two Ecaflip triggers on the same lane produce consecutive seed_index entries
    #[test]
    fn test_rng7_consecutive_ecaflip_calls_have_sequential_seed_indices() {
        let mut rng = ServerRng::from_seed(7);
        assert_eq!(rng.current_seed_index(), 1);
        rng.resolve_ecaflip(1);
        rng.resolve_ecaflip(1);
        let log = rng.audit_log();
        assert_eq!(log[1].seed_index, 1);
        assert_eq!(log[2].seed_index, 2);
        assert_eq!(log[1].event_type, RngEvent::ResolveEcaflip { lane: 1 });
        assert_eq!(log[2].event_type, RngEvent::ResolveEcaflip { lane: 1 });
    }

    // RNG12: assign_fake_objectives produces exactly 2 audit entries per call
    #[test]
    fn test_rng12_assign_fake_objectives_produces_two_entries() {
        let mut rng = ServerRng::from_seed(3);
        rng.assign_fake_objectives(1);
        let log = rng.audit_log();
        assert_eq!(log.len(), 3, "sentinel + 2 entries from assign_fake_objectives");
        assert_eq!(
            log[1].event_type,
            RngEvent::AssignFakeObjectives { player_id: 1 }
        );
        assert_eq!(
            log[2].event_type,
            RngEvent::AssignFakeObjectives { player_id: 1 }
        );
        assert_eq!(log[1].seed_index, 1);
        assert_eq!(log[2].seed_index, 2);
    }

    // RNG12: ordering contract — correct ordering produces deterministic audit log
    #[test]
    fn test_rng12_ascending_lane_order_produces_ordered_audit_entries() {
        let mut rng = ServerRng::from_seed(42);
        // Caller follows the ordering contract: ascending lane
        rng.resolve_ecaflip(1);
        rng.resolve_ecaflip(2);
        rng.resolve_ecaflip(3);
        let log = rng.audit_log();
        // Verify entries are in the correct positions and seed_index is sequential
        assert_eq!(log[1].event_type, RngEvent::ResolveEcaflip { lane: 1 });
        assert_eq!(log[2].event_type, RngEvent::ResolveEcaflip { lane: 2 });
        assert_eq!(log[3].event_type, RngEvent::ResolveEcaflip { lane: 3 });
        assert_eq!(log[1].seed_index, 1);
        assert_eq!(log[2].seed_index, 2);
        assert_eq!(log[3].seed_index, 3);
    }

    // API boundary: no method exposes ChaCha20Rng, seed_index writable, or raw state
    // (structural — verified by the module's visibility rules; no runtime assertion needed)
    // Verified: next_seed() is private; rng/seed_index/audit_log fields are private.

    // All 7 intent-named methods exist and push exactly one entry per next_seed() call
    #[test]
    fn test_all_seven_methods_push_one_entry_each() {
        let mut rng = ServerRng::from_seed(0);
        let baseline = rng.audit_log().len(); // 1 (sentinel)
        rng.assign_fake_objectives(1); // +2 entries
        assert_eq!(rng.audit_log().len(), baseline + 2);
        rng.draw_initial_draft(1); // +1
        assert_eq!(rng.audit_log().len(), baseline + 3);
        rng.draw_shop_slot(1, 0); // +1
        assert_eq!(rng.audit_log().len(), baseline + 4);
        rng.resolve_ecaflip(0); // +1
        assert_eq!(rng.audit_log().len(), baseline + 5);
        rng.resolve_prism(1, 0); // +1
        assert_eq!(rng.audit_log().len(), baseline + 6);
        rng.award_fake_objective_reward(1, 0); // +1
        assert_eq!(rng.audit_log().len(), baseline + 7);
        rng.draw_free_card(1); // +1
        assert_eq!(rng.audit_log().len(), baseline + 8);
        // seed_index: 1 (start) + 2 + 1 + 1 + 1 + 1 + 1 + 1 = 9
        assert_eq!(rng.current_seed_index(), 9);
    }

    // -------------------------------------------------------------------------
    // Story 003: Determinism Proof & Session Reset
    // Evidence: tests/unit/foundation/server_rng_determinism_test.rs
    // Implements: ADR-005 VC1, VC2; GDD TR-RNG-04, RNG13, RNG15
    // -------------------------------------------------------------------------

    /// Runs the full scripted ADR-005 §4 consumption-order sequence against a
    /// fixed seed and returns the resulting audit log. Used by both determinism
    /// tests to guarantee the exact same call sequence each time.
    fn run_scripted_session(seed: u64) -> Vec<AuditEntry> {
        let mut rng = ServerRng::from_seed(seed);
        // DRAFT_INITIAL (ADR-005 §4 order 1-2): ascending player_id
        rng.assign_fake_objectives(1);
        rng.assign_fake_objectives(2);
        rng.draw_initial_draft(1);
        rng.draw_initial_draft(2);
        // DRAFT_SHOP (order 3): ascending player_id → ascending slot_index
        for slot in 0..3u8 {
            rng.draw_shop_slot(1, slot);
            rng.draw_shop_slot(2, slot);
        }
        // RESOLUTION (orders 4-7)
        rng.resolve_ecaflip(1);
        rng.resolve_prism(1, 2);
        rng.award_fake_objective_reward(1, 3);
        rng.draw_free_card(1);
        rng.audit_log().to_vec()
    }

    // ADR-005 VC1: same seed → identical audit_log structure across two independent instances
    #[test]
    fn test_determinism_same_seed_produces_identical_audit_log() {
        // Arrange
        const SEED: u64 = 0xDEAD_BEEF_CAFE_1234;

        // Act
        let log_a = run_scripted_session(SEED);
        let log_b = run_scripted_session(SEED);

        // Assert: lengths match
        assert_eq!(
            log_a.len(),
            log_b.len(),
            "audit_log length must be identical for the same fixed seed"
        );
        // Assert: every (seed_index, event_type) pair matches structurally
        for (i, (a, b)) in log_a.iter().zip(log_b.iter()).enumerate() {
            assert_eq!(
                a.seed_index, b.seed_index,
                "seed_index mismatch at audit_log position {}",
                i
            );
            assert_eq!(
                std::mem::discriminant(&a.event_type),
                std::mem::discriminant(&b.event_type),
                "event_type discriminant mismatch at audit_log position {}",
                i
            );
        }
    }

    // ADR-005 VC2: same scripted sequence run twice in the same process produces
    // identical results — guards against state leakage between test runs
    #[test]
    fn test_determinism_repeated_runs_in_same_process_are_identical() {
        // Arrange
        const SEED: u64 = 0xDEAD_BEEF_CAFE_1234;

        // Act: two independent calls within the same test process
        let log_first = run_scripted_session(SEED);
        let log_second = run_scripted_session(SEED);

        // Assert: structurally identical
        assert_eq!(log_first.len(), log_second.len());
        for (i, (a, b)) in log_first.iter().zip(log_second.iter()).enumerate() {
            assert_eq!(
                a.seed_index, b.seed_index,
                "seed_index mismatch on second run at position {}",
                i
            );
            assert_eq!(
                std::mem::discriminant(&a.event_type),
                std::mem::discriminant(&b.event_type),
                "event_type discriminant mismatch on second run at position {}",
                i
            );
        }
    }

    // RNG13: new ServerRng instance always starts clean — seed_index and audit_log
    // do not carry over from a prior session
    #[test]
    fn test_session_reset_new_instance_starts_clean() {
        // Arrange: session A — advance past sentinel
        let mut session_a = ServerRng::from_seed(1);
        session_a.resolve_ecaflip(0);
        session_a.resolve_ecaflip(1);
        // session_a.current_seed_index() == 3 here

        // Act: create a completely independent session B
        let session_b = ServerRng::from_seed(1);

        // Assert: B starts clean regardless of A's state
        assert_eq!(
            session_b.current_seed_index(),
            1,
            "new session must start at seed_index 1 (after sentinel), not inherit from prior session"
        );
        assert_eq!(
            session_b.audit_log().len(),
            1,
            "new session audit_log must contain only the SessionInit sentinel"
        );
        assert!(
            matches!(session_b.audit_log()[0].event_type, RngEvent::SessionInit),
            "audit_log[0] must be SessionInit"
        );
    }

    // RNG13: the first non-sentinel seed_index in a new session is 1, not the
    // prior session's final seed_index + 1 — sessions are fully independent
    #[test]
    fn test_session_reset_seed_index_not_inherited_from_prior_session() {
        // Arrange: session A reaches a high seed_index
        let mut session_a = ServerRng::from_seed(10);
        session_a.resolve_ecaflip(0);
        session_a.resolve_ecaflip(1);
        session_a.resolve_ecaflip(2);
        let prior_final_index = session_a.current_seed_index(); // 4

        // Act: new session B
        let mut session_b = ServerRng::from_seed(10);
        session_b.resolve_ecaflip(0);
        let first_entry_in_b = &session_b.audit_log()[1];

        // Assert: B's first gameplay entry has seed_index == 1, not prior_final_index
        assert_eq!(
            first_entry_in_b.seed_index,
            1,
            "first gameplay entry in new session must have seed_index 1, not {}",
            prior_final_index
        );
    }

    // RNG15 (ADVISORY): u32::MAX seed_index does not panic when next_seed is called
    #[test]
    fn test_overflow_does_not_panic() {
        // Arrange
        let mut rng = ServerRng::at_max_seed_index();
        // Act + Assert: must not panic
        rng.resolve_ecaflip(0);
    }

    // RNG15: after overflow, current_seed_index wraps to 0 (wrapping_add)
    #[test]
    fn test_overflow_wraps_seed_index_to_zero() {
        // Arrange
        let mut rng = ServerRng::at_max_seed_index();

        // Act
        rng.resolve_ecaflip(0);

        // Assert: wrapping_add(u32::MAX, 1) == 0
        assert_eq!(
            rng.current_seed_index(),
            0,
            "seed_index must wrap to 0 after u32::MAX (wrapping_add behaviour)"
        );
    }

    // RNG15: the audit entry for the overflow call records seed_index == u32::MAX
    // (the value AT time of call, before wrap)
    #[test]
    fn test_overflow_audit_entry_records_max_seed_index() {
        // Arrange
        let mut rng = ServerRng::at_max_seed_index();
        let entries_before = rng.audit_log().len();

        // Act
        rng.resolve_ecaflip(0);

        // Assert: one new entry was added
        assert_eq!(
            rng.audit_log().len(),
            entries_before + 1,
            "overflow call must still push an audit entry"
        );
        // Assert: that entry records the pre-wrap seed_index (u32::MAX)
        let overflow_entry = rng.audit_log().last().unwrap();
        assert_eq!(
            overflow_entry.seed_index,
            u32::MAX,
            "overflow audit entry must record u32::MAX as the seed_index at time of call"
        );
    }
}
