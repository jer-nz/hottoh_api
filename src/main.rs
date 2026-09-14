mod hottoh;

use crate::hottoh::http_api::start_http_server;
use crate::hottoh::shared_struct::SharedState;
use actix_web::rt::System;
use hottoh::config::load_config;
use hottoh::logger::initialize_logger;
use hottoh::tcp_client::{TcpClient, WriteQueue};
use log::{error, info};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
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
    println!(
        "Stove: {}:{}, HTTP API port: {}",
        config.stove.ip, config.stove.port, config.http_api.port
    );
    let _logger = match initialize_logger(&config) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            std::process::exit(1);
        }
    };
    info!("Starting hottoh_api {}...", env!("CARGO_PKG_VERSION"));

    let running = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let running = Arc::clone(&running);
        move || {
            info!("Ctrl-C received! Exiting...");
            running.store(false, Ordering::SeqCst);
            System::current().stop();
        }
    })
    .expect("Error while setting the Ctrl-C handler");

    let shared_state = Arc::new(RwLock::new(SharedState::new()));
    let writes: WriteQueue = Arc::new(Mutex::new(VecDeque::new()));
    let request_id = Arc::new(AtomicU32::new(1));

    let worker = TcpClient::new(
        format!("{}:{}", config.stove.ip, config.stove.port),
        Duration::from_millis(config.stove.poll_interval_ms),
        Arc::clone(&writes),
        Arc::clone(&shared_state),
        Arc::clone(&request_id),
        Arc::clone(&running),
    )
    .start();

    let result = start_http_server(&config, shared_state, writes, request_id).await;

    running.store(false, Ordering::SeqCst);
    if worker.join().is_err() {
        error!("TCP worker thread ended with a panic");
    }
    result
}
