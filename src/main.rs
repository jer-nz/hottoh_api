mod hottoh;

use hottoh::config::load_config;
use hottoh::http_api::start_http_server;
use hottoh::logger::{initialize_logger, log_panics};
use hottoh::shared_struct::Bridge;
use hottoh::stats::spawn_reporter;
use hottoh::tcp_client::TcpClient;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_path = std::env::args().nth(1);
    let config = match load_config(config_path.as_deref()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    let _logger = match initialize_logger(&config.log) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            std::process::exit(1);
        }
    };
    log_panics();
    info!(
        "Starting hottoh_api {} (stove {}:{}, poll every {} ms, HTTP {}:{}, log level '{}')",
        env!("CARGO_PKG_VERSION"),
        config.stove.ip,
        config.stove.port,
        config.stove.poll_interval_ms,
        config.http_api.ip,
        config.http_api.port,
        config.log.level
    );

    info!(
        "Enabled module features: {}",
        config.features.enabled().join(", ")
    );

    let bridge = Arc::new(Bridge::new());
    let worker = TcpClient::new(
        format!("{}:{}", config.stove.ip, config.stove.port),
        Duration::from_millis(config.stove.poll_interval_ms),
        Arc::clone(&bridge),
    )
    .start();
    let reporter = spawn_reporter(
        Arc::clone(&bridge),
        Duration::from_secs(config.log.stats_interval_s),
    );

    // Returns on SIGINT or SIGTERM, or if the address cannot be bound
    let result = start_http_server(
        &config.http_api,
        config.features.clone(),
        Arc::clone(&bridge),
    )
    .await;
    match &result {
        Ok(()) => info!("HTTP server stopped"),
        Err(e) => error!("HTTP server failed: {}", e),
    }

    bridge.stop();
    for (name, handle) in [("TCP worker", Some(worker)), ("statistics", reporter)] {
        if let Some(handle) = handle
            && handle.join().is_err()
        {
            error!("{} thread ended with a panic", name);
        }
    }
    info!("hottoh_api stopped");
    result
}
