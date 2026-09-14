//! TCP worker talking to the stove.
//!
//! Constraints from the Wifier 2.0 firmware (10.5.0):
//! - a single client at a time: the module closes its listening socket after `accept()`;
//! - one frame per `read()` on its side: frames sent back to back may be coalesced and are
//!   then both dropped silently, so only one request is in flight at a time;
//! - invalid frames get no answer at all, hence the per-request timeout.
//!
//! A single thread owns the connection. Writes queued by the HTTP API are sent first; the
//! INF and DAT pages are polled in between.

use crate::hottoh::hottoh_const::{Command, CommandType, write_error_message};
use crate::hottoh::hottoh_structs::{CommandData, WriteResult};
use crate::hottoh::shared_struct::{RequestState, SharedState};
use crate::hottoh::tcp_client_structs::{FrameBuffer, Request, Response};
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};
use std::thread;
use std::time::{Duration, Instant};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_POLL: Duration = Duration::from_millis(200);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
/// Minimum delay between two frames, so that the module never reads two at once
const MIN_FRAME_GAP: Duration = Duration::from_millis(100);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);
/// Consecutive unanswered requests before the connection is considered dead
const MAX_CONSECUTIVE_TIMEOUTS: u32 = 3;
/// Attempts for a write request (the stove settings are absolute values, resending is safe)
const MAX_WRITE_ATTEMPTS: u32 = 3;
/// A write still queued after this delay (stove unreachable) is abandoned
const WRITE_EXPIRY: Duration = Duration::from_secs(60);

/// Write request waiting to be sent
#[derive(Debug)]
pub struct QueuedWrite {
    request: Request,
    attempts: u32,
    queued_at: Instant,
}

impl QueuedWrite {
    pub fn new(request: Request) -> Self {
        Self {
            request,
            attempts: 0,
            queued_at: Instant::now(),
        }
    }
}

/// Queue of writes shared with the HTTP API
pub type WriteQueue = Arc<Mutex<VecDeque<QueuedWrite>>>;

/// Locks the shared state, recovering from a poisoned lock instead of panicking
pub fn state_write(state: &RwLock<SharedState>) -> RwLockWriteGuard<'_, SharedState> {
    state
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_queue(queue: &WriteQueue) -> std::sync::MutexGuard<'_, VecDeque<QueuedWrite>> {
    queue
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Pages polled in turn
const POLLED: [(Command, Option<&str>); 4] = [
    (Command::Inf, None),
    (Command::Dat, Some("0")),
    (Command::Dat, Some("1")),
    (Command::Dat, Some("2")),
];

/// What the worker is about to send
enum Job {
    Write(QueuedWrite),
    Poll(Request),
}

/// TCP client for the stove
pub struct TcpClient {
    address: String,
    poll_interval: Duration,
    writes: WriteQueue,
    state: Arc<RwLock<SharedState>>,
    request_id: Arc<AtomicU32>,
    running: Arc<AtomicBool>,
}

impl TcpClient {
    pub fn new(
        address: String,
        poll_interval: Duration,
        writes: WriteQueue,
        state: Arc<RwLock<SharedState>>,
        request_id: Arc<AtomicU32>,
        running: Arc<AtomicBool>,
    ) -> Self {
        Self {
            address,
            poll_interval,
            writes,
            state,
            request_id,
            running,
        }
    }

    /// Starts the worker thread
    pub fn start(self) -> thread::JoinHandle<()> {
        state_write(&self.state).set_stove_address(&self.address);
        thread::spawn(move || {
            while self.is_running() {
                match panic::catch_unwind(AssertUnwindSafe(|| self.run_connection())) {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        warn!("Stove connection: {}", e);
                        state_write(&self.state).set_last_error(e);
                    }
                    Err(panic) => {
                        error!("TCP worker panicked: {:?}", panic);
                        state_write(&self.state).set_last_error("internal error (panic)");
                    }
                }
                {
                    let mut state = state_write(&self.state);
                    state.set_connected(false);
                }
                if self.is_running() {
                    info!(
                        "Reconnecting to stove in {} s...",
                        RECONNECT_DELAY.as_secs()
                    );
                    self.sleep(RECONNECT_DELAY);
                }
            }
            info!("TCP worker stopped.");
        })
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Sleeps in small steps so that shutdown is not delayed
    fn sleep(&self, duration: Duration) {
        let end = Instant::now() + duration;
        while self.is_running() {
            let now = Instant::now();
            if now >= end {
                break;
            }
            thread::sleep((end - now).min(Duration::from_millis(100)));
        }
    }

    fn next_request_id(&self) -> u32 {
        self.request_id.fetch_add(1, Ordering::SeqCst) % 100_000
    }

    fn resolve(&self) -> Result<SocketAddr, String> {
        self.address
            .to_socket_addrs()
            .map_err(|e| format!("cannot resolve {}: {}", self.address, e))?
            .next()
            .ok_or_else(|| format!("cannot resolve {}", self.address))
    }

    /// Runs one connection until it fails or the application stops
    fn run_connection(&self) -> Result<(), String> {
        let addr = self.resolve()?;
        let mut stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
            .map_err(|e| format!("cannot connect to {}: {}", addr, e))?;
        stream
            .set_read_timeout(Some(READ_POLL))
            .and_then(|_| stream.set_write_timeout(Some(WRITE_TIMEOUT)))
            .and_then(|_| stream.set_nodelay(true))
            .map_err(|e| format!("socket setup failed: {}", e))?;
        info!("Connected to stove at {}", addr);
        state_write(&self.state).set_connected(true);

        let mut buffer = FrameBuffer::new();
        let mut consecutive_timeouts = 0;
        let mut poll_index = 0;
        let mut next_poll = Instant::now();
        let mut last_frame = Instant::now() - MIN_FRAME_GAP;

        while self.is_running() {
            let job = match self.next_write() {
                Some(write) => Job::Write(write),
                None if Instant::now() >= next_poll => {
                    let (command, page) = POLLED[poll_index];
                    poll_index = (poll_index + 1) % POLLED.len();
                    if poll_index == 0 {
                        next_poll = Instant::now() + self.poll_interval;
                    }
                    let params = page.map(|p| vec![p.to_string()]).unwrap_or_default();
                    Job::Poll(Request::new(
                        self.next_request_id(),
                        command,
                        CommandType::Read,
                        params,
                    ))
                }
                None => {
                    self.sleep(Duration::from_millis(50));
                    continue;
                }
            };

            let gap = last_frame.elapsed();
            if gap < MIN_FRAME_GAP {
                thread::sleep(MIN_FRAME_GAP - gap);
            }

            let request = match &job {
                Job::Write(write) => write.request.clone(),
                Job::Poll(request) => request.clone(),
            };
            if let Job::Write(_) = job {
                state_write(&self.state).update_request(
                    request.get_req_id(),
                    RequestState::Sent,
                    None,
                    None,
                );
            }

            let outcome = self.exchange(&mut stream, &mut buffer, &request);
            last_frame = Instant::now();

            match outcome {
                Ok(Some(response)) => {
                    consecutive_timeouts = 0;
                    state_write(&self.state).mark_response_received();
                    self.handle_response(&request, &response);
                }
                Ok(None) => {
                    consecutive_timeouts += 1;
                    warn!(
                        "No answer to request {} ({} {} {:?})",
                        request.get_req_id(),
                        request.get_command().as_str(),
                        request.get_command_type().as_str(),
                        request.get_params()
                    );
                    if let Job::Write(write) = job {
                        self.retry_or_fail(write, "no answer from stove");
                    }
                    if consecutive_timeouts >= MAX_CONSECUTIVE_TIMEOUTS {
                        return Err(format!(
                            "{} requests in a row without answer",
                            consecutive_timeouts
                        ));
                    }
                }
                Err(e) => {
                    if let Job::Write(write) = job {
                        self.retry_or_fail(write, &e);
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Takes the next write from the queue, dropping the expired ones
    fn next_write(&self) -> Option<QueuedWrite> {
        let mut queue = lock_queue(&self.writes);
        let mut result = None;
        while let Some(write) = queue.pop_front() {
            if write.queued_at.elapsed() > WRITE_EXPIRY {
                warn!(
                    "Dropping write request {}: stove unreachable for {} s",
                    write.request.get_req_id(),
                    WRITE_EXPIRY.as_secs()
                );
                state_write(&self.state).update_request(
                    write.request.get_req_id(),
                    RequestState::Timeout,
                    None,
                    Some("stove unreachable".into()),
                );
                continue;
            }
            result = Some(write);
            break;
        }
        state_write(&self.state).set_pending_writes(queue.len());
        result
    }

    /// Puts a failed write back at the head of the queue, or marks it as timed out
    fn retry_or_fail(&self, mut write: QueuedWrite, reason: &str) {
        write.attempts += 1;
        let id = write.request.get_req_id();
        if write.attempts < MAX_WRITE_ATTEMPTS {
            debug!("Retrying write request {} ({})", id, reason);
            state_write(&self.state).update_request(id, RequestState::Pending, None, None);
            lock_queue(&self.writes).push_front(write);
        } else {
            warn!("Write request {} failed: {}", id, reason);
            state_write(&self.state).update_request(
                id,
                RequestState::Timeout,
                None,
                Some(reason.to_string()),
            );
        }
    }

    /// Sends a request and waits for the answer with the same id.
    /// `Ok(None)` means no answer in time; `Err` means the connection is broken.
    fn exchange(
        &self,
        stream: &mut TcpStream,
        buffer: &mut FrameBuffer,
        request: &Request,
    ) -> Result<Option<Response>, String> {
        debug!(
            "Sending {}",
            String::from_utf8_lossy(&request.build_message()).trim_end()
        );
        stream
            .write_all(&request.build_message())
            .map_err(|e| format!("send failed: {}", e))?;

        let deadline = Instant::now() + RESPONSE_TIMEOUT;
        let mut chunk = [0u8; 1024];
        while Instant::now() < deadline && self.is_running() {
            match stream.read(&mut chunk) {
                Ok(0) => return Err("connection closed by the stove".into()),
                Ok(n) => {
                    buffer.extend(&chunk[..n]);
                    while let Some(frame) = buffer.next_frame() {
                        match Response::from_message(&frame) {
                            Ok(response) if response.get_req_id() == request.get_req_id() => {
                                return Ok(Some(response));
                            }
                            Ok(response) => debug!(
                                "Ignoring answer to request {} (late or unexpected)",
                                response.get_req_id()
                            ),
                            Err(e) => warn!(
                                "Invalid frame from stove: {} ({:?})",
                                e,
                                String::from_utf8_lossy(&frame).trim_end()
                            ),
                        }
                    }
                }
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(format!("receive failed: {}", e)),
            }
        }
        Ok(None)
    }

    /// Stores decoded data or the outcome of a write
    fn handle_response(&self, request: &Request, response: &Response) {
        let id = request.get_req_id();
        let is_write = request.get_command_type() == CommandType::Write;
        match response.command_data() {
            Ok(CommandData::Inf(data)) => state_write(&self.state).set_inf(data),
            Ok(CommandData::Dat0(data)) => state_write(&self.state).set_dat0(data),
            Ok(CommandData::Dat1(data)) => state_write(&self.state).set_dat1(data),
            Ok(CommandData::Dat2(data)) => state_write(&self.state).set_dat2(data),
            Ok(CommandData::Write(WriteResult::Ok)) => {
                info!("Write request {} {:?}: OK", id, request.get_params());
                state_write(&self.state).update_request(id, RequestState::Ok, None, None);
            }
            Ok(CommandData::Write(WriteResult::Error(code))) => {
                warn!(
                    "Write request {} {:?} refused by stove: ERR {} ({})",
                    id,
                    request.get_params(),
                    code,
                    write_error_message(code)
                );
                state_write(&self.state).update_request(
                    id,
                    RequestState::Error,
                    Some(code),
                    Some(write_error_message(code).to_string()),
                );
            }
            Err(e) => {
                warn!("Cannot decode answer to request {}: {}", id, e);
                if is_write {
                    state_write(&self.state).update_request(
                        id,
                        RequestState::Error,
                        None,
                        Some(format!("invalid answer: {}", e)),
                    );
                }
            }
        }
    }
}
