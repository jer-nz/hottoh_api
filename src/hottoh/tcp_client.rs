//! TCP worker talking to the stove.
//!
//! Constraints from the Wifier 2.0 firmware (10.5.0):
//! - a single client at a time: the module closes its listening socket after `accept()`;
//! - one frame per `read()` on its side: frames sent back to back may be coalesced and are
//!   then both dropped silently, so only one request is in flight at a time;
//! - invalid frames get no answer at all, hence the per-request timeout.
//!
//! A single thread owns the connection. Requests queued by the HTTP API (writes, and reads
//! made on demand) are sent first; the INF and DAT pages are polled in between.

use crate::hottoh::hottoh_const::{Command, CommandType, error_message};
use crate::hottoh::hottoh_structs::{CommandData, DAT0Data, INFData};
use crate::hottoh::module_data::Outcome;
use crate::hottoh::shared_struct::{Bridge, QueuedRequest, RequestState};
use crate::hottoh::tcp_client_structs::{FrameBuffer, Request, Response};
use log::{debug, error, info, warn};
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Consecutive unanswered requests before the connection is considered dead
const MAX_CONSECUTIVE_TIMEOUTS: u32 = 3;
/// Attempts for a queued request (settings are absolute values and reads have no side effect,
/// resending is safe)
const MAX_ATTEMPTS: u32 = 3;
/// Answer delay allowed to slow commands (Wi-Fi scan), as a multiple of the normal one
const SLOW_ANSWER_FACTOR: u32 = 4;
/// While the stove stays unreachable, one connection failure in this many is logged as a warning
const FAILURES_PER_WARNING: u32 = 60;

/// Delays used by the worker
#[derive(Debug, Clone)]
pub struct Timings {
    pub connect: Duration,
    /// Granularity of the socket reads (shutdown and deadlines are checked in between)
    pub read_poll: Duration,
    pub write: Duration,
    /// Time left to the stove to answer a request
    pub response: Duration,
    /// Minimum delay between two frames, so that the module never reads two at once
    pub min_frame_gap: Duration,
    pub reconnect: Duration,
    /// A request still queued after this delay (stove unreachable) is abandoned
    pub write_expiry: Duration,
}

impl Default for Timings {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(5),
            read_poll: Duration::from_millis(200),
            write: Duration::from_secs(5),
            response: Duration::from_secs(5),
            min_frame_gap: Duration::from_millis(100),
            reconnect: Duration::from_secs(5),
            write_expiry: Duration::from_secs(60),
        }
    }
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
    Queued(QueuedRequest),
    Poll(Request),
}

impl Job {
    fn request(&self) -> &Request {
        match self {
            Job::Queued(queued) => &queued.request,
            Job::Poll(request) => request,
        }
    }
}

/// Most parameters written to the log for one request (a weekly schedule has 337)
const LOGGED_PARAMS: usize = 8;

/// Parameters of a request as written to the log (secrets are hidden, long lists shortened)
fn shown_params(request: &Request) -> String {
    let params = request.get_params();
    if request.get_command().is_sensitive() {
        "[redacted]".into()
    } else if params.len() > LOGGED_PARAMS {
        format!(
            "{:?}... ({} values)",
            &params[..LOGGED_PARAMS],
            params.len()
        )
    } else {
        format!("{:?}", params)
    }
}

/// Frame as written to the log (secrets are hidden)
fn shown_frame(command: Command, frame: &[u8]) -> String {
    if command.is_sensitive() {
        format!("[{} frame redacted]", command.as_str())
    } else {
        String::from_utf8_lossy(frame).trim_end().to_string()
    }
}

/// Exits the process if the worker thread ends while the application still runs (a panic
/// outside `catch_unwind`): the service supervisor then restarts it, instead of the API
/// serving frozen data.
struct ExitOnUnexpectedEnd(Arc<Bridge>);

impl Drop for ExitOnUnexpectedEnd {
    fn drop(&mut self) {
        if self.0.is_running() {
            error!("TCP worker ended unexpectedly: exiting so that the service is restarted");
            std::process::exit(70);
        }
    }
}

/// TCP client for the stove
pub struct TcpClient {
    address: String,
    poll_interval: Duration,
    timings: Timings,
    bridge: Arc<Bridge>,
}

impl TcpClient {
    pub fn new(address: String, poll_interval: Duration, bridge: Arc<Bridge>) -> Self {
        Self {
            address,
            poll_interval,
            timings: Timings::default(),
            bridge,
        }
    }

    #[cfg(test)]
    pub fn with_timings(mut self, timings: Timings) -> Self {
        self.timings = timings;
        self
    }

    /// Starts the worker thread
    pub fn start(self) -> thread::JoinHandle<()> {
        self.bridge.state_mut().set_stove_address(&self.address);
        thread::Builder::new()
            .name("tcp-worker".into())
            .spawn(move || {
                let _guard = ExitOnUnexpectedEnd(Arc::clone(&self.bridge));
                self.run();
                info!("TCP worker stopped.");
            })
            .expect("cannot start the TCP worker thread")
    }

    /// Connects, serves the connection, and reconnects until the application stops
    fn run(&self) {
        let mut failures: u32 = 0;
        let mut down_since: Option<Instant> = None;

        while self.bridge.is_running() {
            let error = match self.connect() {
                Ok((stream, addr)) => {
                    match down_since.take() {
                        Some(since) => info!(
                            "Connected to stove at {} after {} failed attempts ({} s without link)",
                            addr,
                            failures,
                            since.elapsed().as_secs()
                        ),
                        None => info!("Connected to stove at {}", addr),
                    }
                    failures = 0;
                    self.bridge.state_mut().set_connected(true);
                    let outcome = panic::catch_unwind(AssertUnwindSafe(|| self.serve(stream)));
                    let error = match outcome {
                        Ok(Ok(())) => None,
                        Ok(Err(e)) => Some(e),
                        // Details were logged by the panic hook
                        Err(_) => Some("internal error (panic)".to_string()),
                    };
                    {
                        let mut state = self.bridge.state_mut();
                        state.set_connected(false);
                        if error.is_some() {
                            state.stats_mut().disconnections += 1;
                        }
                    }
                    if let Some(e) = &error {
                        warn!("Stove connection lost: {}", e);
                        down_since = Some(Instant::now());
                    }
                    error
                }
                Err(e) => {
                    failures += 1;
                    down_since.get_or_insert_with(Instant::now);
                    self.bridge.state_mut().stats_mut().connect_failures += 1;
                    if failures == 1 || failures.is_multiple_of(FAILURES_PER_WARNING) {
                        warn!("Stove unreachable ({} attempts): {}", failures, e);
                    } else {
                        debug!("Stove unreachable ({} attempts): {}", failures, e);
                    }
                    Some(e)
                }
            };
            if let Some(e) = error {
                self.bridge.state_mut().set_last_error(e);
            }
            self.drop_expired_requests();
            if self.bridge.is_running() {
                debug!(
                    "Reconnecting to stove in {} ms",
                    self.timings.reconnect.as_millis()
                );
                self.bridge.sleep(self.timings.reconnect);
            }
        }
    }

    fn resolve(&self) -> Result<SocketAddr, String> {
        self.address
            .to_socket_addrs()
            .map_err(|e| format!("cannot resolve {}: {}", self.address, e))?
            .next()
            .ok_or_else(|| format!("cannot resolve {}", self.address))
    }

    fn connect(&self) -> Result<(TcpStream, SocketAddr), String> {
        let addr = self.resolve()?;
        let stream = TcpStream::connect_timeout(&addr, self.timings.connect)
            .map_err(|e| format!("cannot connect to {}: {}", addr, e))?;
        stream
            .set_read_timeout(Some(self.timings.read_poll))
            .and_then(|_| stream.set_write_timeout(Some(self.timings.write)))
            .and_then(|_| stream.set_nodelay(true))
            .map_err(|e| format!("socket setup failed: {}", e))?;
        Ok((stream, addr))
    }

    /// Runs one connection until it fails or the application stops
    fn serve(&self, mut stream: TcpStream) -> Result<(), String> {
        let mut buffer = FrameBuffer::new();
        let mut consecutive_timeouts = 0;
        let mut poll_index = 0;
        let mut next_poll = Instant::now();
        let mut last_frame: Option<Instant> = None;

        while self.bridge.is_running() {
            let job = match self.next_queued() {
                Some(queued) => Job::Queued(queued),
                None if Instant::now() >= next_poll => {
                    let (command, page) = POLLED[poll_index];
                    poll_index = (poll_index + 1) % POLLED.len();
                    if poll_index == 0 {
                        next_poll = Instant::now() + self.poll_interval;
                    }
                    let params = page.map(|p| vec![p.to_string()]).unwrap_or_default();
                    Job::Poll(Request::new(
                        self.bridge.next_request_id(),
                        command,
                        CommandType::Read,
                        params,
                    ))
                }
                None => {
                    self.bridge.sleep(Duration::from_millis(50));
                    continue;
                }
            };

            if let Some(gap) = last_frame.map(|t| t.elapsed())
                && gap < self.timings.min_frame_gap
            {
                thread::sleep(self.timings.min_frame_gap - gap);
            }

            let request = job.request().clone();
            if let Job::Queued(_) = job {
                self.bridge.state_mut().update_request(
                    request.get_req_id(),
                    RequestState::Sent,
                    None,
                    None,
                );
            }

            if let Job::Queued(queued) = &job
                && !queued.expects_answer
            {
                let sent = self.send(&mut stream, &request);
                last_frame = Some(Instant::now());
                let mut state = self.bridge.state_mut();
                match &sent {
                    Ok(()) => {
                        info!(
                            "Request {} ({} {}) sent, no answer expected",
                            request.get_req_id(),
                            request.get_command().as_str(),
                            request.get_command_type().as_str()
                        );
                        state.update_request(request.get_req_id(), RequestState::Ok, None, None);
                        state.count_outcome(request.get_command_type(), &RequestState::Ok);
                    }
                    Err(e) => {
                        state.update_request(
                            request.get_req_id(),
                            RequestState::Timeout,
                            None,
                            Some(e.clone()),
                        );
                        state.count_outcome(request.get_command_type(), &RequestState::Timeout);
                    }
                }
                sent?;
                continue;
            }

            let outcome = self.exchange(&mut stream, &mut buffer, &request);
            last_frame = Some(Instant::now());
            if !self.bridge.is_running() {
                break;
            }

            match outcome {
                Ok(Some(response)) => {
                    consecutive_timeouts = 0;
                    match job {
                        Job::Queued(_) => self.handle_queued_answer(&request, &response),
                        Job::Poll(_) => self.handle_page(&request, &response),
                    }
                }
                Ok(None) => {
                    consecutive_timeouts += 1;
                    self.bridge.state_mut().stats_mut().timeouts += 1;
                    warn!(
                        "No answer to request {} ({} {} {}) within {} ms",
                        request.get_req_id(),
                        request.get_command().as_str(),
                        request.get_command_type().as_str(),
                        shown_params(&request),
                        self.timings.response.as_millis()
                    );
                    if let Job::Queued(queued) = job {
                        self.retry_or_fail(queued, "no answer from stove");
                    }
                    if consecutive_timeouts >= MAX_CONSECUTIVE_TIMEOUTS {
                        return Err(format!(
                            "{} requests in a row without answer",
                            consecutive_timeouts
                        ));
                    }
                }
                Err(e) => {
                    if let Job::Queued(queued) = job {
                        self.retry_or_fail(queued, &e);
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Takes the next request from the queue
    fn next_queued(&self) -> Option<QueuedRequest> {
        self.drop_expired_requests();
        self.bridge.queue().pop_front()
    }

    /// Abandons the requests queued for too long (stove unreachable)
    fn drop_expired_requests(&self) {
        let mut expired = Vec::new();
        self.bridge.queue().retain(|queued| {
            let alive = queued.queued_at.elapsed() <= self.timings.write_expiry;
            if !alive {
                expired.push((
                    queued.request.get_req_id(),
                    queued.request.get_command_type(),
                ));
            }
            alive
        });
        for (id, command_type) in expired {
            warn!(
                "Dropping request {}: not sent within {} s",
                id,
                self.timings.write_expiry.as_secs()
            );
            let mut state = self.bridge.state_mut();
            state.count_outcome(command_type, &RequestState::Timeout);
            state.update_request(
                id,
                RequestState::Timeout,
                None,
                Some("stove unreachable".into()),
            );
        }
    }

    /// Puts a failed request back at the head of the queue, or marks it as timed out
    fn retry_or_fail(&self, mut queued: QueuedRequest, reason: &str) {
        queued.attempts += 1;
        let id = queued.request.get_req_id();
        if queued.attempts < MAX_ATTEMPTS && queued.request.get_command().can_retry() {
            info!(
                "Retrying request {} (attempt {} failed: {})",
                id, queued.attempts, reason
            );
            self.bridge
                .state_mut()
                .update_request(id, RequestState::Pending, None, None);
            self.bridge.queue().push_front(queued);
        } else {
            warn!(
                "Request {} {} {} failed after {} attempts: {}",
                id,
                queued.request.get_command().as_str(),
                shown_params(&queued.request),
                queued.attempts,
                reason
            );
            let mut state = self.bridge.state_mut();
            state.count_outcome(queued.request.get_command_type(), &RequestState::Timeout);
            state.update_request(id, RequestState::Timeout, None, Some(reason.to_string()));
        }
    }

    fn send(&self, stream: &mut TcpStream, request: &Request) -> Result<(), String> {
        self.bridge.state_mut().stats_mut().requests += 1;
        stream
            .write_all(&request.build_message())
            .map_err(|e| format!("send failed: {}", e))
    }

    /// Sends a request and waits for the answer with the same id.
    /// `Ok(None)` means no answer in time; `Err` means the connection is broken.
    fn exchange(
        &self,
        stream: &mut TcpStream,
        buffer: &mut FrameBuffer,
        request: &Request,
    ) -> Result<Option<Response>, String> {
        let command = request.get_command();
        let sent = Instant::now();
        self.send(stream, request)?;

        let timeout = if command.is_slow() {
            self.timings.response * SLOW_ANSWER_FACTOR
        } else {
            self.timings.response
        };
        let deadline = sent + timeout;
        let mut chunk = [0u8; 1024];
        while Instant::now() < deadline && self.bridge.is_running() {
            match stream.read(&mut chunk) {
                Ok(0) => return Err("connection closed by the stove".into()),
                Ok(n) => {
                    buffer.extend(&chunk[..n]);
                    while let Some(frame) = buffer.next_frame() {
                        match Response::from_message(&frame) {
                            Ok(response) if response.get_req_id() == request.get_req_id() => {
                                let latency = sent.elapsed();
                                debug!(
                                    "{} -> {} ({} ms)",
                                    shown_frame(command, &request.build_message()),
                                    shown_frame(command, &frame),
                                    latency.as_millis()
                                );
                                let mut state = self.bridge.state_mut();
                                state.stats_mut().record_answer(latency);
                                state.mark_response_received();
                                return Ok(Some(response));
                            }
                            Ok(response) => {
                                self.bridge.state_mut().stats_mut().late_answers += 1;
                                debug!(
                                    "Ignoring answer to request {} while waiting for {}: {}",
                                    response.get_req_id(),
                                    request.get_req_id(),
                                    shown_frame(response.get_command(), &frame)
                                );
                            }
                            Err(e) => {
                                self.bridge.state_mut().stats_mut().invalid_frames += 1;
                                warn!(
                                    "Invalid frame from stove: {} ({:?})",
                                    e,
                                    String::from_utf8_lossy(&frame).trim_end()
                                );
                            }
                        }
                    }
                }
                Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(format!("receive failed: {}", e)),
            }
        }
        debug!(
            "{} -> no answer",
            shown_frame(command, &request.build_message())
        );
        Ok(None)
    }

    /// Stores the outcome of a queued request, and the answer of a read for its HTTP handler
    fn handle_queued_answer(&self, request: &Request, response: &Response) {
        let id = request.get_req_id();
        let command_type = request.get_command_type();
        let params = response.get_params();
        let outcome = Outcome::from_params(params);
        let (state_value, code, message) = match (&outcome, command_type) {
            (Some(Outcome::Ok), _) => (RequestState::Ok, None, None),
            (Some(Outcome::Error(code)), _) => (
                RequestState::Error,
                *code,
                Some(error_message(*code).to_string()),
            ),
            // A read answers with its data
            (None, CommandType::Read) => (RequestState::Ok, None, None),
            (None, _) => {
                warn!(
                    "Unexpected answer to request {} ({} {})",
                    id,
                    request.get_command().as_str(),
                    command_type.as_str()
                );
                self.bridge.state_mut().stats_mut().decode_errors += 1;
                (
                    RequestState::Error,
                    None,
                    Some("invalid answer from stove".to_string()),
                )
            }
        };
        match &state_value {
            RequestState::Ok if command_type == CommandType::Read => debug!(
                "Request {} ({} R): {} answer fields",
                id,
                request.get_command().as_str(),
                params.len()
            ),
            RequestState::Ok => info!(
                "Request {} ({} {} {}): OK",
                id,
                request.get_command().as_str(),
                command_type.as_str(),
                shown_params(request)
            ),
            _ => warn!(
                "Request {} ({} {} {}) refused by stove: ERR {} ({})",
                id,
                request.get_command().as_str(),
                command_type.as_str(),
                shown_params(request),
                code.map_or_else(|| "without code".into(), |c| c.to_string()),
                message.as_deref().unwrap_or_default()
            ),
        }
        let mut state = self.bridge.state_mut();
        state.count_outcome(command_type, &state_value);
        state.set_answer(id, params.to_vec());
        state.update_request(id, state_value, code, message);
    }

    /// Stores a polled page
    fn handle_page(&self, request: &Request, response: &Response) {
        let data = match response.command_data() {
            Ok(data) => data,
            Err(e) => {
                warn!(
                    "Cannot decode answer to request {}: {}",
                    request.get_req_id(),
                    e
                );
                self.bridge.state_mut().stats_mut().decode_errors += 1;
                return;
            }
        };

        let mut state = self.bridge.state_mut();
        match data {
            CommandData::Inf(data) => {
                log_inf_changes(state.inf_if_received(), &data);
                state.set_inf(data);
            }
            CommandData::Dat0(data) => {
                log_dat0_changes(state.dat0_if_received(), &data);
                state.set_dat0(data);
            }
            CommandData::Dat1(data) => state.set_dat1(data),
            CommandData::Dat2(data) => state.set_dat2(data),
        }
    }
}
/// Logs the module identity when first received, and its changes
fn log_inf_changes(old: Option<&INFData>, new: &INFData) {
    match old {
        None => info!(
            "Module {} firmware {}, Wi-Fi signal {}",
            new.hostname, new.version, new.signal
        ),
        Some(old) if old.hostname != new.hostname || old.version != new.version => info!(
            "Module changed: {} {} -> {} {}",
            old.hostname, old.version, new.hostname, new.version
        ),
        Some(_) => {}
    }
}

/// Logs the stove state when first received, then every change of its discrete fields
/// (state, modes, set points). Measures are only in the debug frame dumps.
fn log_dat0_changes(old: Option<&DAT0Data>, new: &DAT0Data) {
    let Some(old) = old else {
        info!(
            "Stove: state {:?}, on {}, eco {}, chrono mode {}, set point {} °C, power {} ({}..={}), \
             room {} °C, smoke {} °C, Modbus link {}",
            new.index_stove_state,
            new.index_stove_on,
            new.index_eco_mode,
            new.index_chrono_mode,
            new.index_ambient_t1_set,
            new.index_power_set,
            new.index_power_min,
            new.index_power_max,
            new.index_ambient_t1,
            new.index_smoke_t,
            new.index_valid
        );
        return;
    };
    macro_rules! log_changed {
        ($($field:ident),+ $(,)?) => {$(
            if old.$field != new.$field {
                info!("Stove {}: {:?} -> {:?}", stringify!($field), old.$field, new.$field);
            }
        )+};
    }
    log_changed!(
        index_valid,
        index_stove_state,
        index_stove_on,
        index_eco_mode,
        index_chrono_mode,
        index_ambient_t1_set,
        index_ambient_t2_set,
        index_water_set,
        index_power_set,
        index_fan_1_set,
        index_fan_2_set,
        index_fan_3_set,
        index_manufacturer,
        index_stove_type,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hottoh::hottoh_const::StoveCommands;
    use crate::hottoh::hottoh_structs::calculate_checksum;
    use crate::hottoh::shared_struct::Job as SharedJob;
    use crate::hottoh::shared_struct::RequestStatus;
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const DAT0: &str = "0;9;0;1;33;8;1;0;2;215;220;50;300;-15;0;0;0;0;0;0;0;1450;\
                        3;3;1;5;1200;3;3;5;0;0;0;0;0;0;";
    const DAT1: &str = "1;2;200;70;300;180;70;300;200;70;300;";
    const DAT2: &str = "2;0;1;3;0;0;450;600;300;800;-10;0;0;0;0;0;0;0;195;200;70;300;";

    fn frame(id: u32, command: &str, kind: &str, params: &str) -> Vec<u8> {
        let body = format!(
            "{:05}C---{:04X}{}{}{}",
            id,
            params.len(),
            command,
            kind,
            params
        );
        format!("#{}{}\n", body, calculate_checksum(&body)).into_bytes()
    }

    /// What the fake module does with a request
    enum Reply {
        /// Sends these bytes at once
        Send(Vec<u8>),
        /// Sends these bytes in two TCP writes, 20 ms apart
        Split(Vec<u8>),
        Silent,
        Close,
    }

    /// Answer of a healthy stove
    fn answer(request: &Response) -> Vec<u8> {
        let id = request.get_req_id();
        let command = request.get_command();
        let kind = request.get_command_type();
        match (command, kind) {
            (Command::Inf, _) => frame(id, "INF", "R", "HOTTOH32;10.5.0;196;"),
            (Command::Dat, CommandType::Read) => {
                let page = match request.get_params() {
                    [p] if p == "0" => DAT0,
                    [p] if p == "1" => DAT1,
                    _ => DAT2,
                };
                frame(id, "DAT", "R", page)
            }
            (Command::Clk, CommandType::Read) => frame(id, "CLK", "R", "1757930400;1757937600;"),
            (Command::Pin, CommandType::Read) => frame(id, "PIN", "R", r#"\"12345\";"#),
            (Command::Met, CommandType::Read) => frame(id, "MET", "R", ""),
            (_, _) => frame(id, command.as_str(), kind.as_str(), "OK;"),
        }
    }

    /// Fake Wi-Fi module on a local port: one client at a time, `reply` gets the connection
    /// number (from 0) and the request. Returns the address and the connection counter.
    fn fake_stove<F>(reply: F) -> (String, Arc<AtomicUsize>)
    where
        F: Fn(usize, &Response) -> Reply + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let connections = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&connections);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let connection = counter.fetch_add(1, Ordering::SeqCst);
                let mut buffer = FrameBuffer::new();
                let mut chunk = [0u8; 256];
                'connection: loop {
                    let n = match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => n,
                    };
                    buffer.extend(&chunk[..n]);
                    while let Some(frame) = buffer.next_frame() {
                        let request = Response::from_message(&frame).expect("invalid request");
                        let sent = match reply(connection, &request) {
                            Reply::Send(bytes) => stream.write_all(&bytes),
                            Reply::Split(bytes) => {
                                let (head, tail) = bytes.split_at(bytes.len() / 2);
                                stream.write_all(head).and_then(|_| {
                                    thread::sleep(Duration::from_millis(20));
                                    stream.write_all(tail)
                                })
                            }
                            Reply::Silent => Ok(()),
                            Reply::Close => break 'connection,
                        };
                        if sent.is_err() {
                            break 'connection;
                        }
                    }
                }
            }
        });
        (address, connections)
    }

    fn fast_timings() -> Timings {
        Timings {
            connect: Duration::from_secs(1),
            read_poll: Duration::from_millis(10),
            write: Duration::from_secs(1),
            response: Duration::from_millis(150),
            min_frame_gap: Duration::from_millis(1),
            reconnect: Duration::from_millis(50),
            write_expiry: Duration::from_secs(60),
        }
    }

    struct Worker {
        bridge: Arc<Bridge>,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl Worker {
        fn start(address: String, timings: Timings, setup: impl FnOnce(&Bridge)) -> Self {
            let bridge = Arc::new(Bridge::new());
            setup(&bridge);
            let handle = TcpClient::new(address, Duration::from_millis(10), Arc::clone(&bridge))
                .with_timings(timings)
                .start();
            Self {
                bridge,
                handle: Some(handle),
            }
        }

        fn wait_for(&self, what: &str, condition: impl Fn(&Bridge) -> bool) {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !condition(&self.bridge) {
                assert!(Instant::now() < deadline, "timeout waiting for {}", what);
                thread::sleep(Duration::from_millis(10));
            }
        }

        fn request(&self, id: u32) -> RequestStatus {
            self.bridge.state().get_request(id).unwrap().clone()
        }
    }

    impl Drop for Worker {
        fn drop(&mut self) {
            self.bridge.stop();
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }

    fn request_done(id: u32) -> impl Fn(&Bridge) -> bool {
        move |b| {
            !matches!(
                b.state().get_request(id).map(|r| r.status.clone()),
                Some(RequestState::Pending | RequestState::Sent)
            )
        }
    }

    #[test]
    fn polls_every_page() {
        let (address, connections) = fake_stove(|_, r| Reply::Send(answer(r)));
        let worker = Worker::start(address, fast_timings(), |_| {});
        worker.wait_for("all pages", |b| {
            let s = b.state();
            s.inf_if_received().is_some()
                && s.dat0_if_received().is_some()
                && s.dat1_if_received().is_some()
                && s.get_dat2().index_fan_1_speed == 3
        });
        let state = worker.bridge.state();
        assert_eq!(state.get_inf().version, "10.5.0");
        assert_eq!(state.get_dat0().index_ambient_t1, 21.5);
        assert!(state.connection().connected);
        assert_eq!(state.connection().stats.timeouts, 0);
        assert_eq!(connections.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn write_is_sent_first_and_acknowledged() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        let (address, _) = fake_stove(move |_, r| {
            log.lock()
                .unwrap()
                .push((r.get_command_type(), r.get_params().to_vec()));
            Reply::Send(answer(r))
        });
        let mut id = 0;
        let worker = Worker::start(address, fast_timings(), |b| {
            id = b.queue_write(StoveCommands::PowerLevel, 3).unwrap();
        });
        worker.wait_for("write outcome", request_done(id));
        let status = worker.request(id);
        assert_eq!(status.status, RequestState::Ok);
        assert_eq!(status.attempts, 1);
        assert_eq!(
            seen.lock().unwrap()[0],
            (CommandType::Write, vec!["2".to_string(), "3".to_string()])
        );
        assert_eq!(worker.bridge.state().connection().stats.writes_ok, 1);
    }

    #[test]
    fn refused_write_reports_the_stove_error() {
        let (address, _) = fake_stove(|_, r| match r.get_command_type() {
            CommandType::Write => Reply::Send(frame(r.get_req_id(), "DAT", "W", "ERR;17;")),
            _ => Reply::Send(answer(r)),
        });
        let mut id = 0;
        let worker = Worker::start(address, fast_timings(), |b| {
            id = b.queue_write(StoveCommands::ChronoOnOff, 1).unwrap();
        });
        worker.wait_for("write outcome", request_done(id));
        let status = worker.request(id);
        assert_eq!(status.status, RequestState::Error);
        assert_eq!(status.error_code, Some(17));
        assert_eq!(worker.bridge.state().connection().stats.writes_refused, 1);
    }

    #[test]
    fn split_noisy_and_late_answers_are_handled() {
        let (address, _) = fake_stove(|_, r| {
            let mut bytes = b"noise\n#garbage\n".to_vec();
            // An answer to an old request comes first
            bytes.extend(frame(r.get_req_id().wrapping_sub(1), "INF", "R", "X;1;2;"));
            bytes.extend(answer(r));
            Reply::Split(bytes)
        });
        let worker = Worker::start(address, fast_timings(), |_| {});
        worker.wait_for("DAT pages", |b| b.state().dat1_if_received().is_some());
        let stats = worker.bridge.state().connection().stats.clone();
        assert_eq!(stats.timeouts, 0);
        assert!(stats.invalid_frames > 0);
        assert!(stats.late_answers > 0);
        assert_eq!(stats.decode_errors, 0);
    }

    #[test]
    fn silent_stove_causes_retries_then_reconnection() {
        let (address, connections) = fake_stove(|connection, r| match connection {
            0 => Reply::Silent,
            _ => Reply::Send(answer(r)),
        });
        let mut id = 0;
        let worker = Worker::start(address, fast_timings(), |b| {
            id = b.queue_write(StoveCommands::EcoMode, 1).unwrap();
        });
        worker.wait_for("write outcome", request_done(id));
        worker.wait_for("data after reconnection", |b| {
            b.state().dat0_if_received().is_some()
        });
        let status = worker.request(id);
        assert_eq!(status.status, RequestState::Timeout);
        assert_eq!(status.attempts, MAX_ATTEMPTS);
        let state = worker.bridge.state();
        assert_eq!(state.connection().stats.writes_failed, 1);
        assert_eq!(state.connection().stats.disconnections, 1);
        assert!(state.connection().connected);
        assert_eq!(connections.load(Ordering::SeqCst), 2);
    }

    fn job(command: Command, command_type: CommandType, expects_answer: bool) -> SharedJob {
        SharedJob {
            command,
            command_type,
            params: vec![],
            label: command.as_str(),
            shown_value: String::new(),
            expects_answer,
        }
    }

    #[test]
    fn reads_on_demand_keep_their_answer() {
        let (address, _) = fake_stove(|_, r| Reply::Send(answer(r)));
        let (mut clock, mut datalog, mut pin) = (0, 0, 0);
        let worker = Worker::start(address, fast_timings(), |b| {
            clock = b
                .queue_job(job(Command::Clk, CommandType::Read, true))
                .unwrap();
            datalog = b
                .queue_job(job(Command::Met, CommandType::Read, true))
                .unwrap();
            pin = b
                .queue_job(job(Command::Pin, CommandType::Read, true))
                .unwrap();
        });
        for id in [clock, datalog, pin] {
            worker.wait_for("read outcome", request_done(id));
            assert_eq!(worker.request(id).status, RequestState::Ok);
        }
        assert_eq!(worker.request(clock).answer, ["1757930400", "1757937600"]);
        assert!(worker.request(datalog).answer.is_empty());
        assert_eq!(worker.request(pin).answer, [r#"\"12345\""#]);
        let stats = worker.bridge.state().connection().stats.clone();
        assert_eq!((stats.reads_ok, stats.writes_ok), (3, 0));
        // The answer is never exposed by /api/request/{id}
        let json = serde_json::to_string(&worker.request(pin)).unwrap();
        assert!(!json.contains("12345"));
    }

    #[test]
    fn command_without_answer_is_not_retried() {
        let seen = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&seen);
        let (address, _) = fake_stove(move |_, r| match r.get_command() {
            Command::Rst => {
                count.fetch_add(1, Ordering::SeqCst);
                Reply::Silent
            }
            _ => Reply::Send(answer(r)),
        });
        let mut id = 0;
        let worker = Worker::start(address, fast_timings(), |b| {
            id = b
                .queue_job(job(Command::Rst, CommandType::Execute, false))
                .unwrap();
        });
        worker.wait_for("restart outcome", request_done(id));
        worker.wait_for("data after the restart", |b| {
            b.state().dat0_if_received().is_some()
        });
        assert_eq!(worker.request(id).status, RequestState::Ok);
        assert_eq!(seen.load(Ordering::SeqCst), 1);
        assert_eq!(worker.bridge.state().connection().stats.timeouts, 0);
    }

    #[test]
    fn unanswered_scan_is_not_repeated() {
        let scans = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&scans);
        let (address, _) = fake_stove(move |_, r| match r.get_command() {
            Command::Scn => {
                count.fetch_add(1, Ordering::SeqCst);
                Reply::Silent
            }
            _ => Reply::Send(answer(r)),
        });
        let mut id = 0;
        let worker = Worker::start(address, fast_timings(), |b| {
            id = b
                .queue_job(job(Command::Scn, CommandType::Read, true))
                .unwrap();
        });
        worker.wait_for("scan outcome", request_done(id));
        assert_eq!(worker.request(id).status, RequestState::Timeout);
        assert_eq!(scans.load(Ordering::SeqCst), 1);
        assert_eq!(worker.bridge.state().connection().stats.reads_failed, 1);
    }

    #[test]
    fn refused_read_keeps_error_code_and_answer() {
        let (address, _) = fake_stove(|_, r| match r.get_command() {
            Command::Tmz => Reply::Send(frame(r.get_req_id(), "TMZ", "R", r#"ERR;8;\"Mars\";"#)),
            _ => Reply::Send(answer(r)),
        });
        let mut id = 0;
        let worker = Worker::start(address, fast_timings(), |b| {
            id = b
                .queue_job(job(Command::Tmz, CommandType::Read, true))
                .unwrap();
        });
        worker.wait_for("read outcome", request_done(id));
        let status = worker.request(id);
        assert_eq!(status.status, RequestState::Error);
        assert_eq!(status.error_code, Some(8));
        assert_eq!(status.answer.len(), 3);
        assert_eq!(worker.bridge.state().connection().stats.reads_refused, 1);
    }

    #[test]
    fn secrets_are_not_logged() {
        let request = Request::new(1, Command::Pin, CommandType::Write, vec!["x".into()]);
        assert_eq!(shown_params(&request), "[redacted]");
        assert!(!shown_frame(Command::Pin, &request.build_message()).contains('x'));
        let request = Request::new(1, Command::Dat, CommandType::Write, vec!["2".into()]);
        assert_eq!(shown_params(&request), "[\"2\"]");
        let request = Request::new(1, Command::Sch, CommandType::Write, vec!["0".into(); 337]);
        assert!(shown_params(&request).ends_with("... (337 values)"));
    }

    #[test]
    fn closed_connection_is_reopened() {
        let (address, connections) = fake_stove(|connection, r| match connection {
            0 | 1 => Reply::Close,
            _ => Reply::Send(answer(r)),
        });
        let worker = Worker::start(address, fast_timings(), |_| {});
        worker.wait_for("data", |b| b.state().dat0_if_received().is_some());
        let state = worker.bridge.state();
        assert_eq!(state.connection().stats.disconnections, 2);
        assert_eq!(state.connection().connections, 3);
        assert!(state.connection().last_error.is_some());
        assert_eq!(connections.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn unreachable_stove_expires_queued_writes() {
        // A port that was just released: connections are refused
        let address = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().to_string()
        };
        let timings = Timings {
            write_expiry: Duration::from_millis(100),
            ..fast_timings()
        };
        let mut id = 0;
        let worker = Worker::start(address, timings, |b| {
            id = b.queue_write(StoveCommands::OnOff, 0).unwrap();
        });
        worker.wait_for("write expiry", request_done(id));
        assert_eq!(worker.request(id).status, RequestState::Timeout);
        let state = worker.bridge.state();
        assert!(!state.connection().connected);
        assert!(state.connection().stats.connect_failures > 0);
        assert_eq!(state.connection().stats.writes_failed, 1);
        assert!(worker.bridge.queue().is_empty());
    }
}
