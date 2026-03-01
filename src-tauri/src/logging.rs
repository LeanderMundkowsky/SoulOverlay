/// Set up logging to both stderr and a log file in %APPDATA%/SoulOverlay/.
/// The log file is rotated on each launch (overwritten, not appended) to keep
/// it at a reasonable size. Falls back to stderr-only if the file can't be created.
pub fn setup() {
    use std::fs;

    let log_level = log::LevelFilter::Info;

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

    // Try to set up a file logger in %APPDATA%/SoulOverlay/
    let file_dispatch = (|| -> Option<fern::Dispatch> {
        let app_data = std::env::var("APPDATA").ok()?;
        let log_dir = std::path::PathBuf::from(app_data).join("SoulOverlay");
        fs::create_dir_all(&log_dir).ok()?;
        let log_path = log_dir.join("soul-overlay.log");
        let file = fern::log_file(&log_path).ok()?;
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
        .level(log_level)
        // Suppress noisy deps
        .level_for("hyper", log::LevelFilter::Warn)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("tao", log::LevelFilter::Warn)
        .level_for("wry", log::LevelFilter::Warn)
        .chain(stderr_dispatch);

    if let Some(fd) = file_dispatch {
        dispatch = dispatch.chain(fd);
        // Can't use log macros yet — logger not initialised
        eprintln!("[INFO] File logging enabled");
    } else {
        eprintln!("[WARN] Could not create log file, logging to stderr only");
    }

    dispatch.apply().unwrap_or_else(|e| {
        eprintln!("[ERROR] Failed to initialize logger: {}", e);
    });
}
