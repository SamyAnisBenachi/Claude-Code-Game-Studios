//! `bot-soak-trigger` binary — PROMPT 1672 BOT-SOAK-ROOM-TRIGGER-PATH-DISPOSITION.
//!
//! Connects a single headless Bevy client to an external CCGS server and
//! sends `C2SCreateBotRoom` so the server's `BotLobbyPlugin` and
//! `BotActionLoopPlugin` can run a full bot-vs-bot session without any GUI.
//!
//! Unlike `two-client-runtime` (which starts server + 2 clients in-process),
//! this binary is a pure client that connects to an already-running server
//! (e.g. one started by `Start-BotVsBotSoak.ps1`).
//!
//! ## What this binary does
//!
//! - Connects over the production WebSocket transport to `--server-url`.
//! - Sends the full lobby handshake: `C2SHello` → `S2CHandshake`.
//! - Sends `C2SCreateBotRoom { mode: OneVOne, bot_kind: Default }`.
//! - Confirms class for the human-proxy slot (`C2SSelectClass + C2SConfirmClass`).
//!   The server auto-confirms the bot's class via `BotLobbyPlugin`.
//! - Responds minimally to each round phase so the game does not stall:
//!   empty placement, draft-shop ready, auction bid.
//! - Exits 0 when `S2CGameOver` is received or `--max-rounds` threshold reached.
//! - Exits 1 on connect timeout or overall wall-clock timeout.
//!
//! ## What this binary does NOT do
//!
//! - Does not start a server. Connect to an external one via `--server-url`.
//! - Does not open a window. Uses `MinimalPlugins` (no rendering, no audio).
//! - Does not mutate authoritative game state. All C2S messages are intents;
//!   the server remains the sole authority (ADR-002).

mod bot_route;
mod logging;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use serde::Serialize;

use crate::bot_route::{BotSoakRoute, TriggerCardEntry};
use crate::logging::{init_role_subscriber, set_role, Role, RoleLogPaths};

const TICK_HZ: f64 = 60.0;
const FRAME_SLEEP: Duration = Duration::from_millis(10);

#[derive(Debug)]
struct Args {
    server_url: String,
    max_rounds: usize,
    connect_timeout_secs: u64,
    overall_timeout_secs: u64,
    evidence_dir: Option<PathBuf>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:5000".to_owned(),
            max_rounds: 0,
            connect_timeout_secs: 15,
            overall_timeout_secs: 300,
            evidence_dir: None,
        }
    }
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args::default();
    let mut i = 0;
    while i < raw.len() {
        let next = || raw.get(i + 1).ok_or_else(|| format!("{} requires a value", raw[i]));
        match raw[i].as_str() {
            "--server-url" => {
                args.server_url = next()?.clone();
                i += 2;
            }
            "--max-rounds" => {
                args.max_rounds = next()?
                    .parse()
                    .map_err(|e| format!("--max-rounds expects usize: {e}"))?;
                i += 2;
            }
            "--connect-timeout-secs" => {
                args.connect_timeout_secs = next()?
                    .parse()
                    .map_err(|e| format!("--connect-timeout-secs expects u64: {e}"))?;
                i += 2;
            }
            "--overall-timeout-secs" => {
                args.overall_timeout_secs = next()?
                    .parse()
                    .map_err(|e| format!("--overall-timeout-secs expects u64: {e}"))?;
                i += 2;
            }
            "--evidence-dir" => {
                args.evidence_dir = Some(PathBuf::from(next()?));
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
        "{bin} — headless bot-soak-trigger client (PROMPT 1672)\n\
         \n\
         Connects to a running CCGS server, sends C2SCreateBotRoom, drives the\n\
         human-proxy slot through all round phases, exits when S2CGameOver fires.\n\
         \n\
         FLAGS:\n  \
           --server-url URL             WebSocket URL of a running server (default ws://127.0.0.1:5000)\n  \
           --max-rounds N               Client-side round cutoff (default 0 = no cutoff)\n  \
           --connect-timeout-secs N     Max seconds to wait for S2CHandshake (default 15)\n  \
           --overall-timeout-secs N     Hard wall-clock cap (default 300)\n  \
           --evidence-dir PATH          Directory for log files and final_state.json\n  \
           -h, --help                   Print this help and exit\n"
    );
}

#[derive(Serialize)]
struct FinalState {
    binary: &'static str,
    server_url: String,
    max_rounds: usize,
    overall_timeout_secs: u64,
    connect_timeout_secs: u64,
    endpoint_reached: &'static str,
    rounds_observed: usize,
    received_handshake: bool,
    received_room_created: bool,
    sent_class_confirm: bool,
    received_game_over: bool,
    elapsed_secs: f64,
    exit_code: u8,
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

    let evidence_dir = prepare_evidence_dir(&args);

    let log_paths = RoleLogPaths::under(&evidence_dir);
    let _writer = match init_role_subscriber(&log_paths) {
        Ok(w) => w,
        Err(err) => {
            eprintln!("failed to install tracing subscriber: {err}");
            return ExitCode::from(2);
        }
    };

    set_role(Role::ClientA);
    tracing::info!(
        server_url = %args.server_url,
        max_rounds = args.max_rounds,
        connect_timeout_secs = args.connect_timeout_secs,
        overall_timeout_secs = args.overall_timeout_secs,
        evidence_dir = %evidence_dir.display(),
        "bot-soak-trigger boot"
    );

    let card_info = load_card_info();
    let loaded = card_info.len();
    tracing::info!(loaded, "bot-soak-trigger: card_info loaded");
    let route = BotSoakRoute {
        card_info: Arc::new(card_info),
        ..BotSoakRoute::default()
    };
    let mut client_app = build_client_app(args.server_url.clone(), route.clone());

    let started = Instant::now();
    let connect_deadline = started + Duration::from_secs(args.connect_timeout_secs);
    let overall_deadline = started + Duration::from_secs(args.overall_timeout_secs);
    let max_ticks =
        ((args.overall_timeout_secs as f64) * TICK_HZ).ceil() as usize + 1_000;

    let mut endpoint_reached: &'static str = "none";
    let mut connected = false;

    for tick in 0..max_ticks {
        client_app.update();

        if !connected && route.received_handshake.load(Ordering::SeqCst) {
            connected = true;
            tracing::info!(
                tick,
                elapsed_ms = started.elapsed().as_millis() as u64,
                "bot-soak-trigger: handshake complete"
            );
        }

        if !connected && Instant::now() > connect_deadline {
            tracing::error!(tick, "bot-soak-trigger: connect timeout before handshake");
            endpoint_reached = "connect_timeout";
            break;
        }

        if route.is_done() {
            endpoint_reached = "game_over";
            tracing::info!(
                tick,
                rounds = route.rounds_observed(),
                "bot-soak-trigger: S2CGameOver received — canonical endpoint"
            );
            break;
        }

        if args.max_rounds > 0 && route.rounds_observed() >= args.max_rounds {
            endpoint_reached = "max_rounds";
            tracing::info!(
                tick,
                rounds = route.rounds_observed(),
                max_rounds = args.max_rounds,
                "bot-soak-trigger: client-side max-rounds cutoff"
            );
            break;
        }

        if Instant::now() > overall_deadline {
            endpoint_reached = "overall_timeout";
            tracing::error!(tick, "bot-soak-trigger: overall wall-clock deadline exceeded");
            break;
        }

        thread::sleep(FRAME_SLEEP);
    }

    if endpoint_reached == "none" {
        endpoint_reached = "tick_overflow";
        tracing::error!(max_ticks, "bot-soak-trigger: tick loop exhausted without endpoint");
    }

    let success = matches!(endpoint_reached, "game_over" | "max_rounds");
    let exit_code: u8 = if success { 0 } else { 1 };

    let elapsed = started.elapsed().as_secs_f64();
    tracing::info!(endpoint_reached, success, elapsed_secs = elapsed, "bot-soak-trigger exit");

    let final_state = FinalState {
        binary: env!("CARGO_BIN_NAME"),
        server_url: args.server_url.clone(),
        max_rounds: args.max_rounds,
        overall_timeout_secs: args.overall_timeout_secs,
        connect_timeout_secs: args.connect_timeout_secs,
        endpoint_reached,
        rounds_observed: route.rounds_observed(),
        received_handshake: route.received_handshake.load(Ordering::SeqCst),
        received_room_created: route.received_room_created.load(Ordering::SeqCst),
        sent_class_confirm: route.sent_confirm_class.load(Ordering::SeqCst),
        received_game_over: route.received_game_over.load(Ordering::SeqCst),
        elapsed_secs: elapsed,
        exit_code,
    };

    if let Err(err) = write_final_state(&evidence_dir, &final_state) {
        tracing::error!(err = %err, "failed to write final_state.json");
    }

    if exit_code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(exit_code)
    }
}

/// Load card cost + type info from `assets/data/cards.json` for placement decisions.
///
/// Tries several candidate paths relative to the CWD (the soak scripts run from
/// the project root).  Returns an empty map when the file cannot be found or
/// parsed — the trigger falls back to empty placements, preserving the PROMPT
/// 1678 contract.
fn load_card_info() -> HashMap<u32, TriggerCardEntry> {
    let candidates = [
        "assets/data/cards.json",
        "../assets/data/cards.json",
        "../../assets/data/cards.json",
    ];

    for path in &candidates {
        let Ok(data) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) else {
            tracing::warn!(path, "bot-soak-trigger: cards.json parse error");
            continue;
        };
        let Some(arr) = json.as_array() else {
            continue;
        };

        let mut map = HashMap::new();
        for card in arr {
            let (Some(id), Some(cost), Some(card_type)) = (
                card["id"].as_u64(),
                card["cost"].as_u64(),
                card["card_type"].as_str(),
            ) else {
                continue;
            };
            map.insert(
                id as u32,
                TriggerCardEntry {
                    cost: cost as u32,
                    is_minion: card_type == "Minion",
                },
            );
        }

        if !map.is_empty() {
            tracing::info!(path, count = map.len(), "bot-soak-trigger: cards.json loaded");
            return map;
        }
    }

    tracing::warn!("bot-soak-trigger: cards.json not found — placement will use empty fallback");
    HashMap::new()
}

fn prepare_evidence_dir(args: &Args) -> PathBuf {
    let dir = match &args.evidence_dir {
        Some(p) => p.clone(),
        None => {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let workspace = manifest
                .parent()
                .and_then(|p| p.parent())
                .map(Path::to_path_buf)
                .unwrap_or(manifest);
            workspace
                .join("production")
                .join("qa")
                .join("evidence")
                .join("captures")
                .join("bot-soak-trigger")
        }
    };
    fs::create_dir_all(&dir).expect("failed to create evidence directory");
    dir
}

fn write_final_state(dir: &Path, state: &FinalState) -> std::io::Result<()> {
    let path = dir.join("final_state.json");
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    file.write_all(json.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

fn build_client_app(url: String, route: BotSoakRoute) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / TICK_HZ),
    });
    ::client::network::register_lightyear_protocol(&mut app);

    app.insert_resource(route);

    let url_clone = url.clone();
    app.add_systems(Startup, move |mut commands: Commands| {
        let client = commands
            .spawn((
                Name::new("Bot Soak Trigger Client"),
                Client::default(),
                RawClient,
                LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
                WebSocketClientIo::from_url(ClientConfig::default(), url_clone.clone()),
            ))
            .id();
        commands.trigger(Connect { entity: client });
    });

    app.add_systems(
        Update,
        (
            bot_route::send_hello_until_handshake,
            bot_route::record_handshake,
            bot_route::send_create_bot_room,
            bot_route::record_room_created,
            bot_route::send_class_selection,
            // record_gold_update before record_draft_offering so the mana budget
            // is populated when we pick a card from the offering (PROMPT 1692).
            bot_route::record_gold_update,
            bot_route::record_draft_offering,
            bot_route::send_initial_purchase,
            bot_route::record_card_acquired,
            bot_route::send_initial_ready,
            bot_route::record_phase_and_auction,
            bot_route::send_loop_actions,
            bot_route::record_game_over,
        )
            .chain(),
    );

    app.finish();
    app
}
