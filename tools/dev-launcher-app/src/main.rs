// CCGS Dev Launcher -- two-button Windows EXE wrapping the dev-launcher
// PowerShell scripts (`tools/dev-launcher/Update-LatestMain.ps1` and
// `tools/dev-launcher/Start-TwoClients.ps1`).
//
// Button 1 -- "Rebuild Latest Main" -- invokes Update-LatestMain.ps1.
// Button 2 -- "Start Two-Client Play Session" -- invokes Start-TwoClients.ps1.
//
// The launcher is intentionally a thin wrapper: it does not duplicate any
// launcher logic. Repo root is resolved from the `CCGS_REPO_ROOT` env
// override, a sidecar file beside the EXE (written by
// `tools/dev-launcher/build-launcher-exe.ps1`), or by walking up from the
// EXE/cwd. The scripts themselves remain the source of truth for cargo
// policy, evidence dir naming, port selection, process spawning, and safety
// guards.

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
#![cfg(windows)]

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;

use std::cell::RefCell;
use std::env;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const APP_TITLE: &str = "CCGS Dev Launcher";
const APP_SUBTITLE: &str =
    "Windows desktop utility for latest-main rebuilds and local two-client sessions.";
const REBUILD_BUTTON_LABEL: &str = "Rebuild Latest Main";
const LAUNCH_BUTTON_LABEL: &str = "Start Two-Client Play Session";
const REBUILD_SCRIPT: &str = "tools\\dev-launcher\\Update-LatestMain.ps1";
const LAUNCH_SCRIPT: &str = "tools\\dev-launcher\\Start-TwoClients.ps1";
// Sidecar written next to the EXE by `tools/dev-launcher/build-launcher-exe.ps1`.
// Contains the absolute repo root path on the first non-blank line.
const SIDECAR_FILENAME: &str = "ccgs-dev-launcher.repo-root.txt";
// Canonical-checkout candidates used as a fallback when the sidecar is pinned
// to a non-main worker worktree (PROMPT 1290). The launcher walks this list in
// order and accepts the first entry that validates as a real repo. Override
// via the `CCGS_CANONICAL_REPO_ROOT` env var.
const CANONICAL_REPO_CANDIDATES: &[&str] = &["D:\\_DEV\\Work\\Claude-Code-Game-Studios"];
const MAIN_BRANCH: &str = "main";

// PROMPT 1309: dedicated play/build checkout. The launcher resolves a SECOND
// path -- the "play repo root" -- separate from the script-source repo root.
// `Rebuild Latest Main` and `Start Two-Client Play Session` operate inside the
// play root so the orchestrator/canonical checkout (which may be dirty or on a
// worker branch at any time) is never destructively switched. The dedicated
// checkout is materialised as a git worktree off the launcher repo root the
// first time `Rebuild Latest Main` runs.
const PLAY_REPO_DEFAULT: &str = "D:\\_DEV\\ccgs-play-main";
// Preferred env override (PROMPT 1309). `CCGS_CANONICAL_MAIN_ROOT` is accepted
// as an alias so users who already typed the latter form do not get a second
// surprise. Both point at an absolute path that the launcher hands to the
// PowerShell scripts via `-PlayRepoRoot`.
const PLAY_REPO_ENV: &str = "CCGS_PLAY_REPO_ROOT";
const PLAY_REPO_ENV_ALIAS: &str = "CCGS_CANONICAL_MAIN_ROOT";
// Start-TwoClients.ps1 writes per-run logs under this path; the launcher
// surfaces the exact directory parsed from script stdout.
#[cfg(test)]
const EVIDENCE_HINT: &str = "production/qa/evidence/dev-runs/";
const MAX_LOG_LINES: usize = 2000;
const TIMER_INTERVAL_MS: u64 = 150;

const WINDOW_SIZE: (i32, i32) = (980, 720);
const MIN_WINDOW_SIZE: (i32, i32) = (760, 560);
const LAUNCHER_ICON_BYTES: &[u8] = include_bytes!("../assets/ccgs-dev-launcher.ico");

const COLOR_HEADER_BG: [u8; 3] = [27, 43, 64];
const COLOR_HEADER_TEXT: [u8; 3] = [245, 249, 255];
const COLOR_HEADER_MUTED: [u8; 3] = [193, 210, 227];
const COLOR_PANEL_BG: [u8; 3] = [246, 248, 251];
const COLOR_PANEL_TEXT: [u8; 3] = [35, 45, 58];
const COLOR_PANEL_HEADING: [u8; 3] = [18, 30, 46];
const COLOR_STATUS_IDLE: [u8; 3] = [231, 238, 247];
const COLOR_STATUS_RUNNING: [u8; 3] = [226, 238, 252];
const COLOR_STATUS_SUCCESS: [u8; 3] = [223, 244, 232];
const COLOR_STATUS_WARNING: [u8; 3] = [255, 242, 209];
const COLOR_STATUS_ERROR: [u8; 3] = [255, 229, 229];
const COLOR_LOG_BG: [u8; 3] = [18, 25, 34];
const COLOR_LOG_TEXT: [u8; 3] = [224, 235, 244];

#[derive(Clone, Copy, PartialEq, Eq)]
enum JobKind {
    Rebuild,
    Launch,
}

impl JobKind {
    fn human(self) -> &'static str {
        match self {
            JobKind::Rebuild => "Rebuild Latest Main",
            JobKind::Launch => "Start Two-Client Play Session",
        }
    }

    fn script_rel(self) -> &'static str {
        match self {
            JobKind::Rebuild => REBUILD_SCRIPT,
            JobKind::Launch => LAUNCH_SCRIPT,
        }
    }
}

enum WorkerMessage {
    Line(String),
    EvidenceDir(PathBuf),
    Finished(i32),
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResolutionSource {
    Env,
    Sidecar,
    CanonicalFallback,
    ExeWalkUp,
    CwdWalkUp,
}

impl ResolutionSource {
    fn human(self) -> &'static str {
        match self {
            ResolutionSource::Env => "CCGS_REPO_ROOT env var",
            ResolutionSource::Sidecar => "sidecar file beside EXE",
            ResolutionSource::CanonicalFallback => "canonical-checkout fallback",
            ResolutionSource::ExeWalkUp => "walk-up from EXE directory",
            ResolutionSource::CwdWalkUp => "walk-up from current working directory",
        }
    }
}

#[derive(Debug)]
enum RepoRootResolution {
    Resolved {
        root: PathBuf,
        source: ResolutionSource,
    },
    Failed {
        attempts: Vec<String>,
    },
}

// PROMPT 1309: source of the play/build repo root path. The launcher reports
// this verbatim in diagnostics so testers can tell whether they are on an env
// override vs. the documented default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayRootSource {
    Env,             // `CCGS_PLAY_REPO_ROOT`
    LegacyEnv,       // `CCGS_CANONICAL_MAIN_ROOT` (alias)
    DedicatedDefault, // documented `D:\_DEV\ccgs-play-main`
}

impl PlayRootSource {
    fn human(self) -> &'static str {
        match self {
            PlayRootSource::Env => "CCGS_PLAY_REPO_ROOT env var",
            PlayRootSource::LegacyEnv => "CCGS_CANONICAL_MAIN_ROOT env var (alias)",
            PlayRootSource::DedicatedDefault => "documented dedicated default",
        }
    }
}

// Snapshot of what the launcher knows about the play/build checkout *before*
// the rebuild script runs. The script is the only thing that creates the
// worktree, switches branches, or merges -- the launcher just reports what is
// there right now so the user can see why a rebuild will proceed or refuse.
#[derive(Clone, Debug, PartialEq, Eq)]
enum PlayRootStatus {
    OnMain,                    // exists, validates, branch == "main"
    OnOtherBranch(String),     // exists, validates, branch == something else
    DetachedOrUnknown,         // exists, validates, but no usable branch label
    Missing,                   // path does not exist on disk
    InvalidRepo(String),       // exists, does not validate as a CCGS workspace
}

impl PlayRootStatus {
    fn human(&self) -> String {
        match self {
            PlayRootStatus::OnMain => "exists, on main".to_string(),
            PlayRootStatus::OnOtherBranch(b) => format!("exists, on branch '{}'", b),
            PlayRootStatus::DetachedOrUnknown => "exists, detached HEAD or unknown".to_string(),
            PlayRootStatus::Missing => {
                "missing -- will be created as a worktree on first rebuild".to_string()
            }
            PlayRootStatus::InvalidRepo(why) => {
                format!("path exists but is not a CCGS workspace ({})", why)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct PlayRootResolution {
    path: PathBuf,
    source: PlayRootSource,
    status: PlayRootStatus,
}

struct LauncherState {
    repo_root: Option<PathBuf>,
    repo_source: Option<ResolutionSource>,
    play_root: Option<PlayRootResolution>,
    error_message: Option<String>,
    job: Option<JobKind>,
    rx: Option<Receiver<WorkerMessage>>,
    last_exit: Option<i32>,
    last_evidence_dir: Option<PathBuf>,
    log_lines: Vec<String>,
    log_dirty: bool,
}

impl LauncherState {
    fn new(
        repo_root: Option<PathBuf>,
        repo_source: Option<ResolutionSource>,
        play_root: Option<PlayRootResolution>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            repo_root,
            repo_source,
            play_root,
            error_message,
            job: None,
            rx: None,
            last_exit: None,
            last_evidence_dir: None,
            log_lines: Vec::new(),
            log_dirty: false,
        }
    }

    fn busy(&self) -> bool {
        self.job.is_some()
    }

    fn append(&mut self, line: String) {
        if self.log_lines.len() >= MAX_LOG_LINES {
            let drain_to = self.log_lines.len() - MAX_LOG_LINES + 1;
            self.log_lines.drain(0..drain_to);
        }
        self.log_lines.push(line);
        self.log_dirty = true;
    }

    fn add_banner(&mut self, text: &str) {
        let sep = "=".repeat(text.len().min(72));
        self.append(format!("==== {} ====", text));
        self.append(sep);
    }
}

#[derive(Clone, Copy)]
enum StatusTone {
    Idle,
    Running,
    Success,
    Warning,
    Error,
}

impl StatusTone {
    fn colors(self) -> ([u8; 3], [u8; 3]) {
        match self {
            StatusTone::Idle => (COLOR_STATUS_IDLE, COLOR_PANEL_TEXT),
            StatusTone::Running => (COLOR_STATUS_RUNNING, COLOR_PANEL_TEXT),
            StatusTone::Success => (COLOR_STATUS_SUCCESS, COLOR_PANEL_TEXT),
            StatusTone::Warning => (COLOR_STATUS_WARNING, COLOR_PANEL_TEXT),
            StatusTone::Error => (COLOR_STATUS_ERROR, COLOR_PANEL_TEXT),
        }
    }
}

#[derive(Default, NwgUi)]
pub struct LauncherUi {
    #[nwg_control(size: WINDOW_SIZE, position: (220, 120), title: APP_TITLE, flags: "WINDOW|VISIBLE|MAIN_WINDOW|MINIMIZE_BOX")]
    #[nwg_events(OnWindowClose: [LauncherUi::on_close], OnInit: [LauncherUi::on_init], OnMinMaxInfo: [LauncherUi::set_min_size(SELF, EVT_DATA)])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 7, margin: [18, 18, 18, 18])]
    layout: nwg::GridLayout,

    #[nwg_control(background_color: Some(COLOR_HEADER_BG))]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, row_span: 2)]
    icon_frame: nwg::ImageFrame,

    #[nwg_control(text: APP_TITLE, flags: "VISIBLE|MULTI_LINE", background_color: Some(COLOR_HEADER_BG))]
    #[nwg_layout_item(layout: layout, col: 1, row: 0, col_span: 7, row_span: 2)]
    brand_label: nwg::RichLabel,

    #[nwg_control(text: REBUILD_BUTTON_LABEL)]
    #[nwg_events(OnButtonClick: [LauncherUi::on_rebuild])]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 4, row_span: 2)]
    rebuild_btn: nwg::Button,

    #[nwg_control(text: LAUNCH_BUTTON_LABEL)]
    #[nwg_events(OnButtonClick: [LauncherUi::on_launch])]
    #[nwg_layout_item(layout: layout, col: 4, row: 2, col_span: 4, row_span: 2)]
    launch_btn: nwg::Button,

    #[nwg_control(text: "Status: Idle - ready.", flags: "VISIBLE|MULTI_LINE", background_color: Some(COLOR_STATUS_IDLE))]
    #[nwg_layout_item(layout: layout, col: 0, row: 4, col_span: 8)]
    state_label: nwg::RichLabel,

    #[nwg_control(text: "Diagnostics", flags: "VISIBLE|MULTI_LINE", background_color: Some(COLOR_PANEL_BG))]
    #[nwg_layout_item(layout: layout, col: 0, row: 5, col_span: 8)]
    diagnostics_heading_label: nwg::RichLabel,

    #[nwg_control(text: "", readonly: true, flags: "VISIBLE|VSCROLL|HSCROLL|AUTOVSCROLL|AUTOHSCROLL|TAB_STOP|SAVE_SELECTION")]
    #[nwg_layout_item(layout: layout, col: 0, row: 6, col_span: 8, row_span: 4)]
    diagnostics_box: nwg::RichTextBox,

    #[nwg_control(text: "Script Output", flags: "VISIBLE|MULTI_LINE", background_color: Some(COLOR_PANEL_BG))]
    #[nwg_layout_item(layout: layout, col: 0, row: 10, col_span: 8)]
    log_heading_label: nwg::RichLabel,

    #[nwg_control(text: "", readonly: true, flags: "VISIBLE|VSCROLL|HSCROLL|AUTOVSCROLL|AUTOHSCROLL|TAB_STOP|SAVE_SELECTION")]
    #[nwg_layout_item(layout: layout, col: 0, row: 11, col_span: 8, row_span: 7)]
    log_box: nwg::RichTextBox,

    #[nwg_control(interval: Duration::from_millis(TIMER_INTERVAL_MS), active: true)]
    #[nwg_events(OnTimerTick: [LauncherUi::on_tick])]
    timer: nwg::AnimationTimer,

    state: Arc<Mutex<Option<LauncherState>>>,

    resolution_init: RefCell<Option<RepoRootResolution>>,

    launcher_icon: RefCell<Option<nwg::Icon>>,
    brand_font: RefCell<Option<nwg::Font>>,
    heading_font: RefCell<Option<nwg::Font>>,
    button_font: RefCell<Option<nwg::Font>>,
    body_font: RefCell<Option<nwg::Font>>,
    mono_font: RefCell<Option<nwg::Font>>,
}

impl LauncherUi {
    fn set_buttons_enabled(&self, enabled: bool) {
        self.rebuild_btn.set_enabled(enabled);
        self.launch_btn.set_enabled(enabled);
    }

    fn set_status(&self, text: &str, tone: StatusTone) {
        let (bg, fg) = tone.colors();
        set_rich_label_text(
            &self.state_label,
            text,
            fg,
            bg,
            Some(nwg::CharEffects::BOLD),
        );
    }

    fn refresh_diagnostics(&self) {
        let snapshot = {
            let guard = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            match guard.as_ref() {
                Some(state) => diagnostics_text(state),
                None => return,
            }
        };
        set_rich_text_box(
            &self.diagnostics_box,
            &snapshot,
            COLOR_PANEL_TEXT,
            COLOR_PANEL_BG,
            false,
        );
    }

    fn on_init(&self) {
        let mut icon = nwg::Icon::default();
        match nwg::Icon::builder()
            .source_bin(Some(LAUNCHER_ICON_BYTES))
            .size(Some((48, 48)))
            .strict(true)
            .build(&mut icon)
        {
            Ok(()) => {
                self.window.set_icon(Some(&icon));
                self.icon_frame.set_icon(Some(&icon));
                *self.launcher_icon.borrow_mut() = Some(icon);
            }
            Err(e) => {
                eprintln!(
                    "[ccgs-dev-launcher] icon build failed ({} bytes): {:?}",
                    LAUNCHER_ICON_BYTES.len(),
                    e
                );
            }
        }

        let brand_font = build_font("Segoe UI", 18, None);
        let heading_font = build_font("Segoe UI Semibold", 15, Some(650));
        let button_font = build_font("Segoe UI Semibold", 18, Some(650));
        let body_font = build_font("Segoe UI", 14, None);
        let mono_font = build_font("Consolas", 13, None);

        apply_font(&self.brand_label, brand_font.as_ref());
        apply_font(&self.diagnostics_heading_label, heading_font.as_ref());
        apply_font(&self.log_heading_label, heading_font.as_ref());
        apply_font(&self.rebuild_btn, button_font.as_ref());
        apply_font(&self.launch_btn, button_font.as_ref());
        apply_font(&self.state_label, body_font.as_ref());
        apply_font(&self.diagnostics_box, mono_font.as_ref());
        apply_font(&self.log_box, mono_font.as_ref());

        *self.brand_font.borrow_mut() = brand_font;
        *self.heading_font.borrow_mut() = heading_font;
        *self.button_font.borrow_mut() = button_font;
        *self.body_font.borrow_mut() = body_font;
        *self.mono_font.borrow_mut() = mono_font;

        set_brand_label(&self.brand_label);
        set_rich_label_text(
            &self.diagnostics_heading_label,
            "Diagnostics",
            COLOR_PANEL_HEADING,
            COLOR_PANEL_BG,
            Some(nwg::CharEffects::BOLD),
        );
        set_rich_label_text(
            &self.log_heading_label,
            "Script Output",
            COLOR_PANEL_HEADING,
            COLOR_PANEL_BG,
            Some(nwg::CharEffects::BOLD),
        );
        self.log_box.set_background_color(COLOR_LOG_BG);
        self.diagnostics_box.set_background_color(COLOR_PANEL_BG);

        let resolution = self.resolution_init.borrow_mut().take().unwrap_or_else(|| {
            RepoRootResolution::Failed {
                attempts: vec!["resolution missing at UI init".to_string()],
            }
        });

        let play_root_init = locate_play_root();

        let (
            repo_root,
            repo_source,
            error_message,
            init_lines,
            state_label_text,
            status_tone,
            buttons_enabled,
        ) = match resolution {
            RepoRootResolution::Resolved { root, source } => {
                let mut lines = Vec::new();
                lines.push(format!(
                    "Launcher repo root: {} (via {})",
                    root.display(),
                    source.human()
                ));
                let branch_label =
                    read_head_branch(&root).unwrap_or_else(|| "<detached or unknown>".to_string());
                lines.push(format!("Launcher branch: {}", branch_label));
                lines.push(format!(
                    "Play/build root: {} (via {})",
                    play_root_init.path.display(),
                    play_root_init.source.human()
                ));
                lines.push(format!(
                    "Play/build status: {}",
                    play_root_init.status.human()
                ));
                lines.push(format!("Scripts: {} | {}", REBUILD_SCRIPT, LAUNCH_SCRIPT));
                (
                    Some(root),
                    Some(source),
                    None,
                    lines,
                    "Status: Idle - ready.".to_string(),
                    StatusTone::Idle,
                    true,
                )
            }
            RepoRootResolution::Failed { attempts } => {
                let err = format!(
                    "ERROR: could not locate a canonical CCGS repo root. \
                         Set CCGS_REPO_ROOT to your canonical checkout, set \
                         CCGS_CANONICAL_REPO_ROOT to override the fallback, \
                         or rebuild the EXE via tools\\dev-launcher\\build-launcher-exe.ps1 \
                         from the canonical repo (writes the {} sidecar beside the EXE).",
                    SIDECAR_FILENAME
                );
                let mut lines = vec![err.clone(), "Attempts:".to_string()];
                for a in &attempts {
                    lines.push(format!("  - {}", a));
                }
                lines.push("Buttons are disabled until a valid repo root is resolved.".to_string());
                (
                    None,
                    None,
                    Some(err.clone()),
                    lines,
                    "Status: ERROR - repo root unresolved. Buttons disabled.".to_string(),
                    StatusTone::Error,
                    false,
                )
            }
        };

        let mut guard = self.state.lock().expect("state poisoned at init");
        let mut state =
            LauncherState::new(repo_root, repo_source, Some(play_root_init), error_message);
        for line in init_lines {
            state.append(line);
        }
        state.log_dirty = true;
        *guard = Some(state);
        drop(guard);

        self.set_status(&state_label_text, status_tone);
        self.set_buttons_enabled(buttons_enabled);
        self.refresh_diagnostics();
        // Flush initial log paint without waiting for the first timer tick.
        self.refresh_log();
    }

    fn set_min_size(&self, data: &nwg::EventData) {
        let data = data.on_min_max();
        data.set_min_size(MIN_WINDOW_SIZE.0, MIN_WINDOW_SIZE.1);
    }

    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn on_rebuild(&self) {
        self.start_job(JobKind::Rebuild);
    }

    fn on_launch(&self) {
        self.start_job(JobKind::Launch);
    }

    fn start_job(&self, job: JobKind) {
        let mut guard = self.state.lock().expect("state poisoned on click");
        let state = match guard.as_mut() {
            Some(s) => s,
            None => return,
        };
        if state.busy() {
            return;
        }

        let repo_root = match state.repo_root.clone() {
            Some(r) => r,
            None => {
                let msg = state
                    .error_message
                    .clone()
                    .unwrap_or_else(|| "repo root unresolved -- cannot launch".to_string());
                state.append(format!("ERROR: {}", msg));
                state.log_dirty = true;
                drop(guard);
                self.set_status(
                    "Status: ERROR - repo root unresolved. Buttons disabled.",
                    StatusTone::Error,
                );
                self.refresh_diagnostics();
                self.refresh_log();
                return;
            }
        };

        let script_path = repo_root.join(Path::new(job.script_rel()));
        if !script_path.exists() {
            state.append(format!(
                "ERROR: launcher script not found at {} -- is repo root correct?",
                script_path.display()
            ));
            state.log_dirty = true;
            drop(guard);
            self.set_status(
                "Status: ERROR - launcher script missing.",
                StatusTone::Error,
            );
            self.refresh_diagnostics();
            self.refresh_log();
            return;
        }

        let play_root_path = state.play_root.as_ref().map(|r| r.path.clone());
        state.last_exit = None;
        state.last_evidence_dir = None;
        state.add_banner(&format!("STARTING: {}", job.human()));
        state.append(format!("Script: {}", script_path.display()));
        if let Some(ref p) = play_root_path {
            state.append(format!("Play/build root passed to script: {}", p.display()));
        }

        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();
        let repo_root_clone = repo_root.clone();
        let script_path_clone = script_path.clone();
        let play_root_clone = play_root_path.clone();
        thread::spawn(move || {
            run_powershell_job(
                repo_root_clone,
                script_path_clone,
                play_root_clone,
                tx_clone,
            )
        });

        state.job = Some(job);
        state.rx = Some(rx);
        drop(guard);

        self.set_buttons_enabled(false);
        self.set_status(
            &format!("Status: RUNNING - {}", job.human()),
            StatusTone::Running,
        );
        self.refresh_diagnostics();
        self.refresh_log();
    }

    fn on_tick(&self) {
        let mut finished: Option<(JobKind, i32)> = None;
        let mut errored: Option<(JobKind, String)> = None;
        let mut diagnostics_dirty = false;
        {
            let mut guard = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let state = match guard.as_mut() {
                Some(s) => s,
                None => return,
            };
            // Temporarily take ownership of rx so we can mutably borrow other
            // state fields (e.g. append, add_banner) inside the recv loop.
            // Re-install rx after the loop unless the job ended.
            if let Some(rx) = state.rx.take() {
                let mut drained = 0usize;
                let mut keep_rx = true;
                loop {
                    match rx.try_recv() {
                        Ok(WorkerMessage::Line(l)) => {
                            state.append(l);
                            drained += 1;
                            if drained > 256 {
                                break;
                            }
                        }
                        Ok(WorkerMessage::EvidenceDir(p)) => {
                            state.last_evidence_dir = Some(p.clone());
                            diagnostics_dirty = true;
                        }
                        Ok(WorkerMessage::Finished(code)) => {
                            if let Some(job) = state.job.take() {
                                state.last_exit = Some(code);
                                state.add_banner(&format!(
                                    "FINISHED: {} (exit {})",
                                    job.human(),
                                    code
                                ));
                                finished = Some((job, code));
                                diagnostics_dirty = true;
                            }
                            keep_rx = false;
                            break;
                        }
                        Ok(WorkerMessage::Error(msg)) => {
                            if let Some(job) = state.job.take() {
                                state.add_banner(&format!("ERROR: {} -- {}", job.human(), msg));
                                errored = Some((job, msg));
                                diagnostics_dirty = true;
                            }
                            keep_rx = false;
                            break;
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            if let Some(job) = state.job.take() {
                                state.add_banner(&format!(
                                    "WARN: worker disconnected unexpectedly for {}",
                                    job.human()
                                ));
                                errored = Some((job, "worker channel disconnected".into()));
                                diagnostics_dirty = true;
                            }
                            keep_rx = false;
                            break;
                        }
                    }
                }
                if keep_rx {
                    state.rx = Some(rx);
                }
            }
        }

        if let Some((job, code)) = finished {
            self.set_buttons_enabled(true);
            let (msg, tone) = if code == 0 {
                (
                    format!("Status: DONE - {} exited 0.", job.human()),
                    StatusTone::Success,
                )
            } else {
                (
                    format!(
                        "Status: DONE WITH ERRORS - {} exited {}.",
                        job.human(),
                        code
                    ),
                    StatusTone::Warning,
                )
            };
            self.set_status(&msg, tone);
        }
        if let Some((job, why)) = errored {
            self.set_buttons_enabled(true);
            self.set_status(
                &format!("Status: ERROR - {} failed. See output.", job.human()),
                StatusTone::Error,
            );
            let _ = why;
        }

        if diagnostics_dirty {
            self.refresh_diagnostics();
        }
        self.refresh_log();
    }

    fn refresh_log(&self) {
        let snapshot = {
            let mut guard = match self.state.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let state = match guard.as_mut() {
                Some(s) => s,
                None => return,
            };
            if !state.log_dirty {
                return;
            }
            state.log_dirty = false;
            state.log_lines.join("\n")
        };
        set_rich_text_box(&self.log_box, &snapshot, COLOR_LOG_TEXT, COLOR_LOG_BG, true);
    }
}

fn run_powershell_job(
    repo_root: PathBuf,
    script_path: PathBuf,
    play_root: Option<PathBuf>,
    tx: Sender<WorkerMessage>,
) {
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path);
    if let Some(ref p) = play_root {
        cmd.arg("-PlayRepoRoot").arg(p);
    }
    cmd.current_dir(&repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(WorkerMessage::Error(format!("spawn failed: {}", e)));
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let tx_out = tx.clone();
    let tx_err = tx.clone();

    let h_out = thread::spawn(move || {
        if let Some(s) = stdout {
            pump_stream(s, tx_out, false);
        }
    });
    let h_err = thread::spawn(move || {
        if let Some(s) = stderr {
            pump_stream(s, tx_err, true);
        }
    });

    let exit = match child.wait() {
        Ok(status) => status.code().unwrap_or(-1),
        Err(e) => {
            let _ = tx.send(WorkerMessage::Error(format!("wait failed: {}", e)));
            return;
        }
    };

    let _ = h_out.join();
    let _ = h_err.join();
    let _ = tx.send(WorkerMessage::Finished(exit));
}

fn pump_stream<R: Read + Send + 'static>(stream: R, tx: Sender<WorkerMessage>, is_err: bool) {
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let raw = match line {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(WorkerMessage::Line(format!("[read error: {}]", e)));
                break;
            }
        };
        if let Some(p) = parse_evidence_dir(&raw) {
            let _ = tx.send(WorkerMessage::EvidenceDir(p));
        }
        let prefix = if is_err { "[err] " } else { "" };
        let _ = tx.send(WorkerMessage::Line(format!("{}{}", prefix, raw)));
    }
}

// Start-TwoClients.ps1 prints lines like `Evidence dir: <path>` on stdout.
// We grep for that marker to surface the run's evidence directory in the UI.
fn parse_evidence_dir(line: &str) -> Option<PathBuf> {
    let needle = "Evidence dir:";
    let idx = line.find(needle)?;
    let tail = line[idx + needle.len()..].trim();
    if tail.is_empty() {
        None
    } else {
        Some(PathBuf::from(tail))
    }
}

// PROMPT 1309: resolves the play/build repo root path (where rebuild + start
// actually run) using the documented priority:
//   1. `CCGS_PLAY_REPO_ROOT` env override.
//   2. `CCGS_CANONICAL_MAIN_ROOT` env alias.
//   3. `D:\_DEV\ccgs-play-main` (documented dedicated default).
// Existence and branch state are reported but never determine the path itself;
// the script handles creation + safety so the launcher does not need to.
fn locate_play_root() -> PlayRootResolution {
    let env_value = env::var(PLAY_REPO_ENV).ok();
    let legacy_env_value = env::var(PLAY_REPO_ENV_ALIAS).ok();
    let default_path = PathBuf::from(PLAY_REPO_DEFAULT);
    resolve_play_root_pure(
        env_value.as_deref(),
        legacy_env_value.as_deref(),
        &default_path,
        |p| p.exists(),
        is_repo_root,
        read_head_branch,
    )
}

// Pure version used by `locate_play_root` and unit tests. `path_exists` is
// kept distinct from `validate_repo` so the test suite can distinguish
// "missing on disk" from "exists but malformed".
fn resolve_play_root_pure<E, V, B>(
    env_play_root: Option<&str>,
    legacy_env: Option<&str>,
    default_path: &Path,
    path_exists: E,
    validate_repo: V,
    read_branch: B,
) -> PlayRootResolution
where
    E: Fn(&Path) -> bool,
    V: Fn(&Path) -> bool,
    B: Fn(&Path) -> Option<String>,
{
    let trim_nonempty = |s: &str| -> Option<String> {
        let t = s.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };

    let (path, source) = if let Some(raw) = env_play_root.and_then(trim_nonempty) {
        (PathBuf::from(raw), PlayRootSource::Env)
    } else if let Some(raw) = legacy_env.and_then(trim_nonempty) {
        (PathBuf::from(raw), PlayRootSource::LegacyEnv)
    } else {
        (default_path.to_path_buf(), PlayRootSource::DedicatedDefault)
    };

    let status = if !path_exists(&path) {
        PlayRootStatus::Missing
    } else if !validate_repo(&path) {
        PlayRootStatus::InvalidRepo(
            "missing Cargo.toml / .git / tools/dev-launcher under the configured path".to_string(),
        )
    } else {
        match read_branch(&path) {
            Some(b) if b == MAIN_BRANCH => PlayRootStatus::OnMain,
            Some(b) => PlayRootStatus::OnOtherBranch(b),
            None => PlayRootStatus::DetachedOrUnknown,
        }
    };

    PlayRootResolution {
        path,
        source,
        status,
    }
}

fn locate_repo_root() -> RepoRootResolution {
    let env_value = env::var("CCGS_REPO_ROOT").ok();
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let cwd = env::current_dir().ok();

    let canonical_override = env::var("CCGS_CANONICAL_REPO_ROOT").ok();
    let canonical_candidates: Vec<PathBuf> = canonical_override
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| vec![PathBuf::from(s)])
        .unwrap_or_else(|| {
            CANONICAL_REPO_CANDIDATES
                .iter()
                .copied()
                .map(PathBuf::from)
                .collect()
        });

    resolve_repo_root_pure(
        env_value.as_deref(),
        exe_dir.as_deref(),
        cwd.as_deref(),
        &canonical_candidates,
        is_repo_root,
        read_head_branch,
        read_sidecar_root,
    )
}

// Pure resolution function used both by `locate_repo_root` and unit tests.
// Validators (`validate`, `read_branch`, `read_sidecar`) are injected so tests
// can supply in-memory fakes without touching the real filesystem.
//
// PROMPT 1290: the sidecar is only accepted when its repo root is on branch
// `main`. If the sidecar points at a valid repo on any other branch (typically
// a worker worktree such as `work/...`), it is treated as unsuitable for the
// `Rebuild Latest Main` flow and we fall through to the canonical-checkout
// fallback list before giving up.
fn resolve_repo_root_pure<F, G, H>(
    env_value: Option<&str>,
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
    canonical_candidates: &[PathBuf],
    validate: F,
    read_branch: H,
    read_sidecar: G,
) -> RepoRootResolution
where
    F: Fn(&Path) -> bool,
    G: Fn(&Path) -> Option<PathBuf>,
    H: Fn(&Path) -> Option<String>,
{
    let mut attempts: Vec<String> = Vec::new();

    match env_value {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                attempts.push("CCGS_REPO_ROOT is set but empty".to_string());
            } else {
                let p = PathBuf::from(trimmed);
                if validate(&p) {
                    return RepoRootResolution::Resolved {
                        root: p,
                        source: ResolutionSource::Env,
                    };
                }
                attempts.push(format!(
                    "CCGS_REPO_ROOT={} -- not a valid repo root",
                    trimmed
                ));
            }
        }
        None => {
            attempts.push("CCGS_REPO_ROOT not set".to_string());
        }
    }

    match exe_dir {
        Some(dir) => {
            match read_sidecar(dir) {
                Some(candidate) => {
                    if validate(&candidate) {
                        let branch = read_branch(&candidate);
                        if branch.as_deref() == Some(MAIN_BRANCH) {
                            return RepoRootResolution::Resolved {
                                root: candidate,
                                source: ResolutionSource::Sidecar,
                            };
                        }
                        let branch_label = branch
                            .clone()
                            .unwrap_or_else(|| "<detached or unknown>".to_string());
                        attempts.push(format!(
                            "sidecar {}\\{} pointed at {} (branch '{}') -- \
                             not on '{}', unsuitable for Rebuild Latest Main; \
                             falling back to canonical checkout",
                            dir.display(),
                            SIDECAR_FILENAME,
                            candidate.display(),
                            branch_label,
                            MAIN_BRANCH,
                        ));
                    } else {
                        attempts.push(format!(
                            "sidecar {}\\{} pointed at {} -- not a valid repo root",
                            dir.display(),
                            SIDECAR_FILENAME,
                            candidate.display()
                        ));
                    }
                }
                None => {
                    attempts.push(format!(
                        "no sidecar at {}\\{}",
                        dir.display(),
                        SIDECAR_FILENAME
                    ));
                }
            }

            if let Some(root) = canonical_lookup(canonical_candidates, &validate, &mut attempts) {
                return RepoRootResolution::Resolved {
                    root,
                    source: ResolutionSource::CanonicalFallback,
                };
            }

            if let Some(root) = walk_up_for_repo_root_with(dir, &validate) {
                return RepoRootResolution::Resolved {
                    root,
                    source: ResolutionSource::ExeWalkUp,
                };
            }
            attempts.push(format!(
                "walk-up from EXE dir {} found no repo root",
                dir.display()
            ));
        }
        None => {
            attempts.push("could not determine EXE directory".to_string());
            if let Some(root) = canonical_lookup(canonical_candidates, &validate, &mut attempts) {
                return RepoRootResolution::Resolved {
                    root,
                    source: ResolutionSource::CanonicalFallback,
                };
            }
        }
    }

    match cwd {
        Some(dir) => {
            if let Some(root) = walk_up_for_repo_root_with(dir, &validate) {
                return RepoRootResolution::Resolved {
                    root,
                    source: ResolutionSource::CwdWalkUp,
                };
            }
            attempts.push(format!(
                "walk-up from cwd {} found no repo root",
                dir.display()
            ));
        }
        None => {
            attempts.push("could not determine current working directory".to_string());
        }
    }

    RepoRootResolution::Failed { attempts }
}

// Iterates `CANONICAL_REPO_CANDIDATES` (or the `CCGS_CANONICAL_REPO_ROOT`
// override). Returns the first candidate that validates; records every
// rejection in `attempts` so the diagnostics panel can show what was tried.
fn canonical_lookup<F: Fn(&Path) -> bool>(
    candidates: &[PathBuf],
    validate: F,
    attempts: &mut Vec<String>,
) -> Option<PathBuf> {
    if candidates.is_empty() {
        attempts.push("canonical-checkout fallback: no candidates configured".to_string());
        return None;
    }
    for candidate in candidates {
        if validate(candidate) {
            return Some(candidate.clone());
        }
        attempts.push(format!(
            "canonical-checkout fallback {} -- not a valid repo root",
            candidate.display()
        ));
    }
    None
}

fn walk_up_for_repo_root_with<F: Fn(&Path) -> bool>(start: &Path, validate: F) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(dir) = current {
        if validate(&dir) {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

fn read_sidecar_root(exe_dir: &Path) -> Option<PathBuf> {
    let sidecar = exe_dir.join(SIDECAR_FILENAME);
    let raw = std::fs::read_to_string(&sidecar).ok()?;
    parse_sidecar_content(&raw)
}

// Sidecar format: the first non-blank, non-comment line is the absolute repo
// root path. Subsequent lines (e.g. additional build-time comments) are
// ignored. Whitespace around the path is trimmed. A leading UTF-8 BOM
// (`\u{FEFF}`, emitted by PowerShell 5.x `Set-Content -Encoding UTF8`) is
// stripped before parsing so a BOM-prefixed comment header is recognised as a
// comment instead of leaking through as the first "path" line. Empty /
// whitespace-only contents return None.
fn parse_sidecar_content(raw: &str) -> Option<PathBuf> {
    let raw = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    for line in raw.lines() {
        let trimmed = line.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}');
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        return Some(PathBuf::from(trimmed));
    }
    None
}

fn is_repo_root(p: &Path) -> bool {
    p.join("Cargo.toml").is_file()
        && p.join("tools").join("dev-launcher").is_dir()
        && p.join(".git").exists()
}

// Reads the symbolic ref from `<repo>/.git/HEAD` and returns the short branch
// name (e.g. "main" or "work/foo"). Returns None on detached HEAD, unreadable
// HEAD file, or non-git path. Handles both regular checkouts (`.git/HEAD`) and
// linked worktrees (`.git` is a file pointing at the per-worktree gitdir).
fn read_head_branch(repo_root: &Path) -> Option<String> {
    let dot_git = repo_root.join(".git");
    let metadata = std::fs::metadata(&dot_git).ok()?;
    let head_path = if metadata.is_dir() {
        dot_git.join("HEAD")
    } else if metadata.is_file() {
        let pointer = std::fs::read_to_string(&dot_git).ok()?;
        let line = pointer.lines().next()?.trim();
        let gitdir = line.strip_prefix("gitdir:")?.trim();
        PathBuf::from(gitdir).join("HEAD")
    } else {
        return None;
    };
    let head = std::fs::read_to_string(&head_path).ok()?;
    let trimmed = head.trim();
    trimmed
        .strip_prefix("ref: refs/heads/")
        .map(|s| s.to_string())
}

fn diagnostics_text(state: &LauncherState) -> String {
    let mut lines = Vec::new();
    match (&state.repo_root, state.repo_source) {
        (Some(root), Some(source)) => {
            lines.push(format!("Launcher repo root: {}", root.display()));
            lines.push(format!("Resolved via: {}", source.human()));
            let branch_label =
                read_head_branch(root).unwrap_or_else(|| "<detached or unknown>".to_string());
            lines.push(format!("Launcher branch: {}", branch_label));
            lines.push(format!(
                "Rebuild script: {}",
                root.join(Path::new(REBUILD_SCRIPT)).display()
            ));
            lines.push(format!(
                "Two-client script: {}",
                root.join(Path::new(LAUNCH_SCRIPT)).display()
            ));
        }
        _ => {
            lines.push("Launcher repo root: UNRESOLVED".to_string());
            lines.push(format!(
                "Expected sidecar: {} beside ccgs-dev-launcher.exe",
                SIDECAR_FILENAME
            ));
            lines.push(
                "Set CCGS_REPO_ROOT to an absolute repo path if the EXE is outside the repo tree."
                    .to_string(),
            );
        }
    }

    match &state.play_root {
        Some(p) => {
            lines.push(format!("Play/build root: {}", p.path.display()));
            lines.push(format!("Play/build source: {}", p.source.human()));
            lines.push(format!("Play/build status: {}", p.status.human()));
            match &p.status {
                PlayRootStatus::OnMain => {
                    lines.push("Play/build branch: main".to_string());
                }
                PlayRootStatus::OnOtherBranch(b) => {
                    lines.push(format!("Play/build branch: {}", b));
                }
                PlayRootStatus::DetachedOrUnknown => {
                    lines.push("Play/build branch: <detached or unknown>".to_string());
                }
                PlayRootStatus::Missing => {
                    lines.push(
                        "Play/build branch: <none -- not yet created>".to_string(),
                    );
                }
                PlayRootStatus::InvalidRepo(_) => {
                    lines.push("Play/build branch: <path is not a CCGS workspace>".to_string());
                }
            }
        }
        None => {
            lines.push("Play/build root: UNRESOLVED".to_string());
        }
    }

    let job = state.job.map(|j| j.human()).unwrap_or("none");
    lines.push(format!("Running job: {}", job));

    match &state.last_evidence_dir {
        Some(path) => lines.push(format!("Evidence: {}", path.display())),
        None => lines.push("Evidence: not yet emitted by the launch script".to_string()),
    }

    match state.last_exit {
        Some(code) => lines.push(format!("Last exit code: {}", code)),
        None => lines.push("Last exit code: none".to_string()),
    }

    if let Some(err) = &state.error_message {
        lines.push(format!("Error detail: {}", err));
    }

    lines.join("\n")
}

fn set_brand_label(label: &nwg::RichLabel) {
    let text = format!("{}\r\n{}", APP_TITLE, APP_SUBTITLE);
    label.set_background_color(COLOR_HEADER_BG);
    label.set_text(&text);

    let title_len = APP_TITLE.chars().count() as u32;
    if title_len > 0 {
        label.set_char_format(
            0..title_len,
            &nwg::CharFormat {
                effects: Some(nwg::CharEffects::BOLD),
                height: Some(420),
                text_color: Some(COLOR_HEADER_TEXT),
                ..Default::default()
            },
        );
    }

    let subtitle_start = title_len + 2;
    let total_len = text.chars().count() as u32;
    if total_len > subtitle_start {
        label.set_char_format(
            subtitle_start..total_len,
            &nwg::CharFormat {
                height: Some(230),
                text_color: Some(COLOR_HEADER_MUTED),
                ..Default::default()
            },
        );
    }
}

fn set_rich_label_text(
    label: &nwg::RichLabel,
    text: &str,
    fg: [u8; 3],
    bg: [u8; 3],
    effects: Option<nwg::CharEffects>,
) {
    label.set_background_color(bg);
    label.set_text(text);
    let len = text.chars().count() as u32;
    if len > 0 {
        label.set_char_format(
            0..len,
            &nwg::CharFormat {
                effects,
                text_color: Some(fg),
                ..Default::default()
            },
        );
    }
}

fn set_rich_text_box(
    box_control: &nwg::RichTextBox,
    text: &str,
    fg: [u8; 3],
    bg: [u8; 3],
    scroll_last: bool,
) {
    box_control.set_background_color(bg);
    box_control.set_text_unix2dos(text);
    let len = box_control.len();
    if len > 0 {
        box_control.set_selection(0..len);
        box_control.set_char_format(&nwg::CharFormat {
            text_color: Some(fg),
            ..Default::default()
        });
        box_control.set_selection(len..len);
    }
    if scroll_last {
        box_control.scroll_lastline();
    }
}

fn build_font(family: &str, size: u32, weight: Option<u32>) -> Option<nwg::Font> {
    let mut font = nwg::Font::default();
    let mut builder = nwg::Font::builder().family(family).size(size);
    if let Some(w) = weight {
        builder = builder.weight(w);
    }
    match builder.build(&mut font) {
        Ok(()) => Some(font),
        Err(e) => {
            eprintln!(
                "[ccgs-dev-launcher] font build failed (family={} size={} weight={:?}): {:?}",
                family, size, weight, e
            );
            None
        }
    }
}

fn apply_font<C: HasFont>(control: &C, font: Option<&nwg::Font>) {
    if let Some(f) = font {
        control.apply_font(f);
    }
}

trait HasFont {
    fn apply_font(&self, font: &nwg::Font);
}

impl HasFont for nwg::RichLabel {
    fn apply_font(&self, font: &nwg::Font) {
        self.set_font(Some(font));
    }
}

impl HasFont for nwg::RichTextBox {
    fn apply_font(&self, font: &nwg::Font) {
        self.set_font(Some(font));
    }
}

impl HasFont for nwg::Button {
    fn apply_font(&self, font: &nwg::Font) {
        self.set_font(Some(font));
    }
}

fn main() {
    let resolution = locate_repo_root();

    nwg::init().expect("Failed to init native-windows-gui");
    if let Some(default_font) = build_font("Segoe UI", 16, None) {
        nwg::Font::set_global_default(Some(default_font));
    }

    let app_template = LauncherUi {
        resolution_init: RefCell::new(Some(resolution)),
        ..Default::default()
    };

    let _ui = LauncherUi::build_ui(app_template).expect("Failed to build LauncherUi");
    nwg::dispatch_thread_events();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = env::temp_dir().join(format!(
            "ccgs-dev-launcher-test-{}-{}-{}-{}",
            label,
            std::process::id(),
            nanos,
            n
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn always_true(_: &Path) -> bool {
        true
    }
    fn always_false(_: &Path) -> bool {
        false
    }
    fn no_sidecar(_: &Path) -> Option<PathBuf> {
        None
    }
    fn branch_main(_: &Path) -> Option<String> {
        Some("main".to_string())
    }
    fn branch_worker(_: &Path) -> Option<String> {
        Some("work/windows-dev-launcher-visual-polish-1255".to_string())
    }
    fn branch_unknown(_: &Path) -> Option<String> {
        None
    }
    fn no_canonical() -> Vec<PathBuf> {
        Vec::new()
    }

    #[test]
    fn parse_evidence_dir_extracts_path() {
        let line = "Evidence dir: D:\\foo\\bar\\2026-05-18-120000";
        assert_eq!(
            parse_evidence_dir(line).unwrap(),
            PathBuf::from("D:\\foo\\bar\\2026-05-18-120000")
        );
    }

    #[test]
    fn parse_evidence_dir_ignores_other_lines() {
        assert!(parse_evidence_dir("Server PID: 1234").is_none());
        assert!(parse_evidence_dir("Evidence dir:").is_none());
        assert!(parse_evidence_dir("").is_none());
    }

    #[test]
    fn parse_evidence_dir_strips_leading_prefix() {
        let line = "    Evidence dir: production/qa/evidence/dev-runs/2026-05-18-120000";
        assert_eq!(
            parse_evidence_dir(line).unwrap(),
            PathBuf::from("production/qa/evidence/dev-runs/2026-05-18-120000")
        );
    }

    #[test]
    fn job_kind_human_labels_match_button_text() {
        assert_eq!(JobKind::Rebuild.human(), REBUILD_BUTTON_LABEL);
        assert_eq!(JobKind::Launch.human(), LAUNCH_BUTTON_LABEL);
    }

    #[test]
    fn job_kind_script_paths_use_dev_launcher_dir() {
        assert!(JobKind::Rebuild.script_rel().contains("dev-launcher"));
        assert!(JobKind::Launch.script_rel().contains("dev-launcher"));
        assert!(JobKind::Rebuild
            .script_rel()
            .ends_with("Update-LatestMain.ps1"));
        assert!(JobKind::Launch
            .script_rel()
            .ends_with("Start-TwoClients.ps1"));
    }

    #[test]
    fn launcher_state_truncates_log_beyond_cap() {
        let mut s = LauncherState::new(Some(PathBuf::from(".")), None, None, None);
        for i in 0..(MAX_LOG_LINES + 50) {
            s.append(format!("line {}", i));
        }
        assert_eq!(s.log_lines.len(), MAX_LOG_LINES);
        assert!(s.log_lines[0].contains("line 50") || s.log_lines[0].contains("line 51"));
    }

    #[test]
    fn evidence_hint_constant_matches_script_output() {
        assert!(EVIDENCE_HINT.contains("dev-runs"));
    }

    #[test]
    fn sidecar_filename_is_documented_value() {
        // The build script and docs both reference this exact filename. If we
        // ever rename it we want a compile-time-visible test failure so the
        // ps1 + docs are updated together.
        assert_eq!(SIDECAR_FILENAME, "ccgs-dev-launcher.repo-root.txt");
    }

    #[test]
    fn parse_sidecar_content_returns_first_nonblank_trimmed_line() {
        let raw = "   D:\\_DEV\\Work\\Claude-Code-Game-Studios   \r\n";
        assert_eq!(
            parse_sidecar_content(raw),
            Some(PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios"))
        );
    }

    #[test]
    fn parse_sidecar_content_skips_blank_and_comment_lines() {
        let raw = "\n# generated by build-launcher-exe.ps1\n\n  D:\\some\\path\n";
        assert_eq!(
            parse_sidecar_content(raw),
            Some(PathBuf::from("D:\\some\\path"))
        );
    }

    #[test]
    fn parse_sidecar_content_rejects_empty_or_blank_only() {
        assert_eq!(parse_sidecar_content(""), None);
        assert_eq!(parse_sidecar_content("   \r\n\t\n"), None);
        assert_eq!(parse_sidecar_content("# only a comment\n"), None);
    }

    #[test]
    fn parse_sidecar_content_skips_bom_prefixed_comment_header() {
        // Regression for the runtime-discovered bug: PowerShell 5.x
        // `Set-Content -Encoding UTF8` writes a UTF-8 BOM (U+FEFF) before the
        // first byte. With a comment as the first body line, the on-disk
        // first line is `\u{FEFF}# ccgs-dev-launcher.repo-root.txt`. Without
        // BOM-aware parsing, `trim().starts_with('#')` fails (U+FEFF is not
        // Unicode White_Space) and the comment leaks through as the resolved
        // "path", which then fails the repo-root validator and the launcher
        // surfaces "launcher script not found ...".
        let raw = "\u{FEFF}# ccgs-dev-launcher.repo-root.txt\r\n\
                   # Generated by tools\\dev-launcher\\build-launcher-exe.ps1\r\n\
                   # Consumed by tools/dev-launcher-app/src/main.rs at startup.\r\n\
                   # Format: first non-blank, non-comment line is the absolute repo root.\r\n\
                   D:\\_DEV\\Work\\Claude-Code-Game-Studios";
        assert_eq!(
            parse_sidecar_content(raw),
            Some(PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios"))
        );
    }

    #[test]
    fn parse_sidecar_content_strips_bom_directly_before_path() {
        // Defensive: if a future writer puts the path on line 1 with no
        // comment header (still UTF-8-BOM-encoded), we must strip the BOM and
        // return the bare path -- not a path that starts with U+FEFF.
        let raw = "\u{FEFF}D:\\_DEV\\Work\\Claude-Code-Game-Studios\r\n";
        assert_eq!(
            parse_sidecar_content(raw),
            Some(PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios"))
        );
    }

    // Hermetic replacement for the former opt-in real-host sidecar check. It
    // still exercises read_sidecar_root against an actual on-disk sidecar file,
    // but the file lives in an isolated temp directory owned by this test.
    #[test]
    fn read_sidecar_root_against_on_disk_file() {
        let dir = unique_temp_dir("real-sidecar");
        let sidecar = dir.join(SIDECAR_FILENAME);
        let expected = dir.join("repo-root");
        fs::write(&sidecar, format!("  {}  \r\n", expected.display())).expect("write sidecar");

        let got = read_sidecar_root(&dir).expect("read_sidecar_root returned None");
        assert_eq!(
            got,
            expected,
            "sidecar at {} resolved to {} but expected {}",
            dir.display(),
            got.display(),
            expected.display(),
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_sidecar_root_handles_utf8_bom_with_comment_header() {
        // End-to-end version of the BOM regression: write the exact byte
        // sequence PowerShell 5.x emits (UTF-8 BOM + comment header + CRLF +
        // repo path) and confirm read_sidecar_root resolves the repo path,
        // not the BOM-decorated comment line.
        let dir = unique_temp_dir("bom-sidecar");
        let sidecar = dir.join(SIDECAR_FILENAME);
        let body = "\u{FEFF}# ccgs-dev-launcher.repo-root.txt\r\n\
                    # Generated by tools\\dev-launcher\\build-launcher-exe.ps1\r\n\
                    # Consumed by tools/dev-launcher-app/src/main.rs at startup.\r\n\
                    # Format: first non-blank, non-comment line is the absolute repo root.\r\n\
                    D:\\_DEV\\Work\\Claude-Code-Game-Studios";
        fs::write(&sidecar, body).expect("write bom sidecar");
        assert_eq!(
            read_sidecar_root(&dir),
            Some(PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios"))
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_sidecar_root_returns_none_when_missing() {
        let dir = unique_temp_dir("missing-sidecar");
        assert_eq!(read_sidecar_root(&dir), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_sidecar_root_returns_path_when_present() {
        let dir = unique_temp_dir("with-sidecar");
        let sidecar = dir.join(SIDECAR_FILENAME);
        fs::write(&sidecar, "D:\\_DEV\\Work\\Claude-Code-Game-Studios\r\n").expect("write sidecar");
        assert_eq!(
            read_sidecar_root(&dir),
            Some(PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios"))
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_sidecar_root_rejects_empty_file() {
        let dir = unique_temp_dir("empty-sidecar");
        let sidecar = dir.join(SIDECAR_FILENAME);
        fs::write(&sidecar, "   \r\n").expect("write empty sidecar");
        assert_eq!(read_sidecar_root(&dir), None);
        let _ = fs::remove_dir_all(&dir);
    }

    fn assert_resolved(
        res: RepoRootResolution,
        expected_root: &Path,
        expected_source: ResolutionSource,
    ) {
        match res {
            RepoRootResolution::Resolved { root, source } => {
                assert_eq!(root, expected_root);
                assert_eq!(source, expected_source);
            }
            RepoRootResolution::Failed { attempts } => {
                panic!("expected Resolved, got Failed: {:?}", attempts)
            }
        }
    }

    #[test]
    fn resolve_repo_root_prefers_env_when_valid() {
        let env_root = PathBuf::from("D:\\repo-from-env");
        let exe_dir = PathBuf::from("D:\\exe");
        let cwd = PathBuf::from("D:\\cwd");

        let res = resolve_repo_root_pure(
            Some("D:\\repo-from-env"),
            Some(&exe_dir),
            Some(&cwd),
            &no_canonical(),
            |p: &Path| p == env_root.as_path(),
            branch_main,
            |dir: &Path| Some(dir.join("sidecar-says-other")),
        );
        assert_resolved(res, &env_root, ResolutionSource::Env);
    }

    #[test]
    fn resolve_repo_root_env_overrides_valid_sidecar_pointing_elsewhere() {
        // PROMPT 1290: even when the sidecar is valid AND on main, the env
        // override must still win. This preserves the documented escape
        // hatch for testers who relocate the EXE.
        let env_root = PathBuf::from("D:\\env-checkout");
        let sidecar_root = PathBuf::from("D:\\sidecar-checkout");
        let env_root_for_closure = env_root.clone();
        let sidecar_root_for_closure = sidecar_root.clone();

        let res = resolve_repo_root_pure(
            Some("D:\\env-checkout"),
            Some(Path::new("D:\\exe")),
            Some(Path::new("D:\\cwd")),
            &no_canonical(),
            move |p: &Path| {
                p == env_root_for_closure.as_path() || p == sidecar_root_for_closure.as_path()
            },
            branch_main,
            move |_dir: &Path| Some(PathBuf::from("D:\\sidecar-checkout")),
        );
        assert_resolved(res, &env_root, ResolutionSource::Env);
    }

    #[test]
    fn resolve_repo_root_falls_through_invalid_env_to_sidecar_on_main() {
        // PROMPT 1290: sidecar is only accepted when its repo is on `main`.
        let sidecar_root = PathBuf::from("D:\\repo-from-sidecar");
        let exe_dir = PathBuf::from("D:\\exe");
        let sidecar_root_for_closure = sidecar_root.clone();

        let res = resolve_repo_root_pure(
            Some("D:\\not-a-repo"),
            Some(&exe_dir),
            Some(Path::new("D:\\cwd")),
            &no_canonical(),
            move |p: &Path| p == sidecar_root_for_closure.as_path(),
            branch_main,
            move |_dir: &Path| Some(PathBuf::from("D:\\repo-from-sidecar")),
        );
        assert_resolved(res, &sidecar_root, ResolutionSource::Sidecar);
    }

    #[test]
    fn resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical() {
        // PROMPT 1290 root cause regression: the sidecar pins the EXE to a
        // worker worktree on branch `work/...`. The launcher must reject the
        // sidecar (unsuitable for Rebuild Latest Main) and use the canonical
        // fallback checkout instead.
        let sidecar_root = PathBuf::from(
            "D:\\_DEV\\claude-code-game-studios-worktrees\\windows-dev-launcher-visual-polish-1255",
        );
        let canonical = PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios");
        let canonical_for_closure = canonical.clone();
        let sidecar_for_closure = sidecar_root.clone();
        let canonicals = vec![canonical.clone()];

        let res = resolve_repo_root_pure(
            None,
            Some(Path::new("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug")),
            Some(Path::new("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug")),
            &canonicals,
            move |p: &Path| {
                p == sidecar_for_closure.as_path() || p == canonical_for_closure.as_path()
            },
            // Sidecar repo is on worker branch; canonical is on main. The
            // branch reader is keyed by path so both repos are distinguishable.
            |p: &Path| {
                if p.as_os_str().to_string_lossy().contains("worktrees") {
                    Some("work/windows-dev-launcher-visual-polish-1255".to_string())
                } else {
                    Some("main".to_string())
                }
            },
            move |_dir: &Path| {
                Some(PathBuf::from(
                    "D:\\_DEV\\claude-code-game-studios-worktrees\\windows-dev-launcher-visual-polish-1255",
                ))
            },
        );
        assert_resolved(res, &canonical, ResolutionSource::CanonicalFallback);
        // Also surface the "unsuitable" note in the attempts list when we
        // fall back, so the diagnostics panel can explain why.
        // (Verified through the Failed branch in the next test.)
    }

    #[test]
    fn resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback() {
        // Positive case: sidecar valid + on main. Should be accepted even
        // when a canonical fallback exists.
        let sidecar_root = PathBuf::from("D:\\sidecar-on-main");
        let canonical = PathBuf::from("D:\\other-canonical");
        let sidecar_for_closure = sidecar_root.clone();
        let canonicals = vec![canonical];

        let res = resolve_repo_root_pure(
            None,
            Some(Path::new("D:\\exe")),
            Some(Path::new("D:\\cwd")),
            &canonicals,
            move |p: &Path| p == sidecar_for_closure.as_path(),
            branch_main,
            move |_dir: &Path| Some(PathBuf::from("D:\\sidecar-on-main")),
        );
        assert_resolved(res, &sidecar_root, ResolutionSource::Sidecar);
    }

    #[test]
    fn resolve_repo_root_invalid_canonical_yields_actionable_error() {
        // PROMPT 1290: sidecar is worker-branched (unsuitable) AND the
        // canonical fallback path does not validate. The launcher must
        // surface a Failed resolution with attempts that mention the
        // canonical fallback was tried; the on-init wiring then renders the
        // actionable CCGS_REPO_ROOT / CCGS_CANONICAL_REPO_ROOT error.
        let sidecar_root =
            PathBuf::from("D:\\_DEV\\claude-code-game-studios-worktrees\\worker-1234");
        let bad_canonical = PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios");
        let sidecar_for_closure = sidecar_root.clone();
        let canonicals = vec![bad_canonical.clone()];

        let res = resolve_repo_root_pure(
            None,
            Some(Path::new("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug")),
            Some(Path::new("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug")),
            &canonicals,
            // Only the sidecar path validates as a repo. Canonical does not.
            move |p: &Path| p == sidecar_for_closure.as_path(),
            branch_worker,
            move |_dir: &Path| {
                Some(PathBuf::from(
                    "D:\\_DEV\\claude-code-game-studios-worktrees\\worker-1234",
                ))
            },
        );
        match res {
            RepoRootResolution::Failed { attempts } => {
                let joined = attempts.join("\n");
                assert!(
                    joined.contains("not on 'main'"),
                    "missing sidecar branch reject note: {}",
                    joined
                );
                assert!(
                    joined.contains("canonical-checkout fallback"),
                    "missing canonical fallback note: {}",
                    joined
                );
                assert!(
                    joined.contains("Claude-Code-Game-Studios"),
                    "missing tried canonical path in attempts: {}",
                    joined
                );
            }
            RepoRootResolution::Resolved { root, source } => panic!(
                "PROMPT 1290 invariant violated: returned Resolved({}, {:?}) \
                 instead of Failed when sidecar is worker-branched and canonical \
                 is invalid",
                root.display(),
                source
            ),
        }
    }

    #[test]
    fn resolve_repo_root_canonical_fallback_records_branch_label_for_unknown_head() {
        // Detached HEAD or missing HEAD file -> branch reader returns None.
        // Treat as not-on-main and fall back to canonical.
        let sidecar_root = PathBuf::from("D:\\detached-sidecar");
        let canonical = PathBuf::from("D:\\canonical-main");
        let sidecar_for_closure = sidecar_root.clone();
        let canonical_for_closure = canonical.clone();
        let canonicals = vec![canonical.clone()];

        let res = resolve_repo_root_pure(
            None,
            Some(Path::new("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug")),
            Some(Path::new("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug")),
            &canonicals,
            move |p: &Path| {
                p == sidecar_for_closure.as_path() || p == canonical_for_closure.as_path()
            },
            branch_unknown,
            move |_dir: &Path| Some(PathBuf::from("D:\\detached-sidecar")),
        );
        assert_resolved(res, &canonical, ResolutionSource::CanonicalFallback);
    }

    #[test]
    fn resolve_repo_root_falls_through_invalid_sidecar_to_exe_walkup() {
        // EXE dir lives outside the repo (mirrors the user's bug: under
        // D:\_DEV\cargo-target\ccgs-msvc\debug). With a malformed sidecar and
        // no canonical match, no valid walk-up from EXE, we should land on
        // the cwd walk-up.
        let exe_dir = PathBuf::from("D:\\cargo-target\\ccgs-msvc\\debug");
        let cwd = PathBuf::from("D:\\some\\subdir\\of\\repo");
        let repo_via_cwd = PathBuf::from("D:\\some\\subdir\\of\\repo");
        let repo_for_closure = repo_via_cwd.clone();

        let res = resolve_repo_root_pure(
            None,
            Some(&exe_dir),
            Some(&cwd),
            &no_canonical(),
            move |p: &Path| p == repo_for_closure.as_path(),
            branch_main,
            |_dir: &Path| Some(PathBuf::from("D:\\bogus-sidecar-target")),
        );
        assert_resolved(res, &repo_via_cwd, ResolutionSource::CwdWalkUp);
    }

    #[test]
    fn resolve_repo_root_uses_exe_walkup_when_sidecar_absent() {
        let exe_dir = PathBuf::from("D:\\repo\\target\\debug");
        let repo = PathBuf::from("D:\\repo");
        let repo_for_closure = repo.clone();

        let res = resolve_repo_root_pure(
            None,
            Some(&exe_dir),
            Some(Path::new("D:\\cwd")),
            &no_canonical(),
            move |p: &Path| p == repo_for_closure.as_path(),
            branch_main,
            no_sidecar,
        );
        assert_resolved(res, &repo, ResolutionSource::ExeWalkUp);
    }

    #[test]
    fn resolve_repo_root_fails_when_nothing_works() {
        // This is exactly the user-reported scenario: EXE lives in
        // D:\_DEV\cargo-target\ccgs-msvc\debug (outside the repo), no env
        // override, no sidecar, no canonical, no walk-up match.
        let exe_dir = PathBuf::from("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug");
        let cwd = PathBuf::from("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug");

        let res = resolve_repo_root_pure(
            None,
            Some(&exe_dir),
            Some(&cwd),
            &no_canonical(),
            always_false,
            branch_main,
            no_sidecar,
        );

        match res {
            RepoRootResolution::Failed { attempts } => {
                assert!(!attempts.is_empty(), "attempts should not be empty");
                let joined = attempts.join("\n");
                assert!(joined.contains("CCGS_REPO_ROOT not set"));
                assert!(joined.contains("no sidecar"));
                assert!(joined.contains("walk-up from EXE dir"));
                assert!(joined.contains("walk-up from cwd"));
            }
            RepoRootResolution::Resolved { root, source } => {
                panic!(
                    "expected Failed for outside-repo EXE with no env/sidecar, \
                     but got Resolved({}, {:?})",
                    root.display(),
                    source
                )
            }
        }
    }

    #[test]
    fn resolve_repo_root_does_not_accept_target_debug_as_root() {
        // Defensive: even if both `exe_dir` and `cwd` point at target/debug
        // (the user-reported bug from PROMPT 1170), and neither walk-up ever
        // visits the real repo root, the function must return Failed -- never
        // silently treat target/debug or any ancestor of it as the repo root.
        let target_debug = PathBuf::from("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug");
        let res = resolve_repo_root_pure(
            None,
            Some(&target_debug),
            Some(&target_debug),
            &no_canonical(),
            // Only the actual repo path (which is NOT on either walk-up
            // chain) is a valid repo root.
            |p: &Path| p == Path::new("D:\\some-other-real-repo"),
            branch_main,
            no_sidecar,
        );
        match res {
            RepoRootResolution::Failed { attempts } => {
                let joined = attempts.join("\n");
                assert!(joined.contains("walk-up from EXE dir"));
                assert!(joined.contains("walk-up from cwd"));
            }
            RepoRootResolution::Resolved { root, source } => panic!(
                "PROMPT 1170 invariant violated: returned Resolved({}, {:?}) \
                 instead of Failed when EXE/cwd are target/debug",
                root.display(),
                source
            ),
        }
    }

    #[test]
    fn resolve_repo_root_handles_empty_env_value() {
        let exe_dir = PathBuf::from("D:\\repo");
        let repo = PathBuf::from("D:\\repo");
        let repo_for_closure = repo.clone();
        let res = resolve_repo_root_pure(
            Some("   "),
            Some(&exe_dir),
            Some(Path::new("D:\\cwd")),
            &no_canonical(),
            move |p: &Path| p == repo_for_closure.as_path(),
            branch_main,
            no_sidecar,
        );
        // Empty/whitespace env should not crash and should fall through to
        // walk-up resolution.
        assert_resolved(res, &repo, ResolutionSource::ExeWalkUp);
    }

    #[test]
    fn resolution_source_human_strings_are_distinct() {
        let all = [
            ResolutionSource::Env.human(),
            ResolutionSource::Sidecar.human(),
            ResolutionSource::CanonicalFallback.human(),
            ResolutionSource::ExeWalkUp.human(),
            ResolutionSource::CwdWalkUp.human(),
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "{} and {} collide", a, b);
                }
            }
        }
    }

    #[test]
    fn always_true_validator_accepts_env_path() {
        let res = resolve_repo_root_pure(
            Some("D:\\anything"),
            Some(Path::new("D:\\exe")),
            Some(Path::new("D:\\cwd")),
            &no_canonical(),
            always_true,
            branch_main,
            no_sidecar,
        );
        assert_resolved(res, &PathBuf::from("D:\\anything"), ResolutionSource::Env);
    }

    #[test]
    fn canonical_repo_candidates_has_at_least_one_entry() {
        // Defensive: the documented default canonical path must be present.
        // If we ever change the constant, the docs (dev-two-button-launcher.md)
        // and build script (build-launcher-exe.ps1) must move in lockstep.
        assert!(
            !CANONICAL_REPO_CANDIDATES.is_empty(),
            "CANONICAL_REPO_CANDIDATES must include at least the documented \
             default (D:\\_DEV\\Work\\Claude-Code-Game-Studios)"
        );
        assert!(CANONICAL_REPO_CANDIDATES
            .iter()
            .any(|p| p.contains("Claude-Code-Game-Studios")));
    }

    #[test]
    fn read_head_branch_returns_main_for_regular_checkout() {
        let dir = unique_temp_dir("head-main");
        let dot_git = dir.join(".git");
        fs::create_dir_all(&dot_git).expect("create .git dir");
        fs::write(dot_git.join("HEAD"), "ref: refs/heads/main\n").expect("write HEAD");
        assert_eq!(read_head_branch(&dir), Some("main".to_string()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_head_branch_returns_worker_branch_name() {
        let dir = unique_temp_dir("head-worker");
        let dot_git = dir.join(".git");
        fs::create_dir_all(&dot_git).expect("create .git dir");
        fs::write(
            dot_git.join("HEAD"),
            "ref: refs/heads/work/windows-dev-launcher-visual-polish-1255\n",
        )
        .expect("write HEAD");
        assert_eq!(
            read_head_branch(&dir),
            Some("work/windows-dev-launcher-visual-polish-1255".to_string())
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_head_branch_returns_none_for_detached_head() {
        let dir = unique_temp_dir("head-detached");
        let dot_git = dir.join(".git");
        fs::create_dir_all(&dot_git).expect("create .git dir");
        // Detached HEAD: file holds a raw 40-char SHA, not a `ref:` line.
        fs::write(
            dot_git.join("HEAD"),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n",
        )
        .expect("write HEAD");
        assert_eq!(read_head_branch(&dir), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_head_branch_follows_worktree_gitdir_pointer() {
        // Linked worktrees write `.git` as a FILE whose content is
        // `gitdir: <path>`. We must follow the pointer to read HEAD.
        let outer = unique_temp_dir("head-worktree-outer");
        let worktree = unique_temp_dir("head-worktree-linked");
        let gitdir = outer.join(".git/worktrees/linked");
        fs::create_dir_all(&gitdir).expect("create linked gitdir");
        fs::write(gitdir.join("HEAD"), "ref: refs/heads/work/example\n")
            .expect("write linked HEAD");
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", gitdir.display()),
        )
        .expect("write .git pointer file");
        assert_eq!(
            read_head_branch(&worktree),
            Some("work/example".to_string())
        );
        let _ = fs::remove_dir_all(&outer);
        let _ = fs::remove_dir_all(&worktree);
    }

    #[test]
    fn launcher_icon_bytes_are_a_real_ico_header() {
        assert!(
            LAUNCHER_ICON_BYTES.len() > 6,
            "icon bytes too short to be an ICO header ({} bytes)",
            LAUNCHER_ICON_BYTES.len()
        );
        assert_eq!(
            &LAUNCHER_ICON_BYTES[0..4],
            &[0x00, 0x00, 0x01, 0x00],
            "first 4 bytes are not the ICONDIR magic"
        );
        let count = u16::from_le_bytes([LAUNCHER_ICON_BYTES[4], LAUNCHER_ICON_BYTES[5]]);
        assert!(count >= 1, "ICO declares zero embedded images");
    }

    #[test]
    fn app_identity_strings_are_distinct_nonempty() {
        assert!(!APP_TITLE.trim().is_empty());
        assert!(!APP_SUBTITLE.trim().is_empty());
        assert_ne!(APP_TITLE, APP_SUBTITLE);
        assert!(APP_SUBTITLE.contains("Windows desktop utility"));
    }

    #[test]
    fn diagnostics_text_surfaces_scrollworthy_paths() {
        let root = PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios");
        let play = PlayRootResolution {
            path: PathBuf::from("D:\\_DEV\\ccgs-play-main"),
            source: PlayRootSource::DedicatedDefault,
            status: PlayRootStatus::Missing,
        };
        let mut state = LauncherState::new(
            Some(root.clone()),
            Some(ResolutionSource::Env),
            Some(play),
            None,
        );
        state.last_evidence_dir =
            Some(root.join("production\\qa\\evidence\\dev-runs\\2026-05-18-120000"));
        let text = diagnostics_text(&state);
        assert!(text.contains("Launcher repo root:"));
        assert!(text.contains(REBUILD_SCRIPT));
        assert!(text.contains(LAUNCH_SCRIPT));
        assert!(text.contains("Evidence:"));
        // PROMPT 1309: both the launcher root AND the play/build root should
        // appear in diagnostics so testers can see they are distinct paths.
        assert!(text.contains("Play/build root:"));
        assert!(text.contains("D:\\_DEV\\ccgs-play-main"));
        assert!(text.contains("Play/build status:"));
    }

    #[test]
    fn status_tone_colors_are_not_flat_defaults() {
        assert_ne!(StatusTone::Idle.colors().0, StatusTone::Running.colors().0);
        assert_ne!(StatusTone::Success.colors().0, StatusTone::Error.colors().0);
        assert_ne!(COLOR_LOG_BG, COLOR_PANEL_BG);
    }

    // ----- PROMPT 1309: play/build dedicated checkout resolution -----

    fn path_never_exists(_: &Path) -> bool {
        false
    }
    fn path_always_exists(_: &Path) -> bool {
        true
    }

    #[test]
    fn play_root_default_constant_is_separate_from_canonical_root() {
        // PROMPT 1309 invariant: the dedicated play/build path MUST be
        // distinct from the orchestrator/canonical checkout. If they collide,
        // a worker-branched orchestrator root would be the rebuild target
        // again -- the exact failure mode this task repairs.
        assert_ne!(PLAY_REPO_DEFAULT, "D:\\_DEV\\Work\\Claude-Code-Game-Studios");
        assert!(
            PLAY_REPO_DEFAULT.contains("ccgs-play"),
            "PLAY_REPO_DEFAULT={} should advertise its dedicated purpose",
            PLAY_REPO_DEFAULT
        );
    }

    #[test]
    fn play_root_default_is_not_inside_worktree_directory() {
        // Defensive: the worker worktree tree at
        // D:\_DEV\claude-code-game-studios-worktrees\ must not collide with
        // the dedicated play checkout. Otherwise a tester juggling worker
        // sessions could be rebuilding inside someone else's branch.
        assert!(!PLAY_REPO_DEFAULT
            .to_ascii_lowercase()
            .contains("claude-code-game-studios-worktrees"));
    }

    #[test]
    fn resolve_play_root_prefers_env_over_legacy_and_default() {
        let res = resolve_play_root_pure(
            Some("D:\\env-play"),
            Some("D:\\legacy-env-play"),
            Path::new("D:\\default-play"),
            path_never_exists,
            always_false,
            branch_main,
        );
        assert_eq!(res.path, PathBuf::from("D:\\env-play"));
        assert_eq!(res.source, PlayRootSource::Env);
        assert_eq!(res.status, PlayRootStatus::Missing);
    }

    #[test]
    fn resolve_play_root_uses_legacy_env_when_primary_unset() {
        let res = resolve_play_root_pure(
            None,
            Some("D:\\legacy-env-play"),
            Path::new("D:\\default-play"),
            path_never_exists,
            always_false,
            branch_main,
        );
        assert_eq!(res.path, PathBuf::from("D:\\legacy-env-play"));
        assert_eq!(res.source, PlayRootSource::LegacyEnv);
    }

    #[test]
    fn resolve_play_root_uses_documented_default_when_no_env() {
        let res = resolve_play_root_pure(
            None,
            None,
            Path::new(PLAY_REPO_DEFAULT),
            path_never_exists,
            always_false,
            branch_main,
        );
        assert_eq!(res.path, PathBuf::from(PLAY_REPO_DEFAULT));
        assert_eq!(res.source, PlayRootSource::DedicatedDefault);
    }

    #[test]
    fn resolve_play_root_treats_empty_or_whitespace_env_as_unset() {
        let res = resolve_play_root_pure(
            Some("   "),
            Some("\t\n"),
            Path::new(PLAY_REPO_DEFAULT),
            path_never_exists,
            always_false,
            branch_main,
        );
        assert_eq!(res.source, PlayRootSource::DedicatedDefault);
    }

    #[test]
    fn resolve_play_root_status_missing_when_path_absent() {
        let res = resolve_play_root_pure(
            None,
            None,
            Path::new(PLAY_REPO_DEFAULT),
            path_never_exists,
            always_true,
            branch_main,
        );
        assert_eq!(res.status, PlayRootStatus::Missing);
        // Status human label should mention the worktree creation intent.
        assert!(res.status.human().contains("worktree"));
    }

    #[test]
    fn resolve_play_root_status_on_main_when_validated_and_main() {
        let res = resolve_play_root_pure(
            None,
            None,
            Path::new(PLAY_REPO_DEFAULT),
            path_always_exists,
            always_true,
            branch_main,
        );
        assert_eq!(res.status, PlayRootStatus::OnMain);
    }

    #[test]
    fn resolve_play_root_status_other_branch_when_worker_checkout() {
        let res = resolve_play_root_pure(
            Some("D:\\some-worker-worktree"),
            None,
            Path::new(PLAY_REPO_DEFAULT),
            path_always_exists,
            always_true,
            branch_worker,
        );
        match res.status {
            PlayRootStatus::OnOtherBranch(b) => {
                assert!(b.starts_with("work/"));
            }
            other => panic!("expected OnOtherBranch, got {:?}", other),
        }
    }

    #[test]
    fn resolve_play_root_status_detached_when_branch_unknown() {
        let res = resolve_play_root_pure(
            None,
            None,
            Path::new(PLAY_REPO_DEFAULT),
            path_always_exists,
            always_true,
            branch_unknown,
        );
        assert_eq!(res.status, PlayRootStatus::DetachedOrUnknown);
    }

    #[test]
    fn resolve_play_root_status_invalid_when_path_exists_but_not_repo() {
        let res = resolve_play_root_pure(
            None,
            None,
            Path::new(PLAY_REPO_DEFAULT),
            path_always_exists,
            always_false,
            branch_main,
        );
        match res.status {
            PlayRootStatus::InvalidRepo(_) => {}
            other => panic!("expected InvalidRepo, got {:?}", other),
        }
    }

    #[test]
    fn play_root_source_human_strings_are_distinct() {
        let all = [
            PlayRootSource::Env.human(),
            PlayRootSource::LegacyEnv.human(),
            PlayRootSource::DedicatedDefault.human(),
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "{} and {} collide", a, b);
                }
            }
        }
    }

    #[test]
    fn play_root_env_constant_names_match_documented_pair() {
        // The build script, docs, and Update-LatestMain.ps1 all reference
        // these env var names. If we rename one, the rest must move together.
        assert_eq!(PLAY_REPO_ENV, "CCGS_PLAY_REPO_ROOT");
        assert_eq!(PLAY_REPO_ENV_ALIAS, "CCGS_CANONICAL_MAIN_ROOT");
    }

    #[test]
    fn diagnostics_text_reports_play_root_status_distinctly_from_launcher() {
        // User-reported scenario regression: launcher resolves to the
        // orchestrator root on a worker branch, the play root is missing.
        // Diagnostics MUST surface both so the user can see the rebuild will
        // target the dedicated path, not the orchestrator checkout.
        let launcher_root = PathBuf::from("D:\\_DEV\\Work\\Claude-Code-Game-Studios");
        let play = PlayRootResolution {
            path: PathBuf::from(PLAY_REPO_DEFAULT),
            source: PlayRootSource::DedicatedDefault,
            status: PlayRootStatus::Missing,
        };
        let state = LauncherState::new(
            Some(launcher_root.clone()),
            Some(ResolutionSource::CanonicalFallback),
            Some(play),
            None,
        );
        let text = diagnostics_text(&state);
        assert!(text.contains("Launcher repo root: D:\\_DEV\\Work\\Claude-Code-Game-Studios"));
        assert!(text.contains("Play/build root: D:\\_DEV\\ccgs-play-main"));
        assert!(text.contains("Play/build source: documented dedicated default"));
        assert!(
            text.contains("not yet created"),
            "diagnostics should call out that the play root will be created -- text was:\n{}",
            text
        );
    }

    #[test]
    fn diagnostics_text_shows_play_branch_when_play_root_on_other_branch() {
        // If the dedicated checkout exists but somehow ended up on a non-main
        // branch (e.g. tester left it there), diagnostics must surface that
        // explicitly so the rebuild outcome is predictable.
        let play = PlayRootResolution {
            path: PathBuf::from(PLAY_REPO_DEFAULT),
            source: PlayRootSource::Env,
            status: PlayRootStatus::OnOtherBranch("work/foo".to_string()),
        };
        let state = LauncherState::new(
            Some(PathBuf::from("D:\\launcher")),
            Some(ResolutionSource::Env),
            Some(play),
            None,
        );
        let text = diagnostics_text(&state);
        assert!(
            text.contains("Play/build branch: work/foo"),
            "missing play branch in diagnostics:\n{}",
            text
        );
        assert!(text.contains("CCGS_PLAY_REPO_ROOT"));
    }

    #[test]
    fn play_root_status_human_labels_are_actionable() {
        // The diagnostics panel surfaces these labels verbatim. They must
        // tell the user WHAT the launcher will do next.
        assert!(PlayRootStatus::OnMain.human().contains("main"));
        assert!(PlayRootStatus::Missing.human().contains("created"));
        let other = PlayRootStatus::OnOtherBranch("work/foo".to_string());
        assert!(other.human().contains("work/foo"));
        let invalid = PlayRootStatus::InvalidRepo("X".to_string());
        assert!(invalid.human().contains("not a CCGS workspace"));
    }
}
