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

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![cfg(windows)]

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

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

const REBUILD_BUTTON_LABEL: &str = "Rebuild Latest Main";
const LAUNCH_BUTTON_LABEL: &str = "Start Two-Client Play Session";
const REBUILD_SCRIPT: &str = "tools\\dev-launcher\\Update-LatestMain.ps1";
const LAUNCH_SCRIPT: &str = "tools\\dev-launcher\\Start-TwoClients.ps1";
// Sidecar written next to the EXE by `tools/dev-launcher/build-launcher-exe.ps1`.
// Contains the absolute repo root path on the first non-blank line.
const SIDECAR_FILENAME: &str = "ccgs-dev-launcher.repo-root.txt";
// Start-TwoClients.ps1 writes per-run logs under this path; the launcher
// surfaces the exact directory parsed from script stdout.
#[cfg(test)]
const EVIDENCE_HINT: &str = "production/qa/evidence/dev-runs/";
const MAX_LOG_LINES: usize = 2000;
const TIMER_INTERVAL_MS: u64 = 150;

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
    ExeWalkUp,
    CwdWalkUp,
}

impl ResolutionSource {
    fn human(self) -> &'static str {
        match self {
            ResolutionSource::Env => "CCGS_REPO_ROOT env var",
            ResolutionSource::Sidecar => "sidecar file beside EXE",
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

struct LauncherState {
    repo_root: Option<PathBuf>,
    error_message: Option<String>,
    job: Option<JobKind>,
    rx: Option<Receiver<WorkerMessage>>,
    last_exit: Option<i32>,
    last_evidence_dir: Option<PathBuf>,
    log_lines: Vec<String>,
    log_dirty: bool,
}

impl LauncherState {
    fn new(repo_root: Option<PathBuf>, error_message: Option<String>) -> Self {
        Self {
            repo_root,
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

#[derive(Default, NwgUi)]
pub struct LauncherUi {
    #[nwg_control(size: (820, 560), position: (260, 180), title: "CCGS Dev Launcher", flags: "WINDOW|VISIBLE|MAIN_WINDOW")]
    #[nwg_events(OnWindowClose: [LauncherUi::on_close], OnInit: [LauncherUi::on_init])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 6)]
    layout: nwg::GridLayout,

    #[nwg_control(text: "Idle. Choose a button to begin.")]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, col_span: 6)]
    state_label: nwg::Label,

    #[nwg_control(text: REBUILD_BUTTON_LABEL)]
    #[nwg_events(OnButtonClick: [LauncherUi::on_rebuild])]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 3)]
    rebuild_btn: nwg::Button,

    #[nwg_control(text: LAUNCH_BUTTON_LABEL)]
    #[nwg_events(OnButtonClick: [LauncherUi::on_launch])]
    #[nwg_layout_item(layout: layout, col: 3, row: 1, col_span: 3)]
    launch_btn: nwg::Button,

    #[nwg_control(text: "Logs / evidence: (not yet)")]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 6)]
    evidence_label: nwg::Label,

    #[nwg_control(text: "", flags: "VISIBLE|VSCROLL|AUTOVSCROLL")]
    #[nwg_layout_item(layout: layout, col: 0, row: 3, col_span: 6, row_span: 9)]
    log_box: nwg::TextBox,

    #[nwg_control(interval: Duration::from_millis(TIMER_INTERVAL_MS), active: true)]
    #[nwg_events(OnTimerTick: [LauncherUi::on_tick])]
    timer: nwg::AnimationTimer,

    state: Arc<Mutex<Option<LauncherState>>>,

    resolution_init: RefCell<Option<RepoRootResolution>>,
}

impl LauncherUi {
    fn set_buttons_enabled(&self, enabled: bool) {
        self.rebuild_btn.set_enabled(enabled);
        self.launch_btn.set_enabled(enabled);
    }

    fn on_init(&self) {
        let resolution = self
            .resolution_init
            .borrow_mut()
            .take()
            .unwrap_or_else(|| RepoRootResolution::Failed {
                attempts: vec!["resolution missing at UI init".to_string()],
            });

        let (repo_root, error_message, init_lines, state_label_text, buttons_enabled) =
            match resolution {
                RepoRootResolution::Resolved { root, source } => {
                    let mut lines = Vec::new();
                    lines.push(format!(
                        "Repo root: {} (via {})",
                        root.display(),
                        source.human()
                    ));
                    lines.push(format!(
                        "Scripts: {} | {}",
                        REBUILD_SCRIPT, LAUNCH_SCRIPT
                    ));
                    lines.push(
                        "Click 'Rebuild Latest Main' first if you just pulled, then \
                         'Start Two-Client Play Session' to launch one server + two clients."
                            .to_string(),
                    );
                    (
                        Some(root),
                        None,
                        lines,
                        "Idle. Choose a button to begin.".to_string(),
                        true,
                    )
                }
                RepoRootResolution::Failed { attempts } => {
                    let err = format!(
                        "ERROR: could not locate CCGS repo root. \
                         Set CCGS_REPO_ROOT or rebuild via tools\\dev-launcher\\build-launcher-exe.ps1 \
                         (writes the {} sidecar beside the EXE).",
                        SIDECAR_FILENAME
                    );
                    let mut lines = vec![err.clone(), "Attempts:".to_string()];
                    for a in &attempts {
                        lines.push(format!("  - {}", a));
                    }
                    lines.push(
                        "Buttons are disabled until a valid repo root is resolved.".to_string(),
                    );
                    (
                        None,
                        Some(err.clone()),
                        lines,
                        err,
                        false,
                    )
                }
            };

        let mut guard = self.state.lock().expect("state poisoned at init");
        let mut state = LauncherState::new(repo_root, error_message);
        for line in init_lines {
            state.append(line);
        }
        state.log_dirty = true;
        *guard = Some(state);
        drop(guard);

        self.state_label.set_text(&state_label_text);
        self.set_buttons_enabled(buttons_enabled);
        // Flush initial log paint without waiting for the first timer tick.
        self.refresh_log();
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
            self.refresh_log();
            return;
        }

        state.last_exit = None;
        state.last_evidence_dir = None;
        state.add_banner(&format!("STARTING: {}", job.human()));
        state.append(format!("Script: {}", script_path.display()));

        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();
        let repo_root_clone = repo_root.clone();
        let script_path_clone = script_path.clone();
        thread::spawn(move || run_powershell_job(repo_root_clone, script_path_clone, tx_clone));

        state.job = Some(job);
        state.rx = Some(rx);
        drop(guard);

        self.set_buttons_enabled(false);
        self.state_label
            .set_text(&format!("RUNNING: {}", job.human()));
        self.refresh_log();
    }

    fn on_tick(&self) {
        let mut finished: Option<(JobKind, i32)> = None;
        let mut errored: Option<(JobKind, String)> = None;
        let mut new_evidence: Option<PathBuf> = None;
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
                            new_evidence = Some(p);
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
                            }
                            keep_rx = false;
                            break;
                        }
                        Ok(WorkerMessage::Error(msg)) => {
                            if let Some(job) = state.job.take() {
                                state.add_banner(&format!("ERROR: {} -- {}", job.human(), msg));
                                errored = Some((job, msg));
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

        if let Some(p) = new_evidence {
            self.evidence_label
                .set_text(&format!("Logs / evidence: {}", p.display()));
        }
        if let Some((job, code)) = finished {
            self.set_buttons_enabled(true);
            let msg = if code == 0 {
                format!("DONE: {} (exit 0)", job.human())
            } else {
                format!("DONE WITH ERRORS: {} (exit {})", job.human(), code)
            };
            self.state_label.set_text(&msg);
        }
        if let Some((job, why)) = errored {
            self.set_buttons_enabled(true);
            self.state_label
                .set_text(&format!("ERROR: {} -- {}", job.human(), why));
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
            state.log_lines.join("\r\n")
        };
        self.log_box.set_text(&snapshot);
    }
}

fn run_powershell_job(repo_root: PathBuf, script_path: PathBuf, tx: Sender<WorkerMessage>) {
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(&script_path)
        .current_dir(&repo_root)
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

fn locate_repo_root() -> RepoRootResolution {
    let env_value = env::var("CCGS_REPO_ROOT").ok();
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let cwd = env::current_dir().ok();

    resolve_repo_root_pure(
        env_value.as_deref(),
        exe_dir.as_deref(),
        cwd.as_deref(),
        is_repo_root,
        read_sidecar_root,
    )
}

// Pure resolution function used both by `locate_repo_root` and unit tests.
// Validators (`validate`, `read_sidecar`) are injected so tests can supply
// in-memory fakes without touching the real filesystem.
fn resolve_repo_root_pure<F, G>(
    env_value: Option<&str>,
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
    validate: F,
    read_sidecar: G,
) -> RepoRootResolution
where
    F: Fn(&Path) -> bool,
    G: Fn(&Path) -> Option<PathBuf>,
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
                        return RepoRootResolution::Resolved {
                            root: candidate,
                            source: ResolutionSource::Sidecar,
                        };
                    }
                    attempts.push(format!(
                        "sidecar {}\\{} pointed at {} -- not a valid repo root",
                        dir.display(),
                        SIDECAR_FILENAME,
                        candidate.display()
                    ));
                }
                None => {
                    attempts.push(format!(
                        "no sidecar at {}\\{}",
                        dir.display(),
                        SIDECAR_FILENAME
                    ));
                }
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

fn main() {
    let resolution = locate_repo_root();

    nwg::init().expect("Failed to init native-windows-gui");
    let mut default_font = nwg::Font::default();
    let _ = nwg::Font::builder()
        .family("Segoe UI")
        .size(16)
        .build(&mut default_font);
    nwg::Font::set_global_default(Some(default_font));

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
        let mut s = LauncherState::new(Some(PathBuf::from(".")), None);
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

    // Opt-in integration check: read a real sidecar file from disk (typically
    // the BOM-encoded one left by a previous PROMPT 1170 build script run)
    // and assert that the fixed parser still resolves it to the expected repo
    // root. Skipped by default because it depends on machine-local paths; set
    // both env vars to enable:
    //
    //   CCGS_TEST_REAL_SIDECAR_DIR  -- absolute dir that contains the sidecar
    //   CCGS_TEST_REAL_SIDECAR_ROOT -- expected absolute repo root path
    //
    // Run with: cargo test -p dev-launcher-app -- --ignored
    #[test]
    #[ignore]
    fn read_sidecar_root_against_real_on_disk_file_opt_in() {
        let dir = match env::var("CCGS_TEST_REAL_SIDECAR_DIR") {
            Ok(v) => PathBuf::from(v),
            Err(_) => return,
        };
        let expected = match env::var("CCGS_TEST_REAL_SIDECAR_ROOT") {
            Ok(v) => PathBuf::from(v),
            Err(_) => return,
        };
        let got = read_sidecar_root(&dir).expect("read_sidecar_root returned None");
        assert_eq!(
            got, expected,
            "real sidecar at {} resolved to {} but expected {}",
            dir.display(),
            got.display(),
            expected.display(),
        );
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
        fs::write(&sidecar, "D:\\_DEV\\Work\\Claude-Code-Game-Studios\r\n")
            .expect("write sidecar");
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
            |p: &Path| p == env_root.as_path(),
            |dir: &Path| Some(dir.join("sidecar-says-other")),
        );
        assert_resolved(res, &env_root, ResolutionSource::Env);
    }

    #[test]
    fn resolve_repo_root_falls_through_invalid_env_to_sidecar() {
        let sidecar_root = PathBuf::from("D:\\repo-from-sidecar");
        let exe_dir = PathBuf::from("D:\\exe");
        let sidecar_root_for_closure = sidecar_root.clone();

        let res = resolve_repo_root_pure(
            Some("D:\\not-a-repo"),
            Some(&exe_dir),
            Some(Path::new("D:\\cwd")),
            move |p: &Path| p == sidecar_root_for_closure.as_path(),
            move |_dir: &Path| Some(PathBuf::from("D:\\repo-from-sidecar")),
        );
        assert_resolved(res, &sidecar_root, ResolutionSource::Sidecar);
    }

    #[test]
    fn resolve_repo_root_falls_through_invalid_sidecar_to_exe_walkup() {
        // EXE dir lives outside the repo (mirrors the user's bug: under
        // D:\_DEV\cargo-target\ccgs-msvc\debug). With a malformed sidecar and
        // no valid walk-up from EXE, we should land on the cwd walk-up.
        let exe_dir = PathBuf::from("D:\\cargo-target\\ccgs-msvc\\debug");
        let cwd = PathBuf::from("D:\\some\\subdir\\of\\repo");
        let repo_via_cwd = PathBuf::from("D:\\some\\subdir\\of\\repo");
        let repo_for_closure = repo_via_cwd.clone();

        let res = resolve_repo_root_pure(
            None,
            Some(&exe_dir),
            Some(&cwd),
            move |p: &Path| p == repo_for_closure.as_path(),
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
            move |p: &Path| p == repo_for_closure.as_path(),
            no_sidecar,
        );
        assert_resolved(res, &repo, ResolutionSource::ExeWalkUp);
    }

    #[test]
    fn resolve_repo_root_fails_when_nothing_works() {
        // This is exactly the user-reported scenario: EXE lives in
        // D:\_DEV\cargo-target\ccgs-msvc\debug (outside the repo), no env
        // override, no sidecar, no walk-up match from EXE or cwd.
        let exe_dir = PathBuf::from("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug");
        let cwd = PathBuf::from("D:\\_DEV\\cargo-target\\ccgs-msvc\\debug");

        let res = resolve_repo_root_pure(
            None,
            Some(&exe_dir),
            Some(&cwd),
            always_false,
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
            // Only the actual repo path (which is NOT on either walk-up
            // chain) is a valid repo root.
            |p: &Path| p == Path::new("D:\\some-other-real-repo"),
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
            move |p: &Path| p == repo_for_closure.as_path(),
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
            always_true,
            no_sidecar,
        );
        assert_resolved(
            res,
            &PathBuf::from("D:\\anything"),
            ResolutionSource::Env,
        );
    }
}
