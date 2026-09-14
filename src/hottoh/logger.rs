use crate::hottoh::config::AppConfig;
use chrono::Local;
use flexi_logger::{
    Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming, WriteMode,
};
use log::Record;
use std::error::Error;
use std::io::Write;

/// Custom log formatter that includes timestamp, log level, and message
///
/// # Arguments
///
/// * `w` - Writer to output the formatted log
/// * `_now` - Deferred timestamp (not used as we use chrono directly)
/// * `record` - Log record containing level, target, and message
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Success or IO error
fn custom_format(
    w: &mut dyn Write,
    _now: &mut flexi_logger::DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    let level_str = match level {
        log::Level::Error => "ERROR",
        log::Level::Warn => "WARN",
        log::Level::Info => "INFO",
        log::Level::Debug => "DEBUG",
        log::Level::Trace => "TRACE",
    };

    write!(
        w,
        "{} {} [{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        level_str,
        record.target(),
        record.args()
    )
}

/// Initializes the application logger with configuration from AppConfig
///
/// Sets up file logging with rotation, console output, and custom formatting.
///
/// # Arguments
///
/// * `cfg` - Application configuration containing log settings
///
/// # Returns
///
/// * `Result<LoggerHandle, Box<dyn Error>>` - The logger handle, or an error
///
/// The returned handle must be kept alive: dropping it stops the buffered file writer.
pub fn initialize_logger(cfg: &AppConfig) -> Result<LoggerHandle, Box<dyn Error>> {
    let handle = Logger::try_with_str(&cfg.log.level)
        .map_err(|e| format!("Failed to initialize logger: {}", e))?
        .log_to_file(
            FileSpec::default()
                .directory(&cfg.log.directory)
                .suffix("log"),
        )
        .write_mode(WriteMode::BufferAndFlush)
        .duplicate_to_stderr(Duplicate::Info)
        .format(custom_format)
        .rotate(
            Criterion::Age(flexi_logger::Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(cfg.log.max_log_files),
        )
        .start()
        .map_err(|e| format!("Failed to start logger: {}", e))?;

    Ok(handle)
}
