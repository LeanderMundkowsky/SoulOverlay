use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Rotate the previous `latest.log` into `<logs_dir>/YYYY-MM-DD_HH-MM-SS.log`.
/// Silently ignores errors — logging should never prevent the app from starting.
fn rotate_previous_log(log_file: &Path, logs_dir: &Path) {
    if !log_file.exists() {
        return;
    }

    // Read the file's last-modified time for the archive name
    let timestamp = std::fs::metadata(log_file)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(chrono::DateTime::<chrono::Local>::from)
        .unwrap_or_else(chrono::Local::now);

    if let Err(e) = std::fs::create_dir_all(logs_dir) {
        eprintln!("[WARN] Could not create logs directory: {}", e);
        return;
    }

    let archive_name = format!("{}.log", timestamp.format("%Y-%m-%d_%H-%M-%S"));
    let dest = logs_dir.join(archive_name);

    if let Err(e) = std::fs::rename(log_file, &dest) {
        eprintln!("[WARN] Could not rotate log file: {}", e);
    }
}

/// Set up logging to both stderr and a log file (`latest.log`).
/// The previous log is rotated into `<logs_dir>/` with a timestamped name.
/// Falls back to stderr-only if the file can't be created.
///
/// Returns a shared flag that controls debug-level output at runtime.
/// When `false` (default), only Info+ messages are emitted for `soul_overlay_lib`.
/// When `true`, Debug messages from `soul_overlay_lib` are also emitted.
pub fn setup(log_file: &Path, logs_dir: &Path) -> Arc<AtomicBool> {
    rotate_previous_log(log_file, logs_dir);

    let debug_flag = Arc::new(AtomicBool::new(false));
    let filter_flag = debug_flag.clone();

    // Build stderr output
    let stderr_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stderr());

    // Try to set up a file logger at the given path
    let file_dispatch = (|| -> Option<fern::Dispatch> {
        let file = fern::log_file(log_file).ok()?;
        Some(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}][{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .chain(file),
        )
    })();

    let mut dispatch = fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        // Always allow Debug through fern's level gate for our crate;
        // the .filter() below decides at runtime whether to actually emit it.
        .level_for("soul_overlay_lib", log::LevelFilter::Debug)
        // Suppress noisy deps
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("tao", log::LevelFilter::Warn)
        .level_for("wry", log::LevelFilter::Warn)
        .filter(move |metadata| {
            if metadata.level() == log::Level::Debug {
                filter_flag.load(Ordering::Relaxed)
            } else {
                true
            }
        })
        .chain(stderr_dispatch);

    if let Some(fd) = file_dispatch {
        dispatch = dispatch.chain(fd);
        // Can't use log macros yet — logger not initialised
        eprintln!("[INFO] File logging enabled at {}", log_file.display());
    } else {
        eprintln!(
            "[WARN] Could not create log file at {}, logging to stderr only",
            log_file.display()
        );
    }

    dispatch.apply().unwrap_or_else(|e| {
        eprintln!("[ERROR] Failed to initialize logger: {}", e);
    });

    debug_flag
}
