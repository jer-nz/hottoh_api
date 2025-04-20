use super::hottoh_const::*;
use super::hottoh_structs::*;
use crate::hottoh::config::AppConfig;
use crate::hottoh::shared_struct::SharedState;
use crate::hottoh::tcp_client_structs::{Request, Response};
use log::{error, info, warn};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};
use std::time::{Duration, Instant};
use std::{panic, thread};

/// TCP client for communicating with the stove
///
/// Handles sending requests and receiving responses over TCP
pub struct TcpClient {
    /// Queue of requests to be sent to the stove
    request_queue: Arc<RwLock<VecDeque<Request>>>,
    /// Queue of responses received from the stove
    response_queue: Arc<RwLock<VecDeque<Response>>>,
    /// Flag indicating whether the client is running
    running: Arc<AtomicBool>,
}

impl TcpClient {
    /// Creates a new TCP client
    ///
    /// # Arguments
    ///
    /// * `request_queue` - Queue of requests to be sent to the stove
    /// * `response_queue` - Queue of responses received from the stove
    /// * `running` - Flag indicating whether the client is running
    ///
    /// # Returns
    ///
    /// * `TcpClient` - A new TCP client
    pub fn new(
        request_queue: Arc<RwLock<VecDeque<Request>>>,
        response_queue: Arc<RwLock<VecDeque<Response>>>,
        running: Arc<AtomicBool>,
    ) -> Self {
        TcpClient {
            request_queue,
            response_queue,
            running,
        }
    }

    /// Starts a thread for TCP communication with the stove
    ///
    /// This thread handles connecting to the stove, sending requests, and receiving responses
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing stove connection details
    ///
    /// # Returns
    ///
    /// * `thread::JoinHandle<()>` - Handle to the spawned thread
    pub fn start_tcp_thread(&self, config: Arc<RwLock<AppConfig>>) -> thread::JoinHandle<()> {
        let cfg = config.read().expect("Cannot read config in tcp thread.");
        let stove_address = format!("{}:{}", cfg.stove.ip, cfg.stove.port);
        let request_queue = Arc::clone(&self.request_queue);
        let response_queue = Arc::clone(&self.response_queue);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let result = panic::catch_unwind(|| loop {
                if !running.load(Ordering::SeqCst) {
                    info!("TCP client thread stopped.");
                    break;
                }

                let mut stream = match TcpStream::connect(&stove_address) {
                    Ok(stream) => {
                        info!("Connected to stove at {}", &stove_address);
                        stream
                            .set_nonblocking(true)
                            .expect("Failed to set non-blocking");
                        stream
                    }
                    Err(e) => {
                        if !running.load(Ordering::SeqCst) {
                            info!("TCP client thread stopped.");
                            break;
                        }
                        warn!(
                            "Could not connect to stove: {}. Retrying in 5 seconds...",
                            e
                        );
                        thread::sleep(Duration::from_secs(5));
                        continue;
                    }
                };

                let mut last_sent = Instant::now();

                loop {
                    if !running.load(Ordering::SeqCst) {
                        info!("TCP client thread stopped.");
                        break;
                    }

                    if last_sent.elapsed() >= Duration::from_millis(1000) {
                        if let Ok(mut req_queue) = request_queue.write() {
                            if let Some(request) = req_queue.front_mut() {
                                if !request.is_sent() {
                                    match stream.write_all(&request.build_message()) {
                                        Ok(_) => {
                                            request.mark_as_sent();
                                            last_sent = Instant::now();
                                        }
                                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                            continue;
                                        }
                                        Err(e) => {
                                            warn!("Failed to send request: {}. Reconnecting...", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut buffer = [0; 4096];
                    match stream.read(&mut buffer) {
                        Ok(size) if size > 0 => {
                            let response_str = String::from_utf8_lossy(&buffer[..size]);
                            // Split the string into individual messages
                            let messages: Vec<&str> =
                                response_str.split('#').filter(|s| !s.is_empty()).collect();
                            for message in messages {
                                let message_with_prefix = format!("#{}", message);
                                match Response::from_message(&message_with_prefix) {
                                    Ok(response) => {
                                        if let Ok(mut resp_queue) = response_queue.write() {
                                            resp_queue.push_back(response);
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "Error parsing response: {}. Raw response: '{}'",
                                            e, message_with_prefix
                                        );
                                    }
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(e) => {
                            if !running.load(Ordering::SeqCst) {
                                info!("TCP client thread stopped.");
                                break;
                            }
                            warn!("Failed to receive response: {}. Reconnecting...", e);
                            break;
                        }
                    }

                    thread::sleep(Duration::from_millis(200));
                }

                if !running.load(Ordering::SeqCst) {
                    info!("TCP client thread stopped.");
                    break;
                }

                info!("Disconnected from stove. Reconnecting in 5 seconds...");
                thread::sleep(Duration::from_secs(5));
            });

            if let Err(err) = result {
                error!("Thread panicked: {:?}", err);
            }
        })
    }

    /// Starts a thread for managing messages between requests and responses
    ///
    /// This thread matches responses to requests, updates the shared state,
    /// and handles timeouts
    ///
    /// # Arguments
    ///
    /// * `shared_state` - Shared state to be updated with response data
    ///
    /// # Returns
    ///
    /// * `thread::JoinHandle<()>` - Handle to the spawned thread
    pub fn message_management_thread(
        &self,
        shared_state: Arc<RwLock<SharedState>>,
    ) -> thread::JoinHandle<()> {
        let request_queue = Arc::clone(&self.request_queue);
        let response_queue = Arc::clone(&self.response_queue);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                if let Ok(mut req_queue) = request_queue.write() {
                    if let Ok(mut res_queue) = response_queue.write() {
                        for res in res_queue.iter_mut() {
                            // Check if the response corresponds to an existing request
                            let matching_req = req_queue
                                .iter()
                                .find(|req| req.get_req_id() == res.get_req_id());

                            if let Some(req) = matching_req {
                                // If the request is marked as deleted (timeout), log this information
                                if req.is_marked_as_deleted() {
                                    warn!(
                                        "Response received for timed out request: req_id={}, command={:?}, command_type={:?}, params={:?}",
                                        req.get_req_id(),
                                        req.get_command(),
                                        req.get_command_type(),
                                        req.get_params()
                                    );
                                }
                            } else {
                                // No corresponding request found
                                res.set_marked_as_deleted(true);
                            }
                        }
                        for req in req_queue.iter_mut() {
                            if req.is_marked_as_deleted() {
                                continue;
                            }
                            if req.is_sent() && req.get_sent_at().unwrap().elapsed().as_secs() > 5 {
                                warn!("Request timeout: req_id={}, command={:?}, command_type={:?}, params={:?}",
                                    req.get_req_id(),
                                    req.get_command(),
                                    req.get_command_type(),
                                    req.get_params()
                                );
                                req.set_marked_as_deleted(true);
                            }
                            for res in res_queue.iter_mut() {
                                if res.is_marked_as_deleted() {
                                    break;
                                }
                                if req.get_req_id() == res.get_req_id() {
                                    if res.is_crc_valid() {
                                        if let Ok(mut state) = shared_state.write() {
                                            match res.get_command_data() {
                                                CommandData::Inf(inf_data) => {
                                                    state.set_inf(inf_data)
                                                }
                                                CommandData::Dat0(dat0_data) => {
                                                    state.set_dat0(dat0_data)
                                                }
                                                CommandData::Dat1(dat1_data) => {
                                                    state.set_dat1(dat1_data)
                                                }
                                                CommandData::Dat2(dat2_data) => {
                                                    state.set_dat2(dat2_data)
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    res.set_marked_as_deleted(true);
                                    req.set_marked_as_deleted(true);
                                    break;
                                }
                            }
                        }
                    }
                }
                clean_queues(&request_queue, &response_queue);
                thread::sleep(Duration::from_millis(200));
            }
            info!("Message management thread stopped.");
        })
    }

    /// Starts a thread for sending periodic requests to the stove
    ///
    /// This thread sends INF and DAT requests at regular intervals
    ///
    /// # Arguments
    ///
    /// * `request_id_counter` - Counter for generating unique request IDs
    ///
    /// # Returns
    ///
    /// * `thread::JoinHandle<()>` - Handle to the spawned thread
    pub fn periodic_request_thread(
        &self,
        request_id_counter: Arc<Mutex<u32>>,
    ) -> thread::JoinHandle<()> {
        let request_queue = Arc::clone(&self.request_queue);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let result = panic::catch_unwind(|| {
                while running.load(Ordering::SeqCst) {
                    if let Ok(mut id_lock) = request_id_counter.lock() {
                        let request_id = *id_lock;
                        if !already_existing_request(
                            &Command::Inf,
                            &CommandType::Read,
                            &[],
                            &request_queue,
                        )
                        .unwrap_or(false)
                        {
                            if let Err(e) = send_request(
                                Request::new(request_id, Command::Inf, CommandType::Read, vec![]),
                                &request_queue,
                            ) {
                                warn!("Failed to send INF request: {:?}", e);
                            }
                            *id_lock = (*id_lock + 1) % 100000;
                        }
                    }

                    for i in 0..3 {
                        if let Ok(mut id_lock) = request_id_counter.lock() {
                            let request_id = *id_lock;
                            if !already_existing_request(
                                &Command::Dat,
                                &CommandType::Read,
                                &[i.to_string()],
                                &request_queue,
                            )
                            .unwrap_or(false)
                            {
                                if let Err(e) = send_request(
                                    Request::new(
                                        request_id,
                                        Command::Dat,
                                        CommandType::Read,
                                        vec![i.to_string()],
                                    ),
                                    &request_queue,
                                ) {
                                    warn!("Failed to send DAT{} request: {:?}", i, e);
                                }
                                *id_lock = (*id_lock + 1) % 100000;
                            }
                        }
                    }
                    thread::sleep(Duration::from_secs(1));
                }

                info!("Periodic request thread stopped.");
            });

            if let Err(err) = result {
                error!("Thread panicked: {:?}", err);
            }
        })
    }
}

/// Sends a request to the stove by adding it to the request queue
///
/// # Arguments
///
/// * `request` - The request to send
/// * `request_queue` - Queue of requests to be sent to the stove
///
/// # Returns
///
/// * `Result<(), String>` - Success or error message
pub fn send_request(
    request: Request,
    request_queue: &Arc<RwLock<VecDeque<Request>>>,
) -> Result<(), String> {
    if let Ok(mut queue) = request_queue.write() {
        queue.push_back(request);
        Ok(())
    } else {
        Err("Failed to lock request queue".to_string())
    }
}

/// Checks if a request with the same command, type, and parameters already exists in the queue
///
/// # Arguments
///
/// * `command` - The command to check
/// * `command_type` - The command type to check
/// * `params` - The parameters to check
/// * `request_queue` - Queue of requests to check
///
/// # Returns
///
/// * `Result<bool, String>` - True if a similar request exists, false otherwise, or an error
pub fn already_existing_request(
    command: &Command,
    command_type: &CommandType,
    params: &[String],
    request_queue: &Arc<RwLock<VecDeque<Request>>>,
) -> Result<bool, String> {
    if let Ok(queue) = request_queue.read() {
        let similar_exists = queue.iter().any(|r| {
            !r.is_marked_as_deleted()
                && r.get_command() == command
                && r.get_command_type() == command_type
                && r.get_params() == params
        });
        Ok(similar_exists)
    } else {
        Err("Failed to lock request queue for reading".to_string())
    }
}

/// Removes requests and responses that are marked for deletion from their respective queues
///
/// # Arguments
///
/// * `request_queue` - Queue of requests to clean
/// * `response_queue` - Queue of responses to clean
fn clean_queues(
    request_queue: &Arc<RwLock<VecDeque<Request>>>,
    response_queue: &Arc<RwLock<VecDeque<Response>>>,
) {
    if let Ok(mut req_queue) = request_queue.write() {
        req_queue.retain(|req| !req.is_marked_as_deleted());
    } else {
        warn!("Failed to lock request queue");
    }

    if let Ok(mut res_queue) = response_queue.write() {
        res_queue.retain(|res| !res.is_marked_as_deleted());
    } else {
        warn!("Failed to lock response queue");
    }
}
