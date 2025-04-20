mod hottoh;
use crate::hottoh::http_api::start_http_server;
use crate::hottoh::shared_struct::SharedState;
use hottoh::config::load_config;
use hottoh::logger::initialize_logger;
use hottoh::tcp_client::TcpClient;
use hottoh::tcp_client_structs::{Request, Response};
use log::info;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use actix_web::rt::System;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).map(|s| s.as_str());
    // Load configuration
    let config = match load_config(config_path) {
        Ok(config) => {
            println!("Configuration loaded successfully!");
            println!(
                "Stove IP: {}, Stove Port: {}",
                config.stove.ip, config.stove.port
            );
            println!("HTTP API Port: {}", config.http_api.port);
            Arc::new(RwLock::new(config))
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    initialize_logger(Arc::clone(&config)).expect("Failed to initialize logger");
    info!("Starting...");
    let running = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let running = Arc::clone(&running);
        move || {
            info!("Ctrl-C received! Exiting...");
            running.store(false, Ordering::SeqCst);
            // Stop the Actix system
            System::current().stop();
        }
    })
    .expect("Error while handling Ctrl-C");

    let request_id_counter = Arc::new(Mutex::new(0));
    let request_queue = Arc::new(RwLock::new(VecDeque::<Request>::new()));
    let response_queue = Arc::new(RwLock::new(VecDeque::<Response>::new()));
    let tcp_client = TcpClient::new(
        Arc::clone(&request_queue),
        Arc::clone(&response_queue),
        Arc::clone(&running),
    );
    let shared_state = Arc::new(RwLock::new(SharedState::new()));

    let http_server_task = start_http_server(
        Arc::clone(&request_queue),
        Arc::clone(&shared_state),
        Arc::clone(&request_id_counter),
        Arc::clone(&config),
    );

    let comm_handle = tcp_client.start_tcp_thread(Arc::clone(&config));
    let manage_handle = tcp_client.message_management_thread(shared_state);
    let periodic_handle = tcp_client.periodic_request_thread(Arc::clone(&request_id_counter));

    // Wait for the HTTP server task to complete
    http_server_task.await?;

    // Signal other threads to stop
    running.store(false, Ordering::SeqCst);

    // Wait for other threads to complete
    comm_handle.join().unwrap();
    manage_handle.join().unwrap();
    periodic_handle.join().unwrap();

    Ok(())
}
