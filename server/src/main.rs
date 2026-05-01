// Lanes and Lies — headless game server
//
// Authority model: ADR-002 — server is the sole authoritative writer for all
// game state. Clients are read-only views that emit C2S intent messages and
// receive S2C projection messages. No game logic, RNG, or hidden state exists
// in the client crate.
//
// Workspace layout: ADR-003 — three-crate workspace (shared/, server/, client/).
// server/ depends on shared/; client/ depends on shared/; neither depends on
// the other. Server-only types (HiddenObjectives, ServerRng) live here and are
// unreachable from client/ by construction.
//
// Channel config: ADR-008 — two Lightyear channels (ReliableChannel,
// UnreliableChannel) defined in shared/src/protocol.rs.
//
// Layer rule (ADR-003 §Consequences): feature/ may import core/, core/ may
// import foundation/. Reverse direction is forbidden. Violations require
// escalation to lead-programmer.

mod core;
mod feature;
mod foundation;
mod lobby;
mod network;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

// ---------------------------------------------------------------------------
// Server-only Resources
// ADR-002: these types MUST NOT appear in shared/ or client/.
// They live here so the linker enforces the boundary at compile time.
// ---------------------------------------------------------------------------

/// Per-session hidden objective assignments.
/// ADR-001: objective identity is sent as a unicast S2CObjectiveIdentities
/// message at DRAFT_INITIAL, never as a replicated ECS component.
/// ADR-002: opponent must never receive another player's ObjectiveIdentity.
#[derive(Resource, Default)]
pub struct HiddenObjectives {
    // TODO(Epic 5 — objective-system story): populate per-player is_fake map.
    // Key: PlayerId, Value: Vec<(ObjectiveId, bool /* is_fake */)>
    _placeholder: (),
}

// ServerRng is defined in foundation::rng — see server/src/foundation/rng.rs
// ADR-005: ChaCha20 seeded from OS entropy, full audit log, per-session resource.

// ---------------------------------------------------------------------------
// C2S message handler entry point
// ADR-002 Implementation Guideline 3: ALL C2S handlers route through this
// single function. It validates phase, sender identity, and domain rules
// before applying state. On any validation failure it returns silently —
// zero S2C response is sent to the client (network-protocol.md Rule 4).
// ---------------------------------------------------------------------------

/// Canonical C2S dispatch entry point.
///
/// ADR-002: every C2S message flows through here. Never panic on invalid input.
///
/// # Implementation note
/// This stub will be replaced in the Lightyear integration story (Epic 4,
/// S1-05 spike) with a proper Bevy system accepting `MessageReceiver<T>` and
/// mutable `RoundState` system params. The signature is reserved here to
/// document the authority-dispatch contract before any networking code is wired.
///
/// # References
/// - ADR-002 §Key Interfaces — `handle_c2s_message` dispatch sketch
/// - ADR-008 — channel config; `MessageReceiver<T>` (verify against Lightyear
///   0.26 docs; checklist items 4–7 must be resolved before implementation)
/// - network-protocol.md Rule 4 — silent discard on validation failure
// TODO(S1-05 Lightyear spike): verify exact Lightyear 0.26 channel + message registration
// API against docs.rs/lightyear/0.26 before implementing. Checklist items 1-6 must be
// signed off first. Stub lives here (not shared/) per ADR-003 fallback.
// Scaffold API consumed by downstream stories.
#[allow(dead_code)]
fn handle_c2s_message() {
    // TODO(Epic 4 — S1-05 Lightyear spike):
    //   1. Resolve Lightyear ClientId → PlayerId via SessionRegistry.
    //      Unknown sender: tracing::warn! and return.
    //   2. Phase-gate: check RoundState.phase(); discard with tracing::debug!.
    //   3. Domain validation: gold, hand size, bid amount, etc.
    //      Discard with tracing::debug! on any failure.
    //   4. Apply to authoritative ECS state atomically (Rule 5).
    //   5. Emit required S2C message(s) via MessageSender.
    //
    // ADR-008 verification checklist items 4–7 MUST be resolved before
    // implementing MessageReceiver / MessageSender system params.
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

fn main() {
    // ADR-002: headless server — no windowing, no rendering, no UI.
    // Bevy feature flags in server/Cargo.toml: "multi_threaded", "bevy_log",
    // "bevy_asset", "bevy_state" (see Cargo.toml).
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);

    // Asset pipeline — must be added before ConfigPlugin.
    // ADR-004: AssetPlugin default configuration; asset root is assets/.
    app.add_plugins(AssetPlugin::default());

    // Foundation — GameConfig + CardCatalog loading pipeline (ADR-004).
    // Registers loaders, AppState machine, and loading systems.
    // State machine: Loading → ConfigValidation → Lobby.
    app.add_plugins(foundation::config::ConfigPlugin);

    // Core — Round State Machine scaffold (ADR-009/ADR-010).
    app.add_plugins(core::session::GameSessionPlugin);
    app.add_plugins(core::rsm::RsmPlugin);
    app.add_plugins(core::economy::EconomyPlugin);
    app.add_plugins(feature::board::BoardPlugin);
    app.add_plugins(feature::acquisition::CardAcquisitionPlugin);
    app.add_plugins(feature::combat::CombatPlugin);

    // Networking - Lightyear 0.26 WebSocket server and shared protocol manifest.
    app.add_plugins(network::ServerNetworkPlugin);

    // Insert server-only resources.
    // ADR-002: these are unreachable from client/ by crate isolation.
    app.insert_resource(HiddenObjectives::default());
    app.insert_resource(foundation::rng::ServerRng::new());

    // TODO(Epic 4 — S1-05 Lightyear spike):
    // Add lightyear ServerPlugin with WebSocket transport config.
    // Verify exact plugin name and config struct against Lightyear 0.26 docs
    // before writing — ADR-008 checklist item 1 (channel definition syntax)
    // must be signed off first.
    // app.add_plugins(lightyear::server::ServerPlugin { ... });

    // TODO(Epic 2 — foundation/core/feature plugin stories):
    // Add layer plugins following the DAG from ADR-003:
    //   app.add_plugins(core::CorePlugin);              // RSM, Session, Economy, Pool
    //   app.add_plugins(feature::FeaturePlugin);        // Board, Objective (M1)
    // Each plugin is defined in its layer's mod.rs.

    info!("Lanes and Lies server starting — authority model: ADR-002");

    app.run();
}
