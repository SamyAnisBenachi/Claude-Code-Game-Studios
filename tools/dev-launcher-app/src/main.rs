// CCGS Dev Launcher -- two-button Windows EXE wrapping the dev-launcher
// PowerShell scripts (`tools/dev-launcher/Update-LatestMain.ps1` and
// `tools/dev-launcher/Start-TwoClients.ps1`).
//
// Button 1 -- "Rebuild Latest Main" -- invokes Update-LatestMain.ps1.
// Button 2 -- "Start Two-Client Play Session" -- invokes Start-TwoClients.ps1.
//
// The launcher is intentionally a thin wrapper: it does not duplicate any
// launcher logic. Repo root is resolved from the EXE working directory or
// from the `CCGS_REPO_ROOT` env override. The scripts themselves remain the
// source of truth for cargo policy, evidence dir naming, port selection,
// process spawning, and safety guards.

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

struct LauncherState {
    repo_root: PathBuf,
    job: Option<JobKind>,
    rx: Option<Receiver<WorkerMessage>>,
    last_exit: Option<i32>,
    last_evidence_dir: Option<PathBuf>,
    log_lines: Vec<String>,
    log_dirty: bool,
}

impl LauncherState {
    fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
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

    repo_root_init: RefCell<Option<PathBuf>>,
}

impl LauncherUi {
    fn set_buttons_enabled(&self, enabled: bool) {
        self.rebuild_btn.set_enabled(enabled);
        self.launch_btn.set_enabled(enabled);
    }

    fn on_init(&self) {
        // The state Arc is constructed empty by Default; populate it now that
        // we have resolved the repo root.
        let root = self
            .repo_root_init
            .borrow_mut()
            .take()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let mut guard = self.state.lock().expect("state poisoned at init");
        *guard = Some(LauncherState::new(root));
        if let Some(s) = guard.as_mut() {
            s.append(format!("Repo root: {}", s.repo_root.display()));
            s.append(format!(
                "Scripts: {} | {}",
                REBUILD_SCRIPT, LAUNCH_SCRIPT
            ));
            s.append(
                "Click 'Rebuild Latest Main' first if you just pulled, then \
                 'Start Two-Client Play Session' to launch one server + two clients."
                    .to_string(),
            );
            s.log_dirty = true;
        }
        // Flush initial log paint without waiting for the first timer tick.
        drop(guard);
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
        let repo_root = state.repo_root.clone();
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

fn locate_repo_root() -> PathBuf {
    if let Ok(env_root) = env::var("CCGS_REPO_ROOT") {
        let p = PathBuf::from(env_root);
        if is_repo_root(&p) {
            return p;
        }
    }

    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if let Some(dir) = exe_dir {
        if let Some(root) = walk_up_for_repo_root(&dir) {
            return root;
        }
    }

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(root) = walk_up_for_repo_root(&cwd) {
        return root;
    }

    cwd
}

fn walk_up_for_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(dir) = current {
        if is_repo_root(&dir) {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

fn is_repo_root(p: &Path) -> bool {
    p.join("Cargo.toml").is_file()
        && p.join("tools").join("dev-launcher").is_dir()
        && p.join(".git").exists()
}

fn main() {
    let repo_root = locate_repo_root();

    nwg::init().expect("Failed to init native-windows-gui");
    let mut default_font = nwg::Font::default();
    let _ = nwg::Font::builder()
        .family("Segoe UI")
        .size(16)
        .build(&mut default_font);
    nwg::Font::set_global_default(Some(default_font));

    let app_template = LauncherUi {
        repo_root_init: RefCell::new(Some(repo_root)),
        ..Default::default()
    };

    let _ui = LauncherUi::build_ui(app_template).expect("Failed to build LauncherUi");
    nwg::dispatch_thread_events();
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Useful when stderr/stdout interleaves with `[err] ` prefixes after
        // we add them, but the raw line from PowerShell is what we parse first.
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
        let mut s = LauncherState::new(PathBuf::from("."));
        for i in 0..(MAX_LOG_LINES + 50) {
            s.append(format!("line {}", i));
        }
        assert_eq!(s.log_lines.len(), MAX_LOG_LINES);
        // Oldest lines should have been dropped.
        assert!(s.log_lines[0].contains("line 50") || s.log_lines[0].contains("line 51"));
    }

    #[test]
    fn evidence_hint_constant_matches_script_output() {
        // Start-TwoClients.ps1 writes evidence under this prefix.
        assert!(EVIDENCE_HINT.contains("dev-runs"));
    }
}
