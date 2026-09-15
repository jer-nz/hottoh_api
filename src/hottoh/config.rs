use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

/// Default port of the HTTP API and of the web interface
pub const DEFAULT_HTTP_PORT: u16 = 3000;
/// Default TCP port of the HottoH Wi-Fi module
pub const DEFAULT_STOVE_PORT: u16 = 5001;

/// Configuration for logging
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error), optionally followed by per-module levels:
    /// `debug, actix_server = info`
    pub level: String,
    /// Directory where log files (and the alarm history) are stored
    pub directory: String,
    /// Rotated log files kept (compressed ones when `compress` is set)
    pub max_log_files: usize,
    /// Gzip the rotated log files
    pub compress: bool,
    /// The log file is also rotated when it grows beyond this size
    pub max_file_size_mb: u64,
    /// Interval of the statistics line in the log, in seconds (0 = disabled)
    pub stats_interval_s: u64,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
            directory: default_log_directory(),
            max_log_files: 7,
            compress: false,
            max_file_size_mb: 100,
            stats_interval_s: 3600,
        }
    }
}

/// Log directory when none is configured: the usual place for application data of the system
/// (`%LOCALAPPDATA%\hottoh_api\logs`, `~/Library/Logs/hottoh_api`,
/// `~/.local/state/hottoh_api/logs`), `logs` in the working directory otherwise
pub fn default_log_directory() -> String {
    let env = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let directory = if cfg!(windows) {
        env("LOCALAPPDATA").map(|d| d.join("hottoh_api").join("logs"))
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library").join("Logs").join("hottoh_api"))
    } else {
        env("XDG_STATE_HOME")
            .or_else(|| env("HOME").map(|h| h.join(".local").join("state")))
            .map(|d| d.join("hottoh_api").join("logs"))
    };
    directory
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_else(|| "logs".into())
}

/// Configuration for the stove connection
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct StoveConfig {
    /// IP address or host name of the Wi-Fi module; empty or `auto` to search the local network
    pub ip: String,
    /// TCP port of the stove
    pub port: u16,
    /// Pause between two polling cycles (INF + DAT 0/1/2), in milliseconds
    pub poll_interval_ms: u64,
}

impl Default for StoveConfig {
    fn default() -> Self {
        Self {
            ip: String::new(),
            port: DEFAULT_STOVE_PORT,
            poll_interval_ms: 1000,
        }
    }
}

impl StoveConfig {
    /// `host:port` of the configured stove, `None` when it must be searched on the network
    pub fn fixed_address(&self) -> Option<String> {
        let ip = self.ip.trim();
        (!ip.is_empty() && !ip.eq_ignore_ascii_case("auto"))
            .then(|| format!("{}:{}", ip, self.port))
    }
}

/// Configuration for the HTTP API
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct HttpApiConfig {
    /// IP address to bind the HTTP server (this computer only by default)
    pub ip: String,
    /// Port to bind the HTTP server
    pub port: u16,
}

impl Default for HttpApiConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".into(),
            port: DEFAULT_HTTP_PORT,
        }
    }
}

/// Configuration of the web interface
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WebUiConfig {
    /// Serves the web interface
    pub enabled: bool,
    /// Address of the web interface, the `[http_api]` one when left out
    pub ip: Option<String>,
    /// Port of the web interface, the `[http_api]` one when left out. On another port, the API is
    /// served there too, so that the interface stays on the same origin as the API.
    pub port: Option<u16>,
    /// Opens the interface in the default browser at startup (default: only without
    /// configuration file)
    pub open_browser: Option<bool>,
}

impl Default for WebUiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ip: None,
            port: None,
            open_browser: None,
        }
    }
}

impl WebUiConfig {
    /// Address of a separate web interface server, or `None` when it is disabled or shares the
    /// address of the API
    pub fn separate_address(&self, api: &HttpApiConfig) -> Option<String> {
        let ip = self.ip.as_deref().unwrap_or(&api.ip);
        let port = self.port.unwrap_or(api.port);
        (self.enabled && (ip != api.ip || port != api.port)).then(|| format!("{}:{}", ip, port))
    }
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
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub stove: StoveConfig,
    pub http_api: HttpApiConfig,
    pub web_ui: WebUiConfig,
    pub log: LogConfig,
    pub features: FeaturesConfig,
    /// Configuration file read, `None` when running without one (every setting by default)
    #[serde(skip)]
    pub file: Option<PathBuf>,
}

impl AppConfig {
    /// Started without configuration file: someone ran the program directly
    pub fn is_desktop(&self) -> bool {
        self.file.is_none()
    }
}

/// Loads the configuration: the given INI file, otherwise `config.ini` in the working directory
/// or next to the program. Without any, every setting keeps its default value.
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, ConfigError> {
    let file = match config_path {
        Some(path) => Some(PathBuf::from(path)),
        None => default_config_file(),
    };
    let mut builder = Config::builder();
    if let Some(path) = &file {
        builder = builder.add_source(File::new(&path.to_string_lossy(), FileFormat::Ini));
    }
    let mut config: AppConfig = builder.build()?.try_deserialize()?;
    config.file = file;
    Ok(config)
}

fn default_config_file() -> Option<PathBuf> {
    let beside_program = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("config.ini")));
    [Some(PathBuf::from("config.ini")), beside_program]
        .into_iter()
        .flatten()
        .find(|path| path.is_file())
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
        assert_eq!(config.web_ui, WebUiConfig::default());
        assert_eq!(config.web_ui.separate_address(&config.http_api), None);
    }

    #[test]
    fn web_ui_can_have_its_own_port_or_be_disabled() {
        let api = HttpApiConfig {
            ip: "0.0.0.0".into(),
            port: 3000,
        };
        let on = |ip: Option<&str>, port| WebUiConfig {
            ip: ip.map(str::to_string),
            port,
            ..WebUiConfig::default()
        };
        assert_eq!(on(None, Some(3000)).separate_address(&api), None);
        assert_eq!(on(Some("0.0.0.0"), None).separate_address(&api), None);
        assert_eq!(
            on(None, Some(8080)).separate_address(&api).as_deref(),
            Some("0.0.0.0:8080")
        );
        assert_eq!(
            on(Some("127.0.0.1"), None)
                .separate_address(&api)
                .as_deref(),
            Some("127.0.0.1:3000")
        );
        let off = WebUiConfig {
            enabled: false,
            ..on(None, Some(8080))
        };
        assert_eq!(off.separate_address(&api), None);

        let ini = "[stove]\nip = 192.168.1.100\nport = 5001\n\
                   [http_api]\nip = 0.0.0.0\nport = 80\n\
                   [web_ui]\nenabled = false\nport = 8080\n\
                   [log]\nlevel = info\ndirectory = logs\nmax_log_files = 7\n";
        let config: AppConfig = Config::builder()
            .add_source(File::from_str(ini, FileFormat::Ini))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert!(!config.web_ui.enabled);
        assert_eq!(config.web_ui.port, Some(8080));
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

    #[test]
    fn everything_has_a_default_without_file() {
        let config: AppConfig = Config::builder()
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert_eq!(config.stove.fixed_address(), None);
        assert_eq!(config.stove.port, 5001);
        assert_eq!(
            (config.http_api.ip.as_str(), config.http_api.port),
            ("127.0.0.1", 3000)
        );
        assert!(config.web_ui.enabled && config.web_ui.open_browser.is_none());
        assert_eq!(config.log.level, "info");
        assert!(!config.log.directory.is_empty());

        let ini = "[stove]\nip = auto\n";
        let config: AppConfig = Config::builder()
            .add_source(File::from_str(ini, FileFormat::Ini))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert_eq!(config.stove.fixed_address(), None);
        let ini = "[stove]\nip = 192.168.1.150\n";
        let config: AppConfig = Config::builder()
            .add_source(File::from_str(ini, FileFormat::Ini))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        assert_eq!(
            config.stove.fixed_address().as_deref(),
            Some("192.168.1.150:5001")
        );
    }
}
