//! S13-TWO-CLIENT-RUNTIME-HARNESS-001 -- non-interactive scripted two-client
//! runtime harness driving the friend-game route end-to-end against a real
//! Lightyear WebSocket server.
//!
//! This binary lives at `tools/two-client-runtime/` per the canonical path
//! pinned in `production/epics/playable-client/story-017-two-client-runtime-harness.md`.
//!
//! ## What this harness is
//!
//! - A developer-invokable driver (NOT a CI gate by default).
//! - Boots a production-faithful Lightyear server in-process on an ephemeral
//!   port and spawns two production-faithful Bevy `App`s as clients.
//! - Connects both clients to the server over the production WebSocket
//!   transport (ADR-008).
//! - Scripts the friend-game route via real C2S intents; clients consume S2C
//!   broadcasts and never mutate authoritative state locally (ADR-002).
//! - Captures structured tracing logs from server, client A, client B, and
//!   the harness driver itself into a dated evidence bundle directory.
//! - Exits 0 when either the configured endpoint is reached (default:
//!   `MaxRounds(10)` OR S2CGameOver, whichever comes first); exits non-zero
//!   on connect timeout, max-tick overflow, or unexpected error.
//!
//! ## What this harness is NOT
//!
//! - A windowed Bevy run. The clients use Bevy 0.18's `App::update()` tick
//!   loop (no `DefaultPlugins`, no `WinitPlugin`); per liv-bevy-018.
//! - A closure of `S8-QA-001-W1`. AC12 explicitly forbids auto-closure. The
//!   manual two-client GAME_OVER gap is closed under a separate `/story-done`
//!   prompt with QA-lead sign-off.
//! - A reconnect-path tester. Mid-game disconnect scripting is a follow-on
//!   enhancement (PROMPT 803 §6 Sprint 14 candidate).
//!
//! ## No-claim restatement
//!
//! No optimistic client-side authority is introduced. The clients are
//! read-only views; the server is the sole authority for all state
//! transitions, RNG, and S2C broadcasts. ADR-002 / ADR-008 / ADR-011
//! binding is preserved.

mod logging;
mod route;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use ::server::core::rsm::RsmPlugin;
use ::server::core::session::{GameSessionPlugin, ServerRngFactory, ServerRngInitError};
use ::server::core::{economy::EconomyPlugin, pool::CardPoolPlugin};
use ::server::feature::{
    acquisition::CardAcquisitionPlugin, auction::AuctionPlugin, board::BoardPlugin,
    combat::CombatPlugin, keyword::KeywordPlugin, objective::ObjectivePlugin, prism::PrismPlugin,
};
use ::server::foundation::{config::ConfigPlugin, rng::ServerRng};
use ::server::network::ServerNetworkPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::logging::{init_role_subscriber, set_role, Role, RoleLogPaths};
use crate::route::{ClientRole, RouteProbe, RouteState};

const HARNESS_TICK_HZ: f64 = 60.0;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

#[derive(Debug, Clone)]
struct HarnessArgs {
    seed: u64,
    max_rounds: usize,
    connect_timeout_secs: u64,
    overall_timeout_secs: u64,
    evidence_dir: Option<PathBuf>,
    port: Option<u16>,
}

impl Default for HarnessArgs {
    fn default() -> Self {
        Self {
            seed: 1,
            max_rounds: 10,
            connect_timeout_secs: 5,
            overall_timeout_secs: 120,
            evidence_dir: None,
            port: None,
        }
    }
}

fn parse_args() -> Result<HarnessArgs, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = HarnessArgs::default();
    let mut i = 0;
    while i < raw.len() {
        let next = || -> Result<&String, String> {
            raw.get(i + 1)
                .ok_or_else(|| format!("flag {} requires a value", raw[i]))
        };
        match raw[i].as_str() {
            "--seed" => {
                args.seed = next()?
                    .parse()
                    .map_err(|err| format!("--seed expects u64: {err}"))?;
                i += 2;
            }
            "--max-rounds" => {
                args.max_rounds = next()?
                    .parse()
                    .map_err(|err| format!("--max-rounds expects usize: {err}"))?;
                i += 2;
            }
            "--connect-timeout-secs" => {
                args.connect_timeout_secs = next()?
                    .parse()
                    .map_err(|err| format!("--connect-timeout-secs expects u64: {err}"))?;
                i += 2;
            }
            "--overall-timeout-secs" => {
                args.overall_timeout_secs = next()?
                    .parse()
                    .map_err(|err| format!("--overall-timeout-secs expects u64: {err}"))?;
                i += 2;
            }
            "--evidence-dir" => {
                args.evidence_dir = Some(PathBuf::from(next()?));
                i += 2;
            }
            "--port" => {
                args.port = Some(
                    next()?
                        .parse()
                        .map_err(|err| format!("--port expects u16: {err}"))?,
                );
                i += 2;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown flag: {other}")),
        }
    }
    Ok(args)
}

fn print_help() {
    let bin = env!("CARGO_BIN_NAME");
    println!(
        "{bin} -- non-interactive scripted two-client runtime harness\n\
         \n\
         USAGE: {bin} [FLAGS]\n\
         \n\
         FLAGS:\n  \
           --seed N                  Deterministic ChaCha20 seed (default 1)\n  \
           --max-rounds N            Round-count cutoff (default 10)\n  \
           --connect-timeout-secs N  Max seconds to wait for both clients to handshake (default 5)\n  \
           --overall-timeout-secs N  Hard wall-clock cap for the whole run (default 120)\n  \
           --evidence-dir PATH       Override evidence bundle root (default \
        production/qa/evidence/captures/sprint-13-two-client-runtime/<UTC-date>/)\n  \
           --port N                  Server bind port (default: ephemeral)\n  \
           -h, --help                Print this help and exit\n"
    );
}

/// Final state snapshot serialised to `final_state.json` per AC5. Field set
/// is intentionally stable so AC7 (determinism) can be checked by `diff`-ing
/// two runs with the same `--seed` (timestamps excluded).
#[derive(Serialize)]
struct FinalState {
    harness_version: &'static str,
    seed: u64,
    max_rounds: usize,
    connect_timeout_secs: u64,
    server_port: u16,
    server_url: String,
    server_evidence_payload: ServerEvidence,
    routes_observed: RouteFacts,
    extra: HashMap<String, JsonValue>,
}

#[derive(Serialize)]
struct ServerEvidence {
    websocket_bound: bool,
    websocket_bind_addr: String,
}

#[derive(Serialize)]
struct RouteFacts {
    host_sent_hello: bool,
    joiner_sent_hello: bool,
    host_received_handshake: bool,
    joiner_received_handshake: bool,
    host_player_id: u64,
    joiner_player_id: u64,
    host_received_room_created: bool,
    joiner_received_join_ack: bool,
    host_received_classes_revealed: bool,
    joiner_received_classes_revealed: bool,
    host_received_card_acquired: bool,
    joiner_received_card_acquired: bool,
    placement_phase_count: usize,
    draft_shop_phase_count: usize,
    resolution_phase_count: usize,
    auction_phase_count: usize,
    host_received_game_over: bool,
    joiner_received_game_over: bool,
    game_over_round: usize,
    game_over_reason_draw: bool,
    endpoint_reached: &'static str,
    rounds_observed: usize,
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}\n");
            print_help();
            return ExitCode::from(2);
        }
    };

    let evidence_dir = match prepare_evidence_dir(&args) {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("failed to prepare evidence directory: {err}");
            return ExitCode::from(2);
        }
    };

    let log_paths = RoleLogPaths::under(&evidence_dir);
    let _writer = match init_role_subscriber(&log_paths) {
        Ok(w) => w,
        Err(err) => {
            eprintln!("failed to install role-aware tracing subscriber: {err}");
            return ExitCode::from(2);
        }
    };

    set_role(Role::Harness);
    tracing::info!(
        target: "harness",
        seed = args.seed,
        max_rounds = args.max_rounds,
        connect_timeout_secs = args.connect_timeout_secs,
        overall_timeout_secs = args.overall_timeout_secs,
        evidence_dir = %evidence_dir.display(),
        "harness boot"
    );

    let port = match args.port {
        Some(p) => p,
        None => match reserve_ephemeral_port() {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(target: "harness", err = %err, "failed to reserve ephemeral port");
                return ExitCode::from(2);
            }
        },
    };
    let url = format!("ws://127.0.0.1:{port}");
    tracing::info!(target: "harness", port, url = %url, "ephemeral websocket port reserved");

    let state = RouteState::default();
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    // `bind_addr` is included in the AC2 evidence payload below.

    set_role(Role::Server);
    let mut server_app = build_server_app(port, args.seed);
    tracing::info!(target: "harness::server", "server app built");

    // Prime the server bind by ticking a few frames before client connects.
    for _ in 0..30 {
        set_role(Role::Server);
        server_app.update();
        thread::sleep(FRAME_SLEEP);
    }
    tracing::info!(target: "harness::server", "server prime ticks complete");

    set_role(Role::ClientA);
    tracing::info!(target: "harness::client_a", url = %url, "client A app build start");
    let mut client_a_app = build_client_app(url.clone(), ClientRole::Host, state.clone());
    tracing::info!(target: "harness::client_a", "client A app built");
    set_role(Role::ClientB);
    tracing::info!(target: "harness::client_b", url = %url, "client B app build start");
    let mut client_b_app = build_client_app(url.clone(), ClientRole::Joiner, state.clone());
    tracing::info!(target: "harness::client_b", "client B app built");

    set_role(Role::Harness);
    tracing::info!(target: "harness", "both client apps built");
    let started = Instant::now();
    let connect_deadline = started + Duration::from_secs(args.connect_timeout_secs);
    let overall_deadline = started + Duration::from_secs(args.overall_timeout_secs);
    let max_ticks = ((args.overall_timeout_secs as f64) * HARNESS_TICK_HZ).ceil() as usize + 1_000;

    let mut endpoint_reached = "none";
    let mut connected = false;
    for tick in 0..max_ticks {
        set_role(Role::Server);
        server_app.update();
        set_role(Role::ClientA);
        client_a_app.update();
        set_role(Role::ClientB);
        client_b_app.update();

        if !connected && state.both_connected() {
            connected = true;
            set_role(Role::Harness);
            tracing::info!(
                target: "harness",
                tick,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "both clients handshake complete"
            );
        }

        if !connected && Instant::now() > connect_deadline {
            set_role(Role::Harness);
            tracing::error!(
                target: "harness",
                tick,
                connect_timeout_secs = args.connect_timeout_secs,
                "AC2 connect-timeout exceeded before both clients handshake"
            );
            endpoint_reached = "connect_timeout";
            break;
        }

        if state.either_game_over() {
            endpoint_reached = "game_over";
            set_role(Role::Harness);
            tracing::info!(
                target: "harness",
                tick,
                round = state.game_over_round.load(Ordering::SeqCst),
                "S2CGameOver observed -- canonical AC3 endpoint reached"
            );
            // Drain a few more ticks so the second client also records the
            // S2CGameOver broadcast (both flags should land before exit).
            for _ in 0..120 {
                set_role(Role::Server);
                server_app.update();
                set_role(Role::ClientA);
                client_a_app.update();
                set_role(Role::ClientB);
                client_b_app.update();
                if state.host_received_game_over.load(Ordering::SeqCst)
                    && state.joiner_received_game_over.load(Ordering::SeqCst)
                {
                    break;
                }
                thread::sleep(FRAME_SLEEP);
            }
            break;
        }

        if state.rounds_observed() >= args.max_rounds {
            endpoint_reached = "max_rounds";
            set_role(Role::Harness);
            tracing::info!(
                target: "harness",
                tick,
                rounds_observed = state.rounds_observed(),
                max_rounds = args.max_rounds,
                "AC3 max-rounds cutoff reached"
            );
            break;
        }

        if Instant::now() > overall_deadline {
            endpoint_reached = "overall_timeout";
            set_role(Role::Harness);
            tracing::error!(
                target: "harness",
                tick,
                overall_timeout_secs = args.overall_timeout_secs,
                "overall wall-clock deadline exceeded"
            );
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    if endpoint_reached == "none" {
        endpoint_reached = "tick_overflow";
        set_role(Role::Harness);
        tracing::error!(
            target: "harness",
            max_ticks,
            "max-tick loop exhausted without endpoint"
        );
    }

    set_role(Role::Harness);

    let final_state = FinalState {
        harness_version: env!("CARGO_PKG_VERSION"),
        seed: args.seed,
        max_rounds: args.max_rounds,
        connect_timeout_secs: args.connect_timeout_secs,
        server_port: port,
        server_url: url.clone(),
        server_evidence_payload: ServerEvidence {
            websocket_bound: connected,
            websocket_bind_addr: bind_addr.to_string(),
        },
        routes_observed: RouteFacts {
            host_sent_hello: state.host_sent_hello.load(Ordering::SeqCst),
            joiner_sent_hello: state.joiner_sent_hello.load(Ordering::SeqCst),
            host_received_handshake: state.host_received_handshake.load(Ordering::SeqCst),
            joiner_received_handshake: state.joiner_received_handshake.load(Ordering::SeqCst),
            host_player_id: state.host_player_id.load(Ordering::SeqCst),
            joiner_player_id: state.joiner_player_id.load(Ordering::SeqCst),
            host_received_room_created: state.host_received_room_created.load(Ordering::SeqCst),
            joiner_received_join_ack: state.joiner_received_join_ack.load(Ordering::SeqCst),
            host_received_classes_revealed: state
                .host_received_classes_revealed
                .load(Ordering::SeqCst),
            joiner_received_classes_revealed: state
                .joiner_received_classes_revealed
                .load(Ordering::SeqCst),
            host_received_card_acquired: state.host_received_card_acquired.load(Ordering::SeqCst),
            joiner_received_card_acquired: state
                .joiner_received_card_acquired
                .load(Ordering::SeqCst),
            placement_phase_count: state.placement_phase_count.load(Ordering::SeqCst),
            draft_shop_phase_count: state.draft_shop_phase_count.load(Ordering::SeqCst),
            resolution_phase_count: state.resolution_phase_count.load(Ordering::SeqCst),
            auction_phase_count: state.auction_phase_count.load(Ordering::SeqCst),
            host_received_game_over: state.host_received_game_over.load(Ordering::SeqCst),
            joiner_received_game_over: state.joiner_received_game_over.load(Ordering::SeqCst),
            game_over_round: state.game_over_round.load(Ordering::SeqCst),
            game_over_reason_draw: state.game_over_reason_draw.load(Ordering::SeqCst),
            endpoint_reached,
            rounds_observed: state.rounds_observed(),
        },
        extra: HashMap::new(),
    };

    if let Err(err) = write_final_state(&evidence_dir, &final_state) {
        tracing::error!(
            target: "harness",
            err = %err,
            "failed to write final_state.json"
        );
        return ExitCode::from(2);
    }

    let success = matches!(endpoint_reached, "game_over" | "max_rounds");
    tracing::info!(
        target: "harness",
        endpoint_reached,
        success,
        elapsed_secs = started.elapsed().as_secs_f64(),
        "harness exit"
    );

    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn prepare_evidence_dir(args: &HarnessArgs) -> std::io::Result<PathBuf> {
    let base = match &args.evidence_dir {
        Some(p) => p.clone(),
        None => {
            let workspace = workspace_root();
            workspace
                .join("production")
                .join("qa")
                .join("evidence")
                .join("captures")
                .join("sprint-13-two-client-runtime")
                .join(today_utc_dirname())
        }
    };
    fs::create_dir_all(&base)?;
    Ok(base)
}

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or(manifest)
}

fn today_utc_dirname() -> String {
    // Match the precedent set by manual-friend-game-evidence-YYYY-MM-DD/ per
    // story AC5. The CARGO_PKG_VERSION-derived UTC date is captured at run
    // start so two runs on the same day land in the same directory (and
    // overwrite per-file).
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Days since UNIX epoch + month/day approximation. We do NOT pull chrono
    // as a dependency — story AC11 / AC8 favour minimal new deps. Instead we
    // emit a YYYYMMDD-HHMMSS UTC stamp using a small civil-date conversion.
    let (year, month, day, _h, _m, _s) = unix_to_utc_ymdhms(now);
    format!("{year:04}-{month:02}-{day:02}")
}

fn unix_to_utc_ymdhms(unix: u64) -> (u32, u32, u32, u32, u32, u32) {
    // Civil-from-days from Howard Hinnant's date algorithms (public domain).
    let secs_per_day: u64 = 86_400;
    let z = (unix / secs_per_day) as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y: i64 = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y_civil = if m <= 2 { y + 1 } else { y };
    let day_of = (unix % secs_per_day) as u32;
    let hour = day_of / 3_600;
    let minute = (day_of % 3_600) / 60;
    let second = day_of % 60;
    (y_civil as u32, m as u32, d as u32, hour, minute, second)
}

fn write_final_state(dir: &Path, state: &FinalState) -> std::io::Result<()> {
    let path = dir.join("final_state.json");
    let json = serde_json::to_string_pretty(state)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    file.write_all(json.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn reserve_ephemeral_port() -> std::io::Result<u16> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let port = listener.local_addr()?.port();
    // Drop closes the socket; the port may briefly enter TIME_WAIT but
    // re-binding on the same loopback typically succeeds within a few ticks.
    drop(listener);
    Ok(port)
}

fn build_server_app(port: u16, seed: u64) -> App {
    // Mirrors `server::main::main()` plugin composition with two harness-
    // controlled deviations:
    //   * SERVER_PORT env var is set so production ServerNetworkPlugin binds
    //     to the harness-reserved port (AC2/AC4).
    //   * RNG factory uses the deterministic from_seed constructor (AC7).
    // No production code is modified per AC8; the plugin set is composed
    // here entirely from server crate exports already on origin/main.
    set_deterministic_seed(seed);
    std::env::set_var("SERVER_PORT", port.to_string());

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(AssetPlugin {
        file_path: format!("{}/../../assets", env!("CARGO_MANIFEST_DIR")),
        ..default()
    });
    app.add_plugins(ConfigPlugin);
    app.add_plugins(GameSessionPlugin);
    app.add_plugins(RsmPlugin);
    app.add_plugins(EconomyPlugin);
    app.add_plugins(CardPoolPlugin);
    app.add_plugins(BoardPlugin);
    app.add_plugins(AuctionPlugin);
    app.add_plugins(CardAcquisitionPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(KeywordPlugin);
    app.add_plugins(PrismPlugin);
    // ServerNetworkPlugin includes:
    //   * `ServerPlugins` (production tick rate)
    //   * `register_lightyear_protocol`
    //   * `EconomyNetworkPlugin`
    //   * `PlayerConnectionMap` resource init
    //   * `open_websocket_server` startup system (reads SERVER_PORT env var)
    //   * `drain_signal_ready_messages` / `drain_submit_placement_messages`
    //   * `handle_class_choice`
    //   * `dispatch_phase_changed` after `advance_phase`
    //   * `insert_replication_sender_on_link` observer (critical for unicast S2C
    //     messages like S2CObjectiveIdentities -- without it the route stalls
    //     before draft completes).
    app.add_plugins(ServerNetworkPlugin);
    app.add_plugins(ObjectivePlugin);

    // Deterministic seed wiring (AC7). ServerRngFactory takes a `fn` pointer
    // (not a closure), so the seed is parked in a process-global cell that
    // the factory reads at session-start time. The harness only ever runs
    // one server App per process, so the cell is safe.
    app.insert_resource(ServerRngFactory::new(deterministic_rng_factory));

    app.finish();
    app
}

fn set_deterministic_seed(seed: u64) {
    let cell = DETERMINISTIC_SEED.get_or_init(|| std::sync::Mutex::new(0));
    let mut guard = cell
        .lock()
        .expect("deterministic-seed mutex must not be poisoned");
    *guard = seed;
}

fn deterministic_rng_factory() -> Result<ServerRng, ServerRngInitError> {
    let cell = DETERMINISTIC_SEED.get_or_init(|| std::sync::Mutex::new(0));
    let guard = cell
        .lock()
        .expect("deterministic-seed mutex must not be poisoned");
    Ok(ServerRng::from_seed(*guard))
}

static DETERMINISTIC_SEED: std::sync::OnceLock<std::sync::Mutex<u64>> = std::sync::OnceLock::new();

fn build_client_app(url: String, role: ClientRole, state: RouteState) -> App {
    // Mirrors `client::main::main()` minus DefaultPlugins / windowing / audio /
    // presentation / UI -- the harness only needs the networking surface to
    // exchange C2S/S2C messages over the production transport. AC9 binding:
    // no presentation systems mutate authoritative state; we omit them
    // entirely so there is no risk of accidentally introducing optimistic
    // mirrors via UI-side flows.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / HARNESS_TICK_HZ),
    });
    ::client::network::register_lightyear_protocol(&mut app);

    app.insert_resource(RouteProbe { role, state });
    app.add_systems(Startup, move |mut commands: Commands| {
        let role_label = match role {
            ClientRole::Host => "Host",
            ClientRole::Joiner => "Joiner",
        };
        let client = commands
            .spawn((
                Name::new(format!("Two-Client Runtime Harness {role_label} Client")),
                Client::default(),
                RawClient,
                LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                WebSocketClientIo::from_url(ClientConfig::default(), url.clone()),
            ))
            .id();
        commands.trigger(Connect { entity: client });
    });

    app.add_systems(
        Update,
        (
            route::send_hello_until_handshake,
            route::record_handshake,
            route::send_lobby_actions,
            route::record_s2c_handshake_chain,
            route::record_s2c_draft_and_phase,
            route::send_draft_initial_purchase,
            route::send_draft_initial_ready,
            route::send_loop_actions,
            route::record_game_over,
        )
            .chain(),
    );

    app.finish();
    app
}
