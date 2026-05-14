//! Role-aware tracing log routing for the two-client runtime harness.
//!
//! All three Bevy `App`s (server, client A, client B) tick on the same OS
//! thread sequentially. Before each `App::update()`, the driver sets the
//! thread-local `CURRENT_ROLE`; the [`RoleWriter`] consults it to route every
//! tracing emission written during that tick to the right per-role file.
//!
//! The harness driver itself also emits tracing events (CLI parsing, route
//! orchestration, evidence writing) — those route to `harness.log`.
//!
//! AC5 maps to the four file writers; AC6 (ISO-8601 UTC ms precision) maps
//! to the `with_timer(UtcTime::rfc_3339())` configuration applied in
//! [`init_role_subscriber`].

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Role {
    Harness = 0,
    Server = 1,
    ClientA = 2,
    ClientB = 3,
}

impl Role {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => Role::Server,
            2 => Role::ClientA,
            3 => Role::ClientB,
            _ => Role::Harness,
        }
    }
}

// Global atomic instead of thread-local so role routing survives Bevy's
// parallel system executor (MinimalPlugins + multi_threaded feature spawns
// worker threads that don't inherit the main thread's TLS). The harness
// driver ticks server / client A / client B sequentially on the main thread
// (no overlap), so a single shared atomic is sufficient and races are
// impossible by construction.
static CURRENT_ROLE: AtomicU8 = AtomicU8::new(0);

pub fn set_role(role: Role) {
    CURRENT_ROLE.store(role as u8, Ordering::SeqCst);
}

pub fn current_role() -> Role {
    Role::from_u8(CURRENT_ROLE.load(Ordering::SeqCst))
}

#[derive(Clone)]
pub struct RoleLogPaths {
    pub harness: PathBuf,
    pub server: PathBuf,
    pub client_a: PathBuf,
    pub client_b: PathBuf,
}

impl RoleLogPaths {
    pub fn under(dir: &Path) -> Self {
        Self {
            harness: dir.join("harness.log"),
            server: dir.join("server.log"),
            client_a: dir.join("client_a.log"),
            client_b: dir.join("client_b.log"),
        }
    }
}

#[derive(Clone)]
struct SharedFile(Arc<Mutex<File>>);

impl SharedFile {
    fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        Ok(Self(Arc::new(Mutex::new(file))))
    }
}

pub struct RoleWriterHandle<'a> {
    inner: std::sync::MutexGuard<'a, File>,
}

impl Write for RoleWriterHandle<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Clone)]
pub struct RoleWriter {
    harness: SharedFile,
    server: SharedFile,
    client_a: SharedFile,
    client_b: SharedFile,
}

impl RoleWriter {
    pub fn open(paths: &RoleLogPaths) -> io::Result<Self> {
        Ok(Self {
            harness: SharedFile::open(&paths.harness)?,
            server: SharedFile::open(&paths.server)?,
            client_a: SharedFile::open(&paths.client_a)?,
            client_b: SharedFile::open(&paths.client_b)?,
        })
    }

    fn pick(&self, role: Role) -> &SharedFile {
        match role {
            Role::Harness => &self.harness,
            Role::Server => &self.server,
            Role::ClientA => &self.client_a,
            Role::ClientB => &self.client_b,
        }
    }
}

impl<'a> MakeWriter<'a> for RoleWriter {
    type Writer = RoleWriterHandle<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        let target = self.pick(current_role());
        RoleWriterHandle {
            inner: target
                .0
                .lock()
                .expect("role log file mutex must not be poisoned during tracing emit"),
        }
    }
}

/// Installs the harness's global tracing subscriber. UTC ISO-8601 timestamps
/// at millisecond precision satisfy AC6. The subscriber is intentionally
/// `try_init` so that re-runs of `cargo test` against this crate do not panic
/// on duplicate subscriber installation (the harness binary itself only
/// initialises once per process).
pub fn init_role_subscriber(paths: &RoleLogPaths) -> io::Result<RoleWriter> {
    let writer = RoleWriter::open(paths)?;
    let writer_clone = writer.clone();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(
            "info,wgpu=warn,wgpu_hal=warn,naga=warn,bevy_ecs=info,lightyear=info",
        )
    });
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer_clone)
        .with_timer(UtcTime::rfc_3339())
        .with_target(true)
        .with_ansi(false)
        .try_init();
    Ok(writer)
}
