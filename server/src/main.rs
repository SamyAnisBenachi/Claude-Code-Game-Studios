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

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

// ServerRng is defined in foundation::rng — see server/src/foundation/rng.rs
// ADR-005: ChaCha20 seeded from OS entropy, full audit log, per-session resource.

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

fn main() {
    // ADR-002: headless server — no windowing, no rendering, no UI.
    // Bevy feature flags in server/Cargo.toml: "multi_threaded", "bevy_log",
    // "bevy_asset", "bevy_state" (see Cargo.toml).
    //
    // MinimalPlugins does not include LogPlugin, so we initialise tracing here
    // directly. This must come before App::new() so that all plugin startup
    // messages are captured.
    //
    // S13-OBS-WALLCLOCK-TIMESTAMPS-001 (PROMPT 837): wall-clock UTC ISO-8601
    // (RFC 3339) timer so multi-process logs from server + client + tests
    // align at sub-second precision. Default fmt timer emits relative seconds
    // since process start, which is useless for cross-process correlation.
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);

    // Asset pipeline — must be added before ConfigPlugin.
    // ADR-004: asset root is repo-root assets/. Resolve from CARGO_MANIFEST_DIR
    // so AppState progresses past Loading regardless of process CWD.
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(AssetPlugin {
        file_path: format!("{}/../assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    });
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(AssetPlugin::default());

    // Foundation — GameConfig + CardCatalog loading pipeline (ADR-004).
    // Registers loaders, AppState machine, and loading systems.
    // State machine: Loading → ConfigValidation → Lobby.
    app.add_plugins(foundation::config::ConfigPlugin);

    // Core — Round State Machine scaffold (ADR-009/ADR-010).
    app.add_plugins(core::session::GameSessionPlugin);
    app.add_plugins(core::rsm::RsmPlugin);
    app.add_plugins(core::economy::EconomyPlugin);
    // CardPoolPlugin owns PlayerPools / ShopSlots / InitialDraftOffering /
    // ManualRefreshCount lifecycle. Without it, `initialize_player_pools_on_draft_started`
    // never runs and `card_acquisition_tick_system` early-returns at every
    // DRAFT_INITIAL ShopRefreshTriggered with "PlayerPools resource not available",
    // so no S2CDraftOffering is ever sent. (Root cause of every DRAFT_INITIAL
    // failure since SAU-001; PROMPT 545 — comprehensive E2E analysis.)
    app.add_plugins(core::pool::CardPoolPlugin);
    app.add_plugins(feature::board::BoardPlugin);
    app.add_plugins(feature::auction::AuctionPlugin);
    app.add_plugins(feature::acquisition::CardAcquisitionPlugin);
    app.add_plugins(feature::combat::CombatPlugin);
    // KeywordPlugin owns the keyword observer registrations (`on_unit_appeared`,
    // `on_unit_died`, `on_final_blow_dealt`, `on_start_of_turn`, `on_end_of_turn`)
    // and the `start_of_turn_dispatch_system`. Without it, none of the keyword
    // effects (Provocation, Shield, Charge, etc.) fire during combat resolution.
    app.add_plugins(feature::keyword::KeywordPlugin);
    app.add_plugins(feature::prism::PrismPlugin);
    // PROMPT 1514 (BOT-ROOM-JOIN-LOOP): deterministic bot lobby auto-confirm so
    // rooms that already contain a bot occupant can lift out of LobbyWaiting.
    app.add_plugins(feature::bot::BotLobbyPlugin);
    // PROMPT 1531 (BOT-PARTICIPANT-ACTION-LOOP-WAVE1): deterministic bot action
    // loop that signals draft-ready, passes auction, and submits an empty
    // placement so flow advances without a human counterpart.
    app.add_plugins(feature::bot::BotActionLoopPlugin);
    // PROMPT 1597 (BOT-FLOW-SERVER-QA-SNAPSHOT-AND-DECISION-LOG):
    // server-authoritative QA evidence for bot-driven flows. Writes JSON
    // snapshots (phase transitions / 10s periodic / best-effort AppExit)
    // under `dev-runs/bot-qa-snapshots/` and streams every BotDecisionLog
    // append to `dev-runs/bot-decision-log.jsonl`. Disabled in release by
    // default; enable explicitly via `CCGS_BOT_QA_SNAPSHOT=1`.
    app.add_plugins(feature::bot::BotQaSnapshotPlugin);

    // Networking - Lightyear 0.26 WebSocket server and shared protocol manifest.
    app.add_plugins(network::ServerNetworkPlugin);

    // Objective System - replicated objective HP plus server-only identities.
    app.add_plugins(feature::objective::ObjectivePlugin);

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
