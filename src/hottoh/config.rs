use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

/// Configuration for logging
#[derive(Debug, Deserialize)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error), optionally followed by per-module levels:
    /// `debug, actix_server = info`
    pub level: String,
    /// Directory where log files will be stored
    pub directory: String,
    /// Rotated log files kept (compressed ones when `compress` is set)
    pub max_log_files: usize,
    /// Gzip the rotated log files
    #[serde(default)]
    pub compress: bool,
    /// The log file is also rotated when it grows beyond this size
    #[serde(default = "default_max_file_size_mb")]
    pub max_file_size_mb: u64,
    /// Interval of the statistics line in the log, in seconds (0 = disabled)
    #[serde(default = "default_stats_interval_s")]
    pub stats_interval_s: u64,
}

fn default_max_file_size_mb() -> u64 {
    100
}

fn default_stats_interval_s() -> u64 {
    3600
}

/// Configuration for the stove connection
#[derive(Debug, Deserialize)]
pub struct StoveConfig {
    /// IP address of the stove
    pub ip: String,
    /// TCP port of the stove
    pub port: u16,
    /// Pause between two polling cycles (INF + DAT 0/1/2), in milliseconds
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
}

fn default_poll_interval_ms() -> u64 {
    1000
}

/// Configuration for the HTTP API
#[derive(Debug, Deserialize)]
pub struct HttpApiConfig {
    /// IP address to bind the HTTP server
    pub ip: String,
    /// Port to bind the HTTP server
    pub port: u16,
}

/// Main application configuration
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub stove: StoveConfig,
    pub http_api: HttpApiConfig,
    pub log: LogConfig,
}

/// Loads the configuration from an INI file (`config.ini` in the working directory by default)
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, ConfigError> {
    let path = config_path.unwrap_or("config");
    Config::builder()
        .add_source(File::new(path, FileFormat::Ini))
        .build()?
        .try_deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_settings_have_defaults() {
        let ini = "[stove]\nip = 192.168.1.100\nport = 5001\n\
                   [http_api]\nip = 0.0.0.0\nport = 80\n\
                   [log]\nlevel = debug, actix_server = info\ndirectory = logs\nmax_log_files = 7\n";
        let config: AppConfig = Config::builder()
            .add_source(File::from_str(ini, FileFormat::Ini))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert_eq!(config.stove.poll_interval_ms, 1000);
        assert_eq!(config.log.level, "debug, actix_server = info");
        assert!(!config.log.compress);
        assert_eq!(config.log.max_file_size_mb, 100);
        assert_eq!(config.log.stats_interval_s, 3600);
    }
}
