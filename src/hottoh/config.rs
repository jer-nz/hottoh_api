use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

/// Optional features of the Wi-Fi module. Reads without side effect are enabled by default;
/// anything that exposes a secret, changes the module setup, deletes data or restarts the module
/// must be enabled explicitly. A disabled feature answers HTTP 403.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(default)]
pub struct FeaturesConfig {
    /// `GET /api/chrono/schedule`: weekly chrono schedule
    pub chrono_schedule_read: bool,
    /// `POST /api/chrono/schedule`: replaces days of the weekly schedule
    pub chrono_schedule_write: bool,
    /// `GET /api/clock`: module and stove clocks
    pub clock_read: bool,
    /// `POST /api/clock`: sets the module and stove clocks
    pub clock_write: bool,
    /// `GET /api/timezone`: time zone of the module
    pub timezone_read: bool,
    /// `POST /api/timezone`: changes the time zone (and sets the stove clock)
    pub timezone_write: bool,
    /// `GET /api/datalog/info` and `GET /api/datalog`: history recorded by the module
    pub datalog_read: bool,
    /// `POST /api/datalog/clear`: deletes the whole history
    pub datalog_clear: bool,
    /// `GET /api/pin`: security PIN of the cloud relay, as set by the owner in AppFire
    pub pin_read: bool,
    /// `POST /api/pin`: changes that PIN (AppFire in cloud mode must be paired again)
    pub pin_write: bool,
    /// `POST /api/module/restart`: restarts the Wi-Fi module
    pub module_restart: bool,
    /// `GET /api/wifi/scan`: networks seen by the module. The stove link is suspended for a few
    /// seconds, and the firmware may misreport (or crash on) WPA3 networks
    pub wifi_scan: bool,
    /// `GET /api/cloud`: HottoH relay and 4-noks cloud servers configured in the module
    pub cloud_read: bool,
    /// `GET /api/firmware`: asks update.hottoh.it whether a newer module firmware exists
    pub firmware_update_check: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            chrono_schedule_read: true,
            chrono_schedule_write: true,
            clock_read: true,
            clock_write: false,
            timezone_read: true,
            timezone_write: false,
            datalog_read: true,
            datalog_clear: false,
            pin_read: true,
            pin_write: false,
            module_restart: false,
            wifi_scan: false,
            cloud_read: true,
            firmware_update_check: true,
        }
    }
}

impl FeaturesConfig {
    /// Names of the enabled features, for the startup log
    pub fn enabled(&self) -> Vec<&'static str> {
        [
            ("chrono_schedule_read", self.chrono_schedule_read),
            ("chrono_schedule_write", self.chrono_schedule_write),
            ("clock_read", self.clock_read),
            ("clock_write", self.clock_write),
            ("timezone_read", self.timezone_read),
            ("timezone_write", self.timezone_write),
            ("datalog_read", self.datalog_read),
            ("datalog_clear", self.datalog_clear),
            ("pin_read", self.pin_read),
            ("pin_write", self.pin_write),
            ("module_restart", self.module_restart),
            ("wifi_scan", self.wifi_scan),
            ("cloud_read", self.cloud_read),
            ("firmware_update_check", self.firmware_update_check),
        ]
        .into_iter()
        .filter_map(|(name, enabled)| enabled.then_some(name))
        .collect()
    }
}

/// Main application configuration
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub stove: StoveConfig,
    pub http_api: HttpApiConfig,
    pub log: LogConfig,
    #[serde(default)]
    pub features: FeaturesConfig,
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
        assert_eq!(config.features, FeaturesConfig::default());
        assert!(config.features.pin_read && !config.features.pin_write);
    }

    #[test]
    fn features_can_be_enabled_one_by_one() {
        let ini = "[stove]\nip = 192.168.1.100\nport = 5001\n\
                   [http_api]\nip = 0.0.0.0\nport = 80\n\
                   [log]\nlevel = info\ndirectory = logs\nmax_log_files = 7\n\
                   [features]\nclock_write = true\nchrono_schedule_write = false\n";
        let config: AppConfig = Config::builder()
            .add_source(File::from_str(ini, FileFormat::Ini))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert!(config.features.clock_write);
        assert!(!config.features.chrono_schedule_write);
        assert!(config.features.datalog_read);
        assert!(!config.features.enabled().contains(&"pin_write"));
    }
}
