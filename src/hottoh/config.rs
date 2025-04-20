use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

/// Configuration for logging
#[derive(Debug, Deserialize)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Directory where log files will be stored
    pub directory: String,
    /// Maximum number of log files to keep
    pub max_log_files: usize,
}

/// Configuration for the stove connection
#[derive(Debug, Deserialize)]
pub struct StoveConfig {
    /// IP address of the stove
    pub ip: String,
    /// TCP port of the stove
    pub port: u16,
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
    /// Stove connection configuration
    pub stove: StoveConfig,
    /// HTTP API configuration
    pub http_api: HttpApiConfig,
    /// Logging configuration
    pub log: LogConfig,
}

/// Loads the application configuration from a file
///
/// # Arguments
///
/// * `config_path` - Optional path to the configuration file. If not provided, "config" is used.
///
/// # Returns
///
/// * `Result<AppConfig, ConfigError>` - The loaded configuration or an error
pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, ConfigError> {
    let path = config_path.unwrap_or("config");
    let settings = Config::builder()
        .add_source(File::new(path, FileFormat::Ini))
        .build()?;
    settings.try_deserialize::<AppConfig>()
}
