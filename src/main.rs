mod hottoh;

use hottoh::config::load_config;
use hottoh::http_api::start_http_server;
use hottoh::logger::{initialize_logger, log_panics};
use hottoh::shared_struct::Bridge;
use hottoh::stats::spawn_reporter;
use hottoh::tcp_client::{StoveTarget, TcpClient};
use log::{error, info};
use std::path::Path;
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
    let desktop = config.is_desktop();
    if desktop {
        println!(
            "hottoh_api {} - no config.ini: searching the stove on the local network.\n\
             Keep this window open while you use the interface; close it (or Ctrl+C) to stop.\n\
             Log files: {}",
            env!("CARGO_PKG_VERSION"),
            config.log.directory
        );
    }
    let _logger = match initialize_logger(&config.log, desktop) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            std::process::exit(1);
        }
    };
    log_panics();
    let target = match config.stove.fixed_address() {
        Some(address) => StoveTarget::Fixed(address),
        None => StoveTarget::Discover(config.stove.port),
    };
    info!(
        "Starting hottoh_api {} (config {}, stove {}, poll every {} ms, HTTP {}:{}, log level '{}')",
        env!("CARGO_PKG_VERSION"),
        config
            .file
            .as_deref()
            .map_or("none".into(), |f| f.display().to_string()),
        match &target {
            StoveTarget::Fixed(address) => address.clone(),
            StoveTarget::Discover(port) => format!("searched on port {}", port),
        },
        config.stove.poll_interval_ms,
        config.http_api.ip,
        config.http_api.port,
        config.log.level
    );

    info!(
        "Enabled module features: {}",
        config.features.enabled().join(", ")
    );

    let bridge = Arc::new(
        Bridge::new().with_alarm_file(Path::new(&config.log.directory).join("alarms.json")),
    );
    let worker = TcpClient::new(
        target,
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
        &config.web_ui,
        config.features.clone(),
        Arc::clone(&bridge),
        desktop,
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
