use crate::hottoh::config::LogConfig;
use flexi_logger::{
    Age, Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, LoggerHandle, Naming,
    WriteMode,
};
use log::{Record, error};
use std::backtrace::Backtrace;
use std::error::Error;
use std::io::Write;
use std::{panic, thread};

/// `2026-09-14 22:54:33.297 INFO [module] message`
fn custom_format(
    w: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{} {} [{}] {}",
        now.format("%Y-%m-%d %H:%M:%S%.3f"),
        record.level(),
        record.target(),
        record.args()
    )
}

/// Starts file logging with daily (or size based) rotation.
///
/// The returned handle must be kept alive until the end of the program.
///
/// `console` also prints info messages on stderr, for someone running the program in a terminal
/// window.
pub fn initialize_logger(cfg: &LogConfig, console: bool) -> Result<LoggerHandle, Box<dyn Error>> {
    let cleanup = if cfg.compress {
        // The previous file stays readable, older ones are gzipped
        Cleanup::KeepLogAndCompressedFiles(1, cfg.max_log_files)
    } else {
        Cleanup::KeepLogFiles(cfg.max_log_files)
    };
    let handle = Logger::try_with_str(&cfg.level)
        .map_err(|e| format!("invalid log level '{}': {}", cfg.level, e))?
        .log_to_file(FileSpec::default().directory(&cfg.directory).suffix("log"))
        // Unbuffered: the last lines before a crash or a kill are not lost
        .write_mode(WriteMode::Direct)
        .duplicate_to_stderr(if console {
            Duplicate::Info
        } else {
            Duplicate::Error
        })
        .format(custom_format)
        .rotate(
            Criterion::AgeOrSize(Age::Day, cfg.max_file_size_mb.saturating_mul(1024 * 1024)),
            Naming::Timestamps,
            cleanup,
        )
        .start()
        .map_err(|e| format!("cannot start logger: {}", e))?;
    Ok(handle)
}

/// Writes panics to the log with a backtrace, then lets the default hook print them on stderr
pub fn log_panics() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        error!(
            "Panic in thread '{}': {}\n{}",
            thread::current().name().unwrap_or("unnamed"),
            info,
            Backtrace::force_capture()
        );
        default_hook(info);
    }));
}
