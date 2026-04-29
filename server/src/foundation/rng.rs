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
/// Appended on every next_seed() call (ADR-005 §5).
/// The audit log is server-only and MUST NOT be transmitted to clients.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Which random event consumed this seed.
    pub event_type: RngEvent,
    /// Monotonically increasing index. Entry 0 is always SessionInit.
    pub seed_index: u32,
    /// Human-readable encoded outcome. None for SessionInit or empty-pool draws.
    /// Encoding per event type defined in server-rng.md Rule 8.
    /// Story 002 fills in real result strings when intent-named methods are added.
    pub result: Option<String>,
}

/// Per-session deterministic RNG resource.
///
/// Wraps a single `ChaCha20Rng` (ADR-005 §1). Seeded once from OS entropy at
/// session start via `ServerRng::new()`. Never re-seeded mid-session.
///
/// All access to the inner `ChaCha20Rng` is private to this module.
/// Consumers call intent-named methods (Story 002), not raw RNG access.
///
/// ADR-005 forbidden list: never use `rand::thread_rng()`, `StdRng`, `SmallRng`.
/// technical-preferences.md: seeds are never transmitted to clients.
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

    /// Internal seed advancement. Private to this module.
    ///
    /// Advances the ChaCha20 stream, appends an audit entry, increments seed_index.
    /// Intent-named public methods (Story 002) call this and supply the event_type
    /// and result encoding. No external code calls this directly.
    pub(crate) fn next_seed(&mut self, event_type: RngEvent, result: Option<String>) -> u64 {
        use rand::RngCore;
        let value = self.rng.next_u64();
        self.audit_log.push(AuditEntry {
            event_type,
            seed_index: self.seed_index,
            result,
        });
        self.seed_index = self.seed_index.wrapping_add(1);
        value
    }
}
