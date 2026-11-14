//! Structured tracing initialization for mesh observability
//!
//! Configures a non-blocking JSON file appender for the core mesh router.
//! The log directory is provided by the host OS via FFI to respect mobile sandboxing.

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize file-based structured tracing with JSON output.
///
/// CRITICAL: Uses `.try_init()` to survive mobile app warm boots where the
/// MeshService is recreated without killing the process. Using `.init()` would
/// panic on the second initialization attempt.
///
/// # Arguments
/// * `log_directory` - Absolute path to log directory from host OS (e.g., iOS Documents, Android filesDir)
///
/// # Output
/// Writes to `<log_directory>/scmessenger-mesh.log` with structured JSON lines:
/// ```json
/// {"timestamp":"2026-03-24T23:00:00Z","level":"INFO","message_id":"abc-123","event":"outbox_enqueue",...}
/// ```
/// Name of the mesh log file inside the host-provided log directory.
#[cfg(not(target_arch = "wasm32"))]
pub const MESH_LOG_FILENAME: &str = "scmessenger-mesh.log";

/// Size at which the mesh log is rotated at startup (64 MiB).
///
/// Chosen against the observed damage, not a round number: #679 measured 4.19 GB
/// (~15.2 M lines over 30 days, ~140 MB/day at the default `info` filter). 64 MiB
/// is roughly half a day of sustained logging at that rate, so a runaway writer
/// still leaves the current session's evidence on disk while the file cannot
/// accumulate across months. Two files exist at most (`.log` and `.log.1`).
#[cfg(not(target_arch = "wasm32"))]
pub const MESH_LOG_CAP_BYTES: u64 = 64 * 1024 * 1024;

/// Rotate `path` to `<path>.1` if it exceeds `cap_bytes`.
///
/// Returns whether a rotation happened. A missing file is not an error: the
/// first run on a fresh install has nothing to rotate. Any previous `.1` is
/// replaced, so the pair of files is bounded at `2 * cap_bytes` rather than
/// unbounded across restarts.
#[cfg(not(target_arch = "wasm32"))]
fn rotate_oversized_log(path: &Path, cap_bytes: u64) -> std::io::Result<bool> {
    let len = match std::fs::metadata(path) {
        Ok(md) => md.len(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    if len <= cap_bytes {
        return Ok(false);
    }
    let rotated = Path::new(&format!("{}.1", path.display())).to_path_buf();
    if rotated.exists() {
        std::fs::remove_file(&rotated)?;
    }
    std::fs::rename(path, &rotated)?;
    Ok(true)
}

/// A `std::io::Write` sink that keeps the live mesh log under `cap_bytes` for the
/// whole life of the process, not only at start-up.
///
/// `tracing_appender::rolling::never` has no size policy: once opened, nothing in
/// the tree checks the file's length. The start-up rotation in
/// `init_file_tracing` therefore bounds the file across restarts but not within a
/// session, and the node that does not restart is exactly the one at risk -- the
/// always-on cloud node lives for weeks. #679 measured that same failure at
/// 4.19 GB. This sink counts the bytes it writes and rotates at the cap, so the
/// bound holds within a session as well as across restarts.
///
/// Rotation keeps the two-file shape of `rotate_oversized_log`: the live file
/// becomes `<path>.1` and a fresh live file is opened, so the pair stays inside
/// `2 * cap_bytes`. The rotation is recorded as one JSON line written straight to
/// the new file rather than through `tracing` -- logging from inside the sink the
/// appender is calling would re-enter that appender.
#[cfg(not(target_arch = "wasm32"))]
struct CappedFileWriter {
    path: std::path::PathBuf,
    file: Option<std::fs::File>,
    written: u64,
    cap_bytes: u64,
    rotations: u64,
}

#[cfg(not(target_arch = "wasm32"))]
impl CappedFileWriter {
    fn open(path: &Path, cap_bytes: u64) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        // Count what is already on disk, so a file the start-up rotation left in
        // place is bounded by the same cap rather than starting the count at zero.
        let written = file.metadata().map(|md| md.len()).unwrap_or(0);
        Ok(Self {
            path: path.to_path_buf(),
            file: Some(file),
            written,
            cap_bytes,
            rotations: 0,
        })
    }

    /// The open sink, reopening the live file if a previous rotation left none.
    fn sink(&mut self) -> std::io::Result<&mut std::fs::File> {
        if self.file.is_none() {
            self.file = Some(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?,
            );
        }
        Ok(self.file.as_mut().expect("sink was just opened"))
    }

    /// Replace the live file with `<path>.1` and reopen an empty one.
    ///
    /// The handle is dropped before the rename because Windows cannot rename an
    /// open file. If rotation is impossible (read-only directory, a `.1` that
    /// cannot be replaced) the count restarts at zero and the sink degrades to the
    /// unbounded append it replaces, instead of attempting a rename per log line.
    fn rotate(&mut self) -> std::io::Result<()> {
        self.file = None;
        let rotated = std::path::PathBuf::from(format!("{}.1", self.path.display()));
        if rotated.exists() {
            let _ = std::fs::remove_file(&rotated);
        }
        let renamed = std::fs::rename(&self.path, &rotated).is_ok();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.written = 0;
        if renamed {
            self.rotations += 1;
            let marker = format!(
                "{{\"level\":\"WARN\",\"event\":\"mesh_log_rotated\",\"cap_bytes\":{},\"rotation\":{},\"rotated_to\":\"{}\"}}\n",
                self.cap_bytes,
                self.rotations,
                rotated.display()
            );
            // Direct write, not `tracing::warn!`: this runs inside the appender's
            // own sink, and re-entering it would deadlock the non-blocking writer.
            std::io::Write::write_all(&mut file, marker.as_bytes())?;
            self.written = marker.len() as u64;
        }
        self.file = Some(file);
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::io::Write for CappedFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Rotate before the write that would cross the cap, so the file that ends
        // up as `.1` never exceeds the cap by more than the line that crossed it.
        if self.written + buf.len() as u64 > self.cap_bytes {
            self.rotate()?;
        }
        let written = std::io::Write::write(self.sink()?, buf)?;
        self.written += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.file.as_mut() {
            Some(file) => std::io::Write::flush(file),
            None => Ok(()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_file_tracing(log_directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = Path::new(log_directory);

    // Ensure the directory exists (critical for mobile sandboxing)
    std::fs::create_dir_all(log_path)?;

    // Bound the file's growth before opening it. #679 measured a 4.19 GB mesh log
    // on the Pixel 6a: a stuck message in an unbounded retry cycle wrote for 30
    // days. The rotation below runs at init, which mobile executes on every app
    // start and on every MeshService warm boot, so a file written by an older
    // build cannot be inherited unbounded. Growth WITHIN one session is bounded by
    // the `CappedFileWriter` sink below, which counts bytes and rotates at the
    // same cap; the retry-cycle cure in core/src/transport (separate, gated
    // change) bounds the rate at which those bytes are produced.
    if rotate_oversized_log(&log_path.join(MESH_LOG_FILENAME), MESH_LOG_CAP_BYTES)? {
        tracing::warn!(
            event = "mesh_log_rotated",
            cap_bytes = MESH_LOG_CAP_BYTES,
            rotated_to = %format!("{}.1", MESH_LOG_FILENAME)
        );
    }

    // Non-blocking file appender (prevents I/O from blocking async mesh router).
    // The sink is `CappedFileWriter`, not `rolling::never`: the appender applies
    // no size policy of its own, so a session that never restarts would write an
    // unbounded file. The sink rotates at the cap during the session; the rotation
    // above bounds the file that already exists when this process starts.
    let file_sink = CappedFileWriter::open(&log_path.join(MESH_LOG_FILENAME), MESH_LOG_CAP_BYTES)?;
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_sink);

    // CRITICAL: Must leak the guard to prevent appender from flushing/dropping prematurely.
    // The guard's lifetime must outlive the tracing subscriber, or logs will be lost.
    // This is safe because the tracing subscriber is process-global and never uninstalled.
    std::mem::forget(_guard);

    // Build JSON formatter layer
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_target(false) // Omit module paths for cleaner mobile logs
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    // ENV-based filter. RUST_LOG is unavailable on mobile (there is no shell
    // to export it from before the app process starts), so this fallback is
    // the only lever mobile builds have. Debug/dev cargo profiles (Android
    // `assembleDebug`, built via cargo-ndk WITHOUT `--release` per
    // android/app/build.gradle's `rustProfile`; and local `cargo build`) get
    // a richer default so a field test captures enough to debug message
    // delivery end to end. Optimized/release profiles (Android
    // `assembleRelease`, and the iOS xcframework -- which scripts/
    // rebuild_ios_core.sh ALWAYS builds with `--release`, independent of the
    // Xcode scheme -- so this richer default never applies to iOS) keep the
    // quieter "info" default to bound on-device log file growth and avoid
    // ever writing chat content to disk. RUST_LOG, when settable (desktop
    // CLI), always takes precedence over this fallback either way.
    #[cfg(debug_assertions)]
    let default_filter = "info,scmessenger_core::transport=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug,scmessenger_core::relay=debug";
    #[cfg(not(debug_assertions))]
    let default_filter = "info";

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    // Install global subscriber (try_init = mobile-safe for warm boots)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .try_init()
        .map_err(|e| format!("Tracing init failed: {}", e))?;

    tracing::info!(
        event = "tracing_initialized",
        log_directory = %log_directory,
        format = "json"
    );

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn init_file_tracing(_log_directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    // File logging is not supported on WASM target; rely on console_error_panic_hook or js tracing instead.
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{rotate_oversized_log, CappedFileWriter, MESH_LOG_CAP_BYTES, MESH_LOG_FILENAME};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    /// Scratch space under the repo-local `tmp/` (AGENTS.md rule 2: never the
    /// system temp dir). Removed by the caller.
    fn scratch(name: &str) -> PathBuf {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate has a parent")
            .join("tmp");
        std::fs::create_dir_all(&base).expect("repo tmp/ is creatable");
        let dir = base.join(format!(
            "tracing-rotate-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    #[test]
    fn live_log_rotates_within_a_session_at_the_cap() {
        // #684: the start-up rotation bounds the file across restarts only. A node
        // that stays up (the always-on cloud node) has to be bounded inside the
        // session too, or #679's 4.19 GB growth simply recurs without a restart.
        let dir = scratch("capped-writer");
        let log = dir.join(MESH_LOG_FILENAME);
        let cap = 2048u64;
        let first = vec![b'a'; 1200];
        let second = vec![b'b'; 1200];

        {
            let mut sink = CappedFileWriter::open(&log, cap).expect("sink opens");
            sink.write_all(&first).expect("first write fits");
            assert_eq!(sink.rotations, 0, "the first write is under the cap");
            sink.write_all(&second)
                .expect("second write crosses the cap and still lands");
            sink.flush().expect("flush");
            assert_eq!(sink.rotations, 1, "crossing the cap rotates exactly once");
        }

        let live = std::fs::read(&log).expect("live log is readable");
        assert!(
            live.len() as u64 <= cap,
            "the live file stays under the cap ({} bytes)",
            live.len()
        );
        assert!(
            live.ends_with(&second),
            "the write that crossed the cap is in the live file"
        );
        assert!(
            String::from_utf8_lossy(&live).contains("mesh_log_rotated"),
            "the rotation is auditable from the file itself"
        );

        let rotated = dir.join(format!("{}.1", MESH_LOG_FILENAME));
        let previous = std::fs::read(&rotated).expect("rotated file exists");
        assert_eq!(
            previous, first,
            "the pre-rotation evidence is preserved in .1"
        );
        assert!(
            previous.len() as u64 <= cap,
            "the rotated file is under the cap as well"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn oversized_log_is_rotated_and_replaced_on_the_next_run() {
        let dir = scratch("rotate");
        let log = dir.join(MESH_LOG_FILENAME);
        std::fs::write(&log, b"first run evidence").unwrap();

        // A cap of 1 byte makes the existing file oversized without writing 64 MiB.
        assert!(
            rotate_oversized_log(&log, 1).unwrap(),
            "oversized file rotates"
        );
        assert!(
            !log.exists(),
            "the live log path is free for the new session"
        );
        let rotated = dir.join(format!("{}.1", MESH_LOG_FILENAME));
        assert_eq!(std::fs::read(&rotated).unwrap(), b"first run evidence");

        // A second rotation replaces the oldest evidence instead of accumulating,
        // so the pair of files stays bounded across restarts.
        std::fs::write(&log, b"second run").unwrap();
        assert!(rotate_oversized_log(&log, 1).unwrap());
        assert_eq!(std::fs::read(&rotated).unwrap(), b"second run");
        assert_eq!(
            std::fs::read_dir(&dir).unwrap().count(),
            1,
            "at most the live file and one .1 exist"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn healthy_log_and_missing_log_are_left_alone() {
        let dir = scratch("keep");
        let log = dir.join(MESH_LOG_FILENAME);

        assert!(
            !rotate_oversized_log(&log, MESH_LOG_CAP_BYTES).unwrap(),
            "a missing file is not an error and nothing is created"
        );
        assert!(!log.exists());

        std::fs::write(&log, b"small").unwrap();
        assert!(!rotate_oversized_log(&log, MESH_LOG_CAP_BYTES).unwrap());
        assert_eq!(std::fs::read(&log).unwrap(), b"small");
        assert!(!dir.join(format!("{}.1", MESH_LOG_FILENAME)).exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
