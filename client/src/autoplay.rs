//! Autoplay / automation harness (dev-only, low-level input).
//!
//! See `docs/autoplay.md` for the full architecture, scope ladder, and the
//! hard invariants that any future autoplay change MUST respect:
//!
//! - **No gameplay mutation.** The RPC surface only exposes status reads,
//!   low-level input injection (keys, mouse buttons, cursor, scroll),
//!   `clear_input`, and `screenshot`. It MUST NOT expose semantic gameplay
//!   verbs (`kill`, `give_xp`, `select_card`, `advance_phase`, …) or any
//!   direct ECS state writes.
//! - **Dev-only.** Gated behind the `autoplay-remote` Cargo feature AND the
//!   `CCGS_AUTOPLAY=1` environment variable; release builds without the
//!   feature do not include this module at all.
//! - **Localhost only.** The RPC socket binds to `127.0.0.1`.
//!
//! Activation:
//!
//! ```sh
//! cargo run -p client --features autoplay-remote
//! # in a separate shell with CCGS_AUTOPLAY=1 set before launching the client
//! ```
//!
//! Default port: `15873` (override with `CCGS_AUTOPLAY_PORT`). Artifact
//! root: `production/qa/evidence/autoplay-runs/<timestamp>/` (override with
//! `CCGS_AUTOPLAY_ARTIFACT_DIR`).

use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::{PrimaryWindow, Window};

use crate::state::{ClientState, CurrentClientPhase};

pub const AUTOPLAY_ENABLE_ENV: &str = "CCGS_AUTOPLAY";
pub const AUTOPLAY_PORT_ENV: &str = "CCGS_AUTOPLAY_PORT";
pub const AUTOPLAY_ARTIFACT_DIR_ENV: &str = "CCGS_AUTOPLAY_ARTIFACT_DIR";
pub const DEFAULT_AUTOPLAY_PORT: u16 = 15873;

/// Schema/protocol version returned by `autoplay/capabilities`. Bump when
/// the RPC surface changes in a backwards-incompatible way.
pub const AUTOPLAY_RPC_VERSION: u32 = 2;

/// Plugin entry point. Build-safe to register; does nothing observable
/// unless `CCGS_AUTOPLAY=1` is set in the process environment at plugin
/// build time.
pub struct AutoplayPlugin;

impl Plugin for AutoplayPlugin {
    fn build(&self, app: &mut App) {
        let cfg = AutoplayConfig::from_env();
        if !cfg.enabled {
            tracing::info!(
                target: "client::autoplay",
                "AutoplayPlugin disabled (set {}=1 to enable)",
                AUTOPLAY_ENABLE_ENV
            );
            return;
        }

        if let Err(err) = fs::create_dir_all(cfg.artifact_dir.join("screenshots")) {
            tracing::error!(
                target: "client::autoplay",
                error = %err,
                dir = %cfg.artifact_dir.display(),
                "AutoplayPlugin failed to create artifact directory; disabling harness"
            );
            return;
        }

        let shared = Arc::new(AutoplayShared::new(cfg.artifact_dir.clone()));
        let bind_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, cfg.port);
        let listener = match TcpListener::bind(bind_addr) {
            Ok(l) => l,
            Err(err) => {
                tracing::error!(
                    target: "client::autoplay",
                    error = %err,
                    addr = %bind_addr,
                    "AutoplayPlugin failed to bind RPC port; disabling harness"
                );
                return;
            }
        };
        let bound_addr = listener.local_addr().ok();

        let server_shared = Arc::clone(&shared);
        thread::Builder::new()
            .name("ccgs-autoplay-rpc".into())
            .spawn(move || run_rpc_server(listener, server_shared))
            .expect("spawn autoplay RPC server thread");

        tracing::info!(
            target: "client::autoplay",
            addr = ?bound_addr,
            artifact_dir = %cfg.artifact_dir.display(),
            "AutoplayPlugin enabled (low-level input only; no gameplay mutation)"
        );

        app.insert_resource(AutoplayShared::handle(Arc::clone(&shared)))
            .insert_resource(cfg)
            .add_systems(Update, (drain_commands_system, publish_status_system));
    }
}

/// Runtime configuration for the autoplay harness.
#[derive(Resource, Debug, Clone)]
pub struct AutoplayConfig {
    pub enabled: bool,
    pub port: u16,
    pub artifact_dir: PathBuf,
}

impl AutoplayConfig {
    pub fn from_env() -> Self {
        let enabled = matches!(std::env::var(AUTOPLAY_ENABLE_ENV).ok().as_deref(), Some("1"));
        let port = std::env::var(AUTOPLAY_PORT_ENV)
            .ok()
            .and_then(|raw| raw.trim().parse::<u16>().ok())
            .unwrap_or(DEFAULT_AUTOPLAY_PORT);
        let artifact_dir = std::env::var(AUTOPLAY_ARTIFACT_DIR_ENV)
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(default_artifact_dir);
        Self {
            enabled,
            port,
            artifact_dir,
        }
    }
}

fn default_artifact_dir() -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // YYYYMMDD-HHMMSS-Z computed by std (no chrono dep on this crate).
    let stamp = format_utc_stamp(now);
    PathBuf::from("production/qa/evidence/autoplay-runs").join(stamp)
}

fn format_utc_stamp(unix_secs: u64) -> String {
    let (y, mo, d, h, mi, s) = unix_to_ymdhms(unix_secs);
    format!("{:04}{:02}{:02}-{:02}{:02}{:02}-Z", y, mo, d, h, mi, s)
}

/// Convert a Unix timestamp (UTC seconds) into a (year, month, day, hour,
/// minute, second) tuple. Implemented inline so this module has no `chrono`
/// or `time` dependency. Accurate for the proleptic Gregorian calendar.
fn unix_to_ymdhms(t: u64) -> (i32, u32, u32, u32, u32, u32) {
    let sec = (t % 60) as u32;
    let mi = ((t / 60) % 60) as u32;
    let h = ((t / 3600) % 24) as u32;
    let mut days = (t / 86_400) as i64;
    let mut y: i32 = 1970;
    loop {
        let yd = if is_leap(y) { 366 } else { 365 };
        if days >= yd as i64 {
            days -= yd as i64;
            y += 1;
        } else {
            break;
        }
    }
    let mdays = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo: u32 = 0;
    while days >= mdays[mo as usize] as i64 {
        days -= mdays[mo as usize] as i64;
        mo += 1;
    }
    (y, mo + 1, (days as u32) + 1, h, mi, sec)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// ---------- shared state between RPC thread and Bevy systems ----------

/// Lock-protected state shared between the RPC server thread and the Bevy
/// `Update` schedule. Kept intentionally small so contention is bounded.
#[derive(Debug)]
struct AutoplayShared {
    artifact_dir: PathBuf,
    inner: Mutex<AutoplayInner>,
    counter: AtomicU64,
}

#[derive(Debug, Default)]
struct AutoplayInner {
    /// Commands pushed by the RPC thread, drained by the Bevy system once
    /// per `Update`.
    pending: VecDeque<AutoplayCommand>,
    /// Last status snapshot published by the Bevy system; read by the RPC
    /// thread without blocking the schedule.
    last_status: AutoplayStatusSnapshot,
}

impl AutoplayShared {
    fn new(artifact_dir: PathBuf) -> Self {
        Self {
            artifact_dir,
            inner: Mutex::new(AutoplayInner::default()),
            counter: AtomicU64::new(0),
        }
    }

    fn next_seq(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Wrap an `Arc<AutoplayShared>` as a Bevy `Resource` so systems can
    /// access it. Stored separately from the `Arc` the RPC thread holds so
    /// `AutoplayShared` itself does not need to be `Resource`-aware.
    fn handle(shared: Arc<AutoplayShared>) -> AutoplaySharedHandle {
        AutoplaySharedHandle(shared)
    }
}

#[derive(Resource, Clone)]
struct AutoplaySharedHandle(Arc<AutoplayShared>);

#[derive(Debug, Clone)]
enum AutoplayCommand {
    Input {
        keys_down: Vec<KeyCode>,
        keys_up: Vec<KeyCode>,
        mouse_down: Vec<MouseButton>,
        mouse_up: Vec<MouseButton>,
        cursor: Option<Vec2>,
        scroll: Option<Vec2>,
    },
    ClearInput,
    Screenshot {
        seq: u64,
        reason: String,
    },
}

#[derive(Debug, Clone, Default)]
struct AutoplayStatusSnapshot {
    /// Monotonic frame counter set by the publisher system.
    frame: u64,
    /// `bevy::time::Real` seconds since process start.
    uptime_secs: f64,
    /// Logical size of the primary window, if present.
    window_logical_size: Option<(f32, f32)>,
    /// Last known cursor position in the primary window's logical coords.
    cursor_logical: Option<(f32, f32)>,
    /// Currently pressed keys (debug names).
    keys_pressed: Vec<String>,
    /// Currently pressed mouse buttons.
    mouse_pressed: Vec<String>,
    /// Number of commands drained since process start.
    commands_drained: u64,
    /// Number of screenshots requested since process start.
    screenshots_requested: u64,
    /// Last `path` written by the screenshot command (relative to artifact
    /// dir).
    last_screenshot_path: Option<String>,
    /// Last error string from the harness (input parse, screenshot, …).
    last_error: Option<String>,
    /// Debug name of the current `RoundPhase` (e.g. `"Placement"`). `null`
    /// before the first `S2CPhaseChanged` lands on the client.
    phase_label: Option<String>,
    /// Round number from `CurrentClientPhase`. `null` until first phase change.
    round: Option<u32>,
    /// Debug name of the `ClientState` machine state (`"Lobby"` or
    /// `"InSession"`). `null` if the state resource is absent.
    client_state_label: Option<String>,
}

// ---------- Bevy systems ----------

fn drain_commands_system(
    handle: Res<AutoplaySharedHandle>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut wheel: MessageWriter<MouseWheel>,
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut commands: Commands,
) {
    let mut drained: Vec<AutoplayCommand> = Vec::new();
    {
        let mut inner = match handle.0.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        drained.extend(inner.pending.drain(..));
    }
    if drained.is_empty() {
        return;
    }

    for cmd in drained {
        match cmd {
            AutoplayCommand::Input {
                keys_down,
                keys_up,
                mouse_down,
                mouse_up,
                cursor,
                scroll,
            } => {
                for k in keys_down {
                    keys.press(k);
                }
                for k in keys_up {
                    keys.release(k);
                }
                for b in mouse_down {
                    mouse.press(b);
                }
                for b in mouse_up {
                    mouse.release(b);
                }
                // Cursor warp + scroll need the primary window entity, so
                // we fetch it once per command.
                let primary_entity = if let Ok((entity, mut window)) = windows.single_mut() {
                    if let Some(pos) = cursor {
                        window.set_cursor_position(Some(pos));
                    }
                    Some(entity)
                } else {
                    None
                };
                if let Some(s) = scroll {
                    wheel.write(MouseWheel {
                        unit: MouseScrollUnit::Pixel,
                        x: s.x,
                        y: s.y,
                        window: primary_entity.unwrap_or(Entity::PLACEHOLDER),
                    });
                }
            }
            AutoplayCommand::ClearInput => {
                keys.release_all();
                mouse.release_all();
            }
            AutoplayCommand::Screenshot { seq, reason } => {
                let rel_path = PathBuf::from("screenshots").join(format!("{:06}.png", seq));
                let abs_path = handle.0.artifact_dir.join(&rel_path);
                if let Some(parent) = abs_path.parent() {
                    if let Err(err) = fs::create_dir_all(parent) {
                        record_error(&handle, format!("screenshot mkdir failed: {err}"));
                        continue;
                    }
                }
                // Sidecar JSON (best-effort) — written before the PNG so a
                // reviewer always sees the request even on capture failure.
                let sidecar = abs_path.with_extension("json");
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);
                let sidecar_body = format!(
                    "{{\"seq\":{seq},\"reason\":{q_reason},\"requested_at_unix_ms\":{now_ms},\"relative_path\":{q_rel}}}\n",
                    seq = seq,
                    q_reason = json_string(&reason),
                    now_ms = now_ms,
                    q_rel = json_string(&rel_path.to_string_lossy()),
                );
                if let Err(err) = fs::write(&sidecar, sidecar_body) {
                    record_error(&handle, format!("screenshot sidecar failed: {err}"));
                }
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(abs_path.clone()));
                {
                    let mut inner = match handle.0.inner.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    inner.last_status.last_screenshot_path =
                        Some(rel_path.to_string_lossy().into_owned());
                    inner.last_status.screenshots_requested =
                        inner.last_status.screenshots_requested.saturating_add(1);
                }
            }
        }
        {
            let mut inner = match handle.0.inner.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            inner.last_status.commands_drained =
                inner.last_status.commands_drained.saturating_add(1);
        }
    }
}

fn publish_status_system(
    handle: Res<AutoplaySharedHandle>,
    time: Res<Time<bevy::time::Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    phase: Option<Res<CurrentClientPhase>>,
    client_state: Option<Res<State<ClientState>>>,
    mut frame_counter: Local<u64>,
) {
    *frame_counter = frame_counter.wrapping_add(1);
    let (window_size, cursor) = if let Ok(window) = windows.single() {
        let size = (window.width(), window.height());
        let cursor = window.cursor_position().map(|p| (p.x, p.y));
        (Some(size), cursor)
    } else {
        (None, None)
    };
    let keys_pressed: Vec<String> = keys
        .get_pressed()
        .map(|k| format!("{:?}", k))
        .collect();
    let mouse_pressed: Vec<String> = mouse
        .get_pressed()
        .map(|b| format!("{:?}", b))
        .collect();
    let (phase_label, round) = if let Some(p) = phase {
        (Some(format!("{:?}", p.phase)), Some(p.round))
    } else {
        (None, None)
    };
    let client_state_label = client_state.map(|s| format!("{:?}", s.get()));

    let mut inner = match handle.0.inner.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    inner.last_status.frame = *frame_counter;
    inner.last_status.uptime_secs = time.elapsed_secs_f64();
    inner.last_status.window_logical_size = window_size;
    inner.last_status.cursor_logical = cursor;
    inner.last_status.keys_pressed = keys_pressed;
    inner.last_status.mouse_pressed = mouse_pressed;
    inner.last_status.phase_label = phase_label;
    inner.last_status.round = round;
    inner.last_status.client_state_label = client_state_label;

    let snapshot = inner.last_status.clone();
    let artifact_dir = handle.0.artifact_dir.clone();
    drop(inner);

    if *frame_counter % 15 == 0 {
        let status_path = artifact_dir.join("status.json");
        if let Err(err) = fs::write(&status_path, render_status_json(&snapshot)) {
            tracing::warn!(
                target: "client::autoplay",
                error = %err,
                path = %status_path.display(),
                "failed to write status.json"
            );
        }
    }
}

fn record_error(handle: &AutoplaySharedHandle, msg: String) {
    tracing::warn!(target: "client::autoplay", "{msg}");
    let mut inner = match handle.0.inner.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    inner.last_status.last_error = Some(msg);
}

// ---------- RPC server (HTTP/1.1 + JSON-RPC 2.0, single-threaded) ----------

fn run_rpc_server(listener: TcpListener, shared: Arc<AutoplayShared>) {
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let shared = Arc::clone(&shared);
                // One thread per connection keeps the code dependency-free
                // (no async runtime); volume is tiny (driver tick rate).
                if let Err(err) = thread::Builder::new()
                    .name("ccgs-autoplay-conn".into())
                    .spawn(move || handle_connection(s, shared))
                {
                    tracing::warn!(
                        target: "client::autoplay",
                        error = %err,
                        "failed to spawn autoplay conn thread"
                    );
                }
            }
            Err(err) => {
                tracing::warn!(target: "client::autoplay", error = %err, "accept failed");
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, shared: Arc<AutoplayShared>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let mut reader = BufReader::new(stream.try_clone().expect("clone tcp stream"));
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let mut content_length: usize = 0;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            return;
        }
        let trimmed = header.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
            if let Ok(n) = rest.trim().parse::<usize>() {
                content_length = n;
            }
        }
    }
    let mut body = vec![0u8; content_length];
    if content_length > 0 && reader.read_exact(&mut body).is_err() {
        return;
    }
    let body_str = String::from_utf8_lossy(&body).to_string();

    // Tolerant single-call dispatch. The body is expected to be a JSON-RPC 2.0
    // request `{"jsonrpc":"2.0","id":<n>,"method":"<m>","params":<obj>}`.
    let parsed = parse_json_value(&body_str);
    let id = parsed.as_ref().and_then(|v| v.get("id")).cloned();
    let method = parsed
        .as_ref()
        .and_then(|v| v.get("method"))
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string();
    let params = parsed
        .as_ref()
        .and_then(|v| v.get("params"))
        .cloned()
        .unwrap_or(JsonValue::Object(Vec::new()));

    let result = dispatch(&method, &params, &shared);

    let body = match result {
        Ok(result) => format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{result}}}",
            id = id.map(|v| v.to_json()).unwrap_or_else(|| "null".to_string()),
            result = result,
        ),
        Err(err) => format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"error\":{{\"code\":-32000,\"message\":{msg}}}}}",
            id = id.map(|v| v.to_json()).unwrap_or_else(|| "null".to_string()),
            msg = json_string(&err),
        ),
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len(),
        body = body,
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn dispatch(method: &str, params: &JsonValue, shared: &Arc<AutoplayShared>) -> Result<String, String> {
    match method {
        "autoplay/capabilities" => Ok(capabilities_json()),
        "autoplay/status" => {
            let inner = match shared.inner.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            Ok(render_status_json(&inner.last_status))
        }
        "autoplay/clear_input" => {
            push_command(shared, AutoplayCommand::ClearInput);
            Ok(format!("{{\"queued\":{}}}", shared.next_seq()))
        }
        "autoplay/screenshot" => {
            let reason = params
                .get("reason")
                .and_then(JsonValue::as_str)
                .unwrap_or("rpc")
                .to_string();
            let seq = shared.next_seq();
            push_command(
                shared,
                AutoplayCommand::Screenshot {
                    seq,
                    reason,
                },
            );
            let rel = format!("screenshots/{:06}.png", seq);
            Ok(format!(
                "{{\"queued\":{seq},\"relative_path\":{rel}}}",
                seq = seq,
                rel = json_string(&rel),
            ))
        }
        "autoplay/input" => {
            let cmd = parse_input(params)?;
            push_command(shared, cmd);
            Ok(format!("{{\"queued\":{}}}", shared.next_seq()))
        }
        other => Err(format!("unknown method: {other}")),
    }
}

fn push_command(shared: &Arc<AutoplayShared>, cmd: AutoplayCommand) {
    let mut inner = match shared.inner.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    inner.pending.push_back(cmd);
}

fn capabilities_json() -> String {
    format!(
        "{{\"version\":{ver},\"methods\":{{\"capabilities\":\"autoplay/capabilities\",\"status\":\"autoplay/status\",\"input\":\"autoplay/input\",\"clear\":\"autoplay/clear_input\",\"screenshot\":\"autoplay/screenshot\"}},\"input\":{{\"keys\":\"by name (KeyA, Space, Escape, F9 …); see KeyCode debug names\",\"mouse_buttons\":\"Left|Right|Middle|Back|Forward\",\"cursor\":\"logical [x,y] in primary window coords\",\"scroll\":\"[x,y] mouse-wheel delta\"}},\"status_fields\":{{\"phase_label\":\"Debug name of current RoundPhase (e.g. Placement); null before first S2CPhaseChanged\",\"round\":\"round number from CurrentClientPhase; null until first phase change\",\"client_state_label\":\"ClientState machine state (Lobby or InSession); null if absent\"}},\"invariants\":\"low-level input only; no semantic gameplay verbs or ECS mutation\"}}",
        ver = AUTOPLAY_RPC_VERSION,
    )
}

fn render_status_json(s: &AutoplayStatusSnapshot) -> String {
    let mut out = String::new();
    out.push_str("{");
    out.push_str(&format!("\"schema\":\"autoplay_status_v2\","));
    out.push_str(&format!("\"frame\":{},", s.frame));
    out.push_str(&format!("\"uptime_secs\":{},", json_float(s.uptime_secs)));
    out.push_str(&format!(
        "\"window_logical_size\":{},",
        match s.window_logical_size {
            Some((w, h)) => format!("[{},{}]", json_float(w as f64), json_float(h as f64)),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "\"cursor_logical\":{},",
        match s.cursor_logical {
            Some((x, y)) => format!("[{},{}]", json_float(x as f64), json_float(y as f64)),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "\"keys_pressed\":{},",
        json_string_array(&s.keys_pressed)
    ));
    out.push_str(&format!(
        "\"mouse_pressed\":{},",
        json_string_array(&s.mouse_pressed)
    ));
    out.push_str(&format!("\"commands_drained\":{},", s.commands_drained));
    out.push_str(&format!(
        "\"screenshots_requested\":{},",
        s.screenshots_requested
    ));
    out.push_str(&format!(
        "\"last_screenshot_path\":{},",
        match &s.last_screenshot_path {
            Some(p) => json_string(p),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "\"last_error\":{},",
        match &s.last_error {
            Some(e) => json_string(e),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "\"phase_label\":{},",
        match &s.phase_label {
            Some(p) => json_string(p),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "\"round\":{},",
        match s.round {
            Some(r) => r.to_string(),
            None => "null".to_string(),
        }
    ));
    out.push_str(&format!(
        "\"client_state_label\":{}",
        match &s.client_state_label {
            Some(l) => json_string(l),
            None => "null".to_string(),
        }
    ));
    out.push_str("}");
    out
}

fn parse_input(params: &JsonValue) -> Result<AutoplayCommand, String> {
    let mut keys_down = Vec::new();
    let mut keys_up = Vec::new();
    let mut mouse_down = Vec::new();
    let mut mouse_up = Vec::new();
    let mut cursor: Option<Vec2> = None;
    let mut scroll: Option<Vec2> = None;

    if let Some(arr) = params.get("keys_down").and_then(JsonValue::as_array) {
        for entry in arr {
            if let Some(name) = entry.as_str() {
                let code = parse_keycode(name).ok_or_else(|| format!("unknown key: {name}"))?;
                keys_down.push(code);
            }
        }
    }
    if let Some(arr) = params.get("keys_up").and_then(JsonValue::as_array) {
        for entry in arr {
            if let Some(name) = entry.as_str() {
                let code = parse_keycode(name).ok_or_else(|| format!("unknown key: {name}"))?;
                keys_up.push(code);
            }
        }
    }
    if let Some(arr) = params.get("mouse_down").and_then(JsonValue::as_array) {
        for entry in arr {
            if let Some(name) = entry.as_str() {
                let btn = parse_mouse_button(name)
                    .ok_or_else(|| format!("unknown mouse button: {name}"))?;
                mouse_down.push(btn);
            }
        }
    }
    if let Some(arr) = params.get("mouse_up").and_then(JsonValue::as_array) {
        for entry in arr {
            if let Some(name) = entry.as_str() {
                let btn = parse_mouse_button(name)
                    .ok_or_else(|| format!("unknown mouse button: {name}"))?;
                mouse_up.push(btn);
            }
        }
    }
    if let Some(c) = params.get("cursor") {
        if let Some(arr) = c.get("screen").and_then(JsonValue::as_array) {
            if arr.len() == 2 {
                let x = arr[0].as_f64().ok_or("cursor.screen[0] not a number")? as f32;
                let y = arr[1].as_f64().ok_or("cursor.screen[1] not a number")? as f32;
                cursor = Some(Vec2::new(x, y));
            }
        }
    }
    if let Some(s) = params.get("scroll") {
        if let Some(arr) = s.as_array() {
            if arr.len() == 2 {
                let x = arr[0].as_f64().ok_or("scroll[0] not a number")? as f32;
                let y = arr[1].as_f64().ok_or("scroll[1] not a number")? as f32;
                scroll = Some(Vec2::new(x, y));
            }
        }
    }

    Ok(AutoplayCommand::Input {
        keys_down,
        keys_up,
        mouse_down,
        mouse_up,
        cursor,
        scroll,
    })
}

fn parse_mouse_button(name: &str) -> Option<MouseButton> {
    match name {
        "Left" | "left" => Some(MouseButton::Left),
        "Right" | "right" => Some(MouseButton::Right),
        "Middle" | "middle" => Some(MouseButton::Middle),
        "Back" | "back" => Some(MouseButton::Back),
        "Forward" | "forward" => Some(MouseButton::Forward),
        _ => None,
    }
}

/// Parse a subset of `KeyCode` debug-name strings. Coverage focuses on the
/// keys autoplay recipes actually need: alphanumerics, arrows, function
/// keys, common navigation/edit, modifiers, and digits. Add to this table
/// when a recipe requires a key that is not yet listed.
fn parse_keycode(name: &str) -> Option<KeyCode> {
    // Letters
    match name {
        "KeyA" => return Some(KeyCode::KeyA),
        "KeyB" => return Some(KeyCode::KeyB),
        "KeyC" => return Some(KeyCode::KeyC),
        "KeyD" => return Some(KeyCode::KeyD),
        "KeyE" => return Some(KeyCode::KeyE),
        "KeyF" => return Some(KeyCode::KeyF),
        "KeyG" => return Some(KeyCode::KeyG),
        "KeyH" => return Some(KeyCode::KeyH),
        "KeyI" => return Some(KeyCode::KeyI),
        "KeyJ" => return Some(KeyCode::KeyJ),
        "KeyK" => return Some(KeyCode::KeyK),
        "KeyL" => return Some(KeyCode::KeyL),
        "KeyM" => return Some(KeyCode::KeyM),
        "KeyN" => return Some(KeyCode::KeyN),
        "KeyO" => return Some(KeyCode::KeyO),
        "KeyP" => return Some(KeyCode::KeyP),
        "KeyQ" => return Some(KeyCode::KeyQ),
        "KeyR" => return Some(KeyCode::KeyR),
        "KeyS" => return Some(KeyCode::KeyS),
        "KeyT" => return Some(KeyCode::KeyT),
        "KeyU" => return Some(KeyCode::KeyU),
        "KeyV" => return Some(KeyCode::KeyV),
        "KeyW" => return Some(KeyCode::KeyW),
        "KeyX" => return Some(KeyCode::KeyX),
        "KeyY" => return Some(KeyCode::KeyY),
        "KeyZ" => return Some(KeyCode::KeyZ),
        _ => {}
    }
    match name {
        "Digit0" => return Some(KeyCode::Digit0),
        "Digit1" => return Some(KeyCode::Digit1),
        "Digit2" => return Some(KeyCode::Digit2),
        "Digit3" => return Some(KeyCode::Digit3),
        "Digit4" => return Some(KeyCode::Digit4),
        "Digit5" => return Some(KeyCode::Digit5),
        "Digit6" => return Some(KeyCode::Digit6),
        "Digit7" => return Some(KeyCode::Digit7),
        "Digit8" => return Some(KeyCode::Digit8),
        "Digit9" => return Some(KeyCode::Digit9),
        _ => {}
    }
    match name {
        "F1" => return Some(KeyCode::F1),
        "F2" => return Some(KeyCode::F2),
        "F3" => return Some(KeyCode::F3),
        "F4" => return Some(KeyCode::F4),
        "F5" => return Some(KeyCode::F5),
        "F6" => return Some(KeyCode::F6),
        "F7" => return Some(KeyCode::F7),
        "F8" => return Some(KeyCode::F8),
        "F9" => return Some(KeyCode::F9),
        "F10" => return Some(KeyCode::F10),
        "F11" => return Some(KeyCode::F11),
        "F12" => return Some(KeyCode::F12),
        _ => {}
    }
    match name {
        "Space" => Some(KeyCode::Space),
        "Enter" | "Return" => Some(KeyCode::Enter),
        "Escape" | "Esc" => Some(KeyCode::Escape),
        "Tab" => Some(KeyCode::Tab),
        "Backspace" => Some(KeyCode::Backspace),
        "Delete" => Some(KeyCode::Delete),
        "ArrowUp" | "Up" => Some(KeyCode::ArrowUp),
        "ArrowDown" | "Down" => Some(KeyCode::ArrowDown),
        "ArrowLeft" | "Left" => Some(KeyCode::ArrowLeft),
        "ArrowRight" | "Right" => Some(KeyCode::ArrowRight),
        "Home" => Some(KeyCode::Home),
        "End" => Some(KeyCode::End),
        "PageUp" => Some(KeyCode::PageUp),
        "PageDown" => Some(KeyCode::PageDown),
        "ShiftLeft" | "Shift" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" | "Control" | "Ctrl" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" | "Alt" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "SuperLeft" | "Super" => Some(KeyCode::SuperLeft),
        "SuperRight" => Some(KeyCode::SuperRight),
        _ => None,
    }
}

// ---------- minimal JSON parser/encoder (no serde dep here) ----------

#[derive(Debug, Clone)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(items) = self {
            items.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }
    fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }
    fn as_array(&self) -> Option<&Vec<JsonValue>> {
        if let JsonValue::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }
    fn as_f64(&self) -> Option<f64> {
        if let JsonValue::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }
    fn to_json(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => json_float(*n),
            JsonValue::String(s) => json_string(s),
            JsonValue::Array(items) => {
                let inner: Vec<String> = items.iter().map(JsonValue::to_json).collect();
                format!("[{}]", inner.join(","))
            }
            JsonValue::Object(items) => {
                let inner: Vec<String> = items
                    .iter()
                    .map(|(k, v)| format!("{}:{}", json_string(k), v.to_json()))
                    .collect();
                format!("{{{}}}", inner.join(","))
            }
        }
    }
}

fn parse_json_value(input: &str) -> Option<JsonValue> {
    let bytes = input.as_bytes();
    let mut i = 0usize;
    skip_ws(bytes, &mut i);
    let v = parse_value(bytes, &mut i)?;
    skip_ws(bytes, &mut i);
    Some(v)
}

fn skip_ws(bytes: &[u8], i: &mut usize) {
    while *i < bytes.len() {
        match bytes[*i] {
            b' ' | b'\t' | b'\n' | b'\r' => *i += 1,
            _ => break,
        }
    }
}

fn parse_value(bytes: &[u8], i: &mut usize) -> Option<JsonValue> {
    skip_ws(bytes, i);
    if *i >= bytes.len() {
        return None;
    }
    match bytes[*i] {
        b'"' => parse_string(bytes, i).map(JsonValue::String),
        b'{' => parse_object(bytes, i),
        b'[' => parse_array(bytes, i),
        b't' | b'f' => parse_bool(bytes, i),
        b'n' => parse_null(bytes, i),
        _ => parse_number(bytes, i),
    }
}

fn parse_string(bytes: &[u8], i: &mut usize) -> Option<String> {
    if bytes[*i] != b'"' {
        return None;
    }
    *i += 1;
    let mut out = String::new();
    while *i < bytes.len() {
        match bytes[*i] {
            b'"' => {
                *i += 1;
                return Some(out);
            }
            b'\\' => {
                *i += 1;
                if *i >= bytes.len() {
                    return None;
                }
                let c = match bytes[*i] {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    b'b' => '\u{0008}',
                    b'f' => '\u{000C}',
                    b'u' => {
                        if *i + 4 >= bytes.len() {
                            return None;
                        }
                        let hex = std::str::from_utf8(&bytes[*i + 1..*i + 5]).ok()?;
                        let code = u32::from_str_radix(hex, 16).ok()?;
                        *i += 4;
                        char::from_u32(code)?
                    }
                    other => other as char,
                };
                out.push(c);
                *i += 1;
            }
            other => {
                out.push(other as char);
                *i += 1;
            }
        }
    }
    None
}

fn parse_object(bytes: &[u8], i: &mut usize) -> Option<JsonValue> {
    if bytes[*i] != b'{' {
        return None;
    }
    *i += 1;
    let mut items = Vec::new();
    loop {
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return None;
        }
        if bytes[*i] == b'}' {
            *i += 1;
            return Some(JsonValue::Object(items));
        }
        let key = parse_string(bytes, i)?;
        skip_ws(bytes, i);
        if *i >= bytes.len() || bytes[*i] != b':' {
            return None;
        }
        *i += 1;
        let value = parse_value(bytes, i)?;
        items.push((key, value));
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return None;
        }
        match bytes[*i] {
            b',' => {
                *i += 1;
            }
            b'}' => {
                *i += 1;
                return Some(JsonValue::Object(items));
            }
            _ => return None,
        }
    }
}

fn parse_array(bytes: &[u8], i: &mut usize) -> Option<JsonValue> {
    if bytes[*i] != b'[' {
        return None;
    }
    *i += 1;
    let mut items = Vec::new();
    loop {
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return None;
        }
        if bytes[*i] == b']' {
            *i += 1;
            return Some(JsonValue::Array(items));
        }
        let v = parse_value(bytes, i)?;
        items.push(v);
        skip_ws(bytes, i);
        if *i >= bytes.len() {
            return None;
        }
        match bytes[*i] {
            b',' => *i += 1,
            b']' => {
                *i += 1;
                return Some(JsonValue::Array(items));
            }
            _ => return None,
        }
    }
}

fn parse_bool(bytes: &[u8], i: &mut usize) -> Option<JsonValue> {
    if bytes[*i..].starts_with(b"true") {
        *i += 4;
        Some(JsonValue::Bool(true))
    } else if bytes[*i..].starts_with(b"false") {
        *i += 5;
        Some(JsonValue::Bool(false))
    } else {
        None
    }
}

fn parse_null(bytes: &[u8], i: &mut usize) -> Option<JsonValue> {
    if bytes[*i..].starts_with(b"null") {
        *i += 4;
        Some(JsonValue::Null)
    } else {
        None
    }
}

fn parse_number(bytes: &[u8], i: &mut usize) -> Option<JsonValue> {
    let start = *i;
    if bytes[*i] == b'-' || bytes[*i] == b'+' {
        *i += 1;
    }
    while *i < bytes.len()
        && (bytes[*i].is_ascii_digit() || matches!(bytes[*i], b'.' | b'e' | b'E' | b'-' | b'+'))
    {
        *i += 1;
    }
    let raw = std::str::from_utf8(&bytes[start..*i]).ok()?;
    raw.parse::<f64>().ok().map(JsonValue::Number)
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_string_array(arr: &[String]) -> String {
    let mut out = String::new();
    out.push('[');
    for (idx, s) in arr.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_string(s));
    }
    out.push(']');
    out
}

fn json_float(n: f64) -> String {
    if n.is_finite() {
        format!("{}", n)
    } else {
        "null".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keycode_letters_digits_function_keys() {
        assert_eq!(parse_keycode("KeyA"), Some(KeyCode::KeyA));
        assert_eq!(parse_keycode("Digit5"), Some(KeyCode::Digit5));
        assert_eq!(parse_keycode("F9"), Some(KeyCode::F9));
        assert_eq!(parse_keycode("Escape"), Some(KeyCode::Escape));
        assert_eq!(parse_keycode("ArrowLeft"), Some(KeyCode::ArrowLeft));
        assert_eq!(parse_keycode("Bogus"), None);
    }

    #[test]
    fn parses_mouse_button_names() {
        assert_eq!(parse_mouse_button("Left"), Some(MouseButton::Left));
        assert_eq!(parse_mouse_button("right"), Some(MouseButton::Right));
        assert_eq!(parse_mouse_button("Middle"), Some(MouseButton::Middle));
        assert_eq!(parse_mouse_button("nope"), None);
    }

    #[test]
    fn config_from_env_defaults_to_off() {
        // Avoids mutating the process env; we just exercise default_artifact_dir.
        let dir = default_artifact_dir();
        // PathBuf::join on Windows yields the platform separator. Compare
        // components instead of the stringified form to stay portable.
        let comps: Vec<String> = dir
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            &comps[..4],
            &[
                "production".to_string(),
                "qa".to_string(),
                "evidence".to_string(),
                "autoplay-runs".to_string(),
            ]
        );
        assert!(comps.len() == 5, "expected timestamp suffix, got {:?}", comps);
    }

    #[test]
    fn utc_stamp_format() {
        // 2026-05-21 00:00:00 UTC = 1779494400
        let s = format_utc_stamp(1779494400);
        assert_eq!(s.len(), "YYYYMMDD-HHMMSS-Z".len());
        assert!(s.ends_with("-Z"));
        // year prefix
        assert!(s.starts_with("2026"));
    }

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string("hello"), "\"hello\"");
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("c\\d"), "\"c\\\\d\"");
        assert_eq!(json_string("e\nf"), "\"e\\nf\"");
    }

    #[test]
    fn json_parse_round_trip_basic_object() {
        let v = parse_json_value("{\"a\":1,\"b\":\"x\",\"c\":[true,null]}").expect("parse");
        assert_eq!(v.get("a").and_then(JsonValue::as_f64), Some(1.0));
        assert_eq!(v.get("b").and_then(JsonValue::as_str), Some("x"));
        let arr = v.get("c").and_then(JsonValue::as_array).expect("array");
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn parse_input_decodes_low_level_fields() {
        let payload = parse_json_value(
            r#"{"keys_down":["KeyA","Space"],"keys_up":["Escape"],"mouse_down":["Left"],"mouse_up":["Right"],"cursor":{"screen":[100,200]},"scroll":[1.0,-2.5]}"#,
        )
        .expect("parse");
        let cmd = parse_input(&payload).expect("input");
        match cmd {
            AutoplayCommand::Input {
                keys_down,
                keys_up,
                mouse_down,
                mouse_up,
                cursor,
                scroll,
            } => {
                assert_eq!(keys_down, vec![KeyCode::KeyA, KeyCode::Space]);
                assert_eq!(keys_up, vec![KeyCode::Escape]);
                assert_eq!(mouse_down, vec![MouseButton::Left]);
                assert_eq!(mouse_up, vec![MouseButton::Right]);
                assert_eq!(cursor, Some(Vec2::new(100.0, 200.0)));
                assert_eq!(scroll, Some(Vec2::new(1.0, -2.5)));
            }
            _ => panic!("expected Input"),
        }
    }

    #[test]
    fn parse_input_rejects_unknown_key() {
        let payload = parse_json_value(r#"{"keys_down":["BananaKey"]}"#).expect("parse");
        assert!(parse_input(&payload).is_err());
    }

    #[test]
    fn capabilities_json_is_valid_and_lists_methods() {
        let s = capabilities_json();
        let v = parse_json_value(&s).expect("valid json");
        assert!(v.get("methods").is_some());
        assert_eq!(
            v.get("version").and_then(JsonValue::as_f64),
            Some(AUTOPLAY_RPC_VERSION as f64)
        );
    }

    #[test]
    fn render_status_json_is_valid() {
        let s = render_status_json(&AutoplayStatusSnapshot {
            frame: 42,
            uptime_secs: 1.25,
            window_logical_size: Some((1280.0, 720.0)),
            cursor_logical: Some((10.0, 20.0)),
            keys_pressed: vec!["KeyA".into()],
            mouse_pressed: vec!["Left".into()],
            commands_drained: 3,
            screenshots_requested: 1,
            last_screenshot_path: Some("screenshots/000000.png".into()),
            last_error: None,
            phase_label: None,
            round: None,
            client_state_label: None,
        });
        let v = parse_json_value(&s).expect("status json valid");
        assert_eq!(v.get("frame").and_then(JsonValue::as_f64), Some(42.0));
        assert_eq!(
            v.get("schema").and_then(JsonValue::as_str),
            Some("autoplay_status_v2")
        );
    }

    #[test]
    fn render_status_json_includes_phase_fields() {
        let s = render_status_json(&AutoplayStatusSnapshot {
            frame: 10,
            uptime_secs: 0.5,
            phase_label: Some("Placement".into()),
            round: Some(3),
            client_state_label: Some("InSession".into()),
            ..AutoplayStatusSnapshot::default()
        });
        let v = parse_json_value(&s).expect("valid json");
        assert_eq!(
            v.get("phase_label").and_then(JsonValue::as_str),
            Some("Placement")
        );
        assert_eq!(v.get("round").and_then(JsonValue::as_f64), Some(3.0));
        assert_eq!(
            v.get("client_state_label").and_then(JsonValue::as_str),
            Some("InSession")
        );
    }

    #[test]
    fn render_status_json_phase_null_when_absent() {
        let s = render_status_json(&AutoplayStatusSnapshot::default());
        let v = parse_json_value(&s).expect("valid json");
        // Before any phase message, all three fields are null.
        assert!(matches!(v.get("phase_label"), Some(JsonValue::Null)));
        assert!(matches!(v.get("round"), Some(JsonValue::Null)));
        assert!(matches!(v.get("client_state_label"), Some(JsonValue::Null)));
    }

    #[test]
    fn capabilities_json_lists_status_fields() {
        let s = capabilities_json();
        let v = parse_json_value(&s).expect("valid json");
        assert_eq!(
            v.get("version").and_then(JsonValue::as_f64),
            Some(AUTOPLAY_RPC_VERSION as f64)
        );
        let sf = v.get("status_fields").expect("status_fields present");
        assert!(sf.get("phase_label").is_some());
        assert!(sf.get("round").is_some());
        assert!(sf.get("client_state_label").is_some());
    }
}
