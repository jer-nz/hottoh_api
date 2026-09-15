//! State shared between the TCP worker, the HTTP API and the statistics thread.

use crate::hottoh::hottoh_const::{Command, CommandType, StoveCommands};
use crate::hottoh::hottoh_structs::{DAT0Data, DAT1Data, DAT2Data, INFData, now};
use crate::hottoh::tcp_client_structs::Request;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use std::time::{Duration, Instant};

/// Number of queued requests whose outcome is kept for `GET /api/request/{id}`
const REQUEST_HISTORY: usize = 100;

/// Requests waiting for the stove beyond this count are refused (HTTP 503)
pub const MAX_QUEUED: usize = 32;

/// Outcome of a queued request (write, or read made on demand)
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestState {
    /// Waiting in the queue
    Pending,
    /// Sent to the stove, waiting for the answer
    Sent,
    /// The stove answered `OK;` (or the requested data)
    Ok,
    /// The stove answered `ERR;<code>;`
    Error,
    /// No answer after all attempts
    Timeout,
}

/// Status of a queued request, as returned by `GET /api/request/{id}`
#[derive(Debug, Serialize, Clone)]
pub struct RequestStatus {
    pub request_id: u32,
    pub command: String,
    pub value: String,
    pub status: RequestState,
    /// Error code from the stove (`ERR;<code>;`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub attempts: u32,
    pub created_at: String,
    pub updated_at: String,
    /// Parameters of the answer, decoded by the HTTP handler that queued a read. Never
    /// serialized: some answers are secrets (PIN).
    #[serde(skip)]
    pub answer: Vec<String>,
}

/// Counters since start, exposed in `GET /api/status` and logged periodically
#[derive(Debug, Serialize, Clone, Default)]
pub struct Stats {
    /// Frames sent to the stove (reads and writes, retries included)
    pub requests: u64,
    /// Answers matching their request
    pub answers: u64,
    /// Requests left without answer
    pub timeouts: u64,
    /// Received frames rejected by the parser (CRC, length, format)
    pub invalid_frames: u64,
    /// Valid frames answering an older request
    pub late_answers: u64,
    /// Answers whose content could not be decoded
    pub decode_errors: u64,
    pub writes_ok: u64,
    /// Writes refused by the stove (`ERR;<code>;`)
    pub writes_refused: u64,
    /// Writes abandoned (no answer after all attempts, or expired in the queue)
    pub writes_failed: u64,
    /// Reads made on demand (schedule, clock, data logger...) answered with data
    pub reads_ok: u64,
    /// Reads made on demand answered with `ERR`
    pub reads_refused: u64,
    /// Reads made on demand abandoned (no answer, or expired in the queue)
    pub reads_failed: u64,
    /// Established connections that were lost
    pub disconnections: u64,
    /// Connection attempts that failed
    pub connect_failures: u64,
    /// Sum of the answer delays, for averages
    pub latency_total_ms: u64,
    pub latency_max_ms: u64,
}

impl Stats {
    pub fn record_answer(&mut self, latency: Duration) {
        let ms = u64::try_from(latency.as_millis()).unwrap_or(u64::MAX);
        self.answers += 1;
        self.latency_total_ms = self.latency_total_ms.saturating_add(ms);
        self.latency_max_ms = self.latency_max_ms.max(ms);
    }
}

/// State of the TCP link with the stove, as returned by `GET /api/status`
#[derive(Debug, Serialize, Clone, Default)]
pub struct ConnectionStatus {
    pub stove_address: String,
    pub connected: bool,
    /// Time of the last valid answer from the stove
    pub last_response_at: Option<String>,
    /// Last connection or communication error (kept after recovery, see `last_error_at`)
    pub last_error: Option<String>,
    pub last_error_at: Option<String>,
    /// Successful TCP connections since start (1 = never reconnected)
    pub connections: u64,
    pub stats: Stats,
}

/// Data and request tracking, behind the lock of [`Bridge`]
#[derive(Debug, Default)]
pub struct SharedState {
    inf: INFData,
    dat0: DAT0Data,
    dat1: DAT1Data,
    dat2: DAT2Data,
    inf_received: bool,
    dat0_received: bool,
    dat1_received: bool,
    connection: ConnectionStatus,
    requests: HashMap<u32, RequestStatus>,
    request_order: VecDeque<u32>,
}

impl SharedState {
    pub fn get_inf(&self) -> &INFData {
        &self.inf
    }

    pub fn get_dat0(&self) -> &DAT0Data {
        &self.dat0
    }

    pub fn get_dat1(&self) -> &DAT1Data {
        &self.dat1
    }

    pub fn get_dat2(&self) -> &DAT2Data {
        &self.dat2
    }

    /// INF data, only once a real answer has been received
    pub fn inf_if_received(&self) -> Option<&INFData> {
        self.inf_received.then_some(&self.inf)
    }

    /// DAT0 data, only once a real page has been received (used for range checks)
    pub fn dat0_if_received(&self) -> Option<&DAT0Data> {
        self.dat0_received.then_some(&self.dat0)
    }

    /// DAT1 data, only once a real page has been received (used for range checks)
    pub fn dat1_if_received(&self) -> Option<&DAT1Data> {
        self.dat1_received.then_some(&self.dat1)
    }

    pub fn set_inf(&mut self, inf: INFData) {
        self.inf = inf;
        self.inf_received = true;
    }

    pub fn set_dat0(&mut self, dat0: DAT0Data) {
        self.dat0 = dat0;
        self.dat0_received = true;
    }

    pub fn set_dat1(&mut self, dat1: DAT1Data) {
        self.dat1 = dat1;
        self.dat1_received = true;
    }

    pub fn set_dat2(&mut self, dat2: DAT2Data) {
        self.dat2 = dat2;
    }

    pub fn connection(&self) -> &ConnectionStatus {
        &self.connection
    }

    pub fn stats_mut(&mut self) -> &mut Stats {
        &mut self.connection.stats
    }

    pub fn set_stove_address(&mut self, address: &str) {
        self.connection.stove_address = address.to_string();
    }

    pub fn set_connected(&mut self, connected: bool) {
        if connected && !self.connection.connected {
            self.connection.connections += 1;
        }
        self.connection.connected = connected;
    }

    pub fn set_last_error(&mut self, error: impl Into<String>) {
        self.connection.last_error = Some(error.into());
        self.connection.last_error_at = Some(now());
    }

    pub fn mark_response_received(&mut self) {
        self.connection.last_response_at = Some(now());
    }

    /// Registers a new queued request as pending
    pub fn track_request(&mut self, request_id: u32, command: &str, value: &str) {
        let timestamp = now();
        let status = RequestStatus {
            request_id,
            command: command.to_string(),
            value: value.to_string(),
            status: RequestState::Pending,
            error_code: None,
            message: None,
            attempts: 0,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            answer: Vec::new(),
        };
        if self.requests.insert(request_id, status).is_none() {
            self.request_order.push_back(request_id);
        }
        while self.request_order.len() > REQUEST_HISTORY {
            if let Some(old) = self.request_order.pop_front() {
                self.requests.remove(&old);
            }
        }
    }

    /// Updates the state of a tracked request (ignored if it left the history)
    pub fn update_request(
        &mut self,
        request_id: u32,
        state: RequestState,
        error_code: Option<i32>,
        message: Option<String>,
    ) {
        if let Some(status) = self.requests.get_mut(&request_id) {
            if state == RequestState::Sent {
                status.attempts += 1;
            }
            status.status = state;
            status.error_code = error_code;
            status.message = message;
            status.updated_at = now();
        }
    }

    /// Keeps the parameters of the answer to a tracked request
    pub fn set_answer(&mut self, request_id: u32, answer: Vec<String>) {
        if let Some(status) = self.requests.get_mut(&request_id) {
            status.answer = answer;
        }
    }

    /// Counts the outcome of a queued request in the write or read counters
    pub fn count_outcome(&mut self, command_type: CommandType, state: &RequestState) {
        let stats = &mut self.connection.stats;
        let counter = match (command_type, state) {
            (CommandType::Read, RequestState::Ok) => &mut stats.reads_ok,
            (CommandType::Read, RequestState::Error) => &mut stats.reads_refused,
            (CommandType::Read, _) => &mut stats.reads_failed,
            (_, RequestState::Ok) => &mut stats.writes_ok,
            (_, RequestState::Error) => &mut stats.writes_refused,
            (_, _) => &mut stats.writes_failed,
        };
        *counter += 1;
    }

    pub fn get_request(&self, request_id: u32) -> Option<&RequestStatus> {
        self.requests.get(&request_id)
    }
}

/// Request waiting to be sent
#[derive(Debug)]
pub struct QueuedRequest {
    pub request: Request,
    pub attempts: u32,
    pub queued_at: Instant,
    /// `false` for a command the module never answers (restart): it is `ok` once sent
    pub expects_answer: bool,
}

/// Request to queue, see [`Bridge::queue`]
#[derive(Debug, Clone)]
pub struct Job {
    pub command: Command,
    pub command_type: CommandType,
    pub params: Vec<String>,
    /// Name shown in `/api/request/{id}` and in the log
    pub label: &'static str,
    /// Value shown in `/api/request/{id}` (never a secret)
    pub shown_value: String,
    pub expects_answer: bool,
}

/// Everything shared between threads. Locks are never held across a blocking call; when both
/// are needed, the queue is locked before the state.
#[derive(Debug)]
pub struct Bridge {
    state: RwLock<SharedState>,
    queue: Mutex<VecDeque<QueuedRequest>>,
    next_id: AtomicU32,
    running: AtomicBool,
    started: Instant,
    started_at: String,
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridge {
    pub fn new() -> Self {
        Self {
            state: RwLock::default(),
            queue: Mutex::default(),
            next_id: AtomicU32::new(1),
            running: AtomicBool::new(true),
            started: Instant::now(),
            started_at: now(),
        }
    }

    /// Reads the state, recovering from a poisoned lock instead of panicking
    pub fn state(&self) -> RwLockReadGuard<'_, SharedState> {
        self.state.read().unwrap_or_else(|p| p.into_inner())
    }

    /// Modifies the state, recovering from a poisoned lock instead of panicking
    pub fn state_mut(&self) -> RwLockWriteGuard<'_, SharedState> {
        self.state.write().unwrap_or_else(|p| p.into_inner())
    }

    /// Queue of requests waiting for the stove (writes and reads made on demand)
    pub fn queue(&self) -> MutexGuard<'_, VecDeque<QueuedRequest>> {
        self.queue.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Next frame id (5 digits on the wire)
    pub fn next_request_id(&self) -> u32 {
        self.next_id.fetch_add(1, Ordering::SeqCst) % 100_000
    }

    /// Queues a `DAT W` setting and tracks its outcome. Returns `None` when the queue is full.
    pub fn queue_write(&self, command: StoveCommands, value: i32) -> Option<u32> {
        self.queue_job(Job {
            command: Command::Dat,
            command_type: CommandType::Write,
            params: vec![(command as u32).to_string(), value.to_string()],
            label: command.name(),
            shown_value: value.to_string(),
            expects_answer: true,
        })
    }

    /// Queues any request and tracks its outcome. Returns `None` when the queue is full.
    pub fn queue_job(&self, job: Job) -> Option<u32> {
        let mut queue = self.queue();
        if queue.len() >= MAX_QUEUED {
            return None;
        }
        let request_id = self.next_request_id();
        self.state_mut()
            .track_request(request_id, job.label, &job.shown_value);
        queue.push_back(QueuedRequest {
            request: Request::new(request_id, job.command, job.command_type, job.params),
            attempts: 0,
            queued_at: Instant::now(),
            expects_answer: job.expects_answer,
        });
        Some(request_id)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Asks the background threads to stop
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Sleeps in small steps, returning early when the application stops
    pub fn sleep(&self, duration: Duration) {
        let end = Instant::now() + duration;
        while self.is_running() {
            let now = Instant::now();
            if now >= end {
                break;
            }
            thread::sleep((end - now).min(Duration::from_millis(100)));
        }
    }

    pub fn uptime(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn started_at(&self) -> &str {
        &self.started_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_history_is_bounded() {
        let mut state = SharedState::default();
        for id in 0..(REQUEST_HISTORY as u32 + 10) {
            state.track_request(id, "OnOff", "1");
        }
        assert!(state.get_request(0).is_none());
        assert!(state.get_request(REQUEST_HISTORY as u32 + 9).is_some());
        assert_eq!(state.requests.len(), REQUEST_HISTORY);
    }

    #[test]
    fn request_lifecycle() {
        let mut state = SharedState::default();
        state.track_request(7, "PowerLevel", "3");
        state.update_request(7, RequestState::Sent, None, None);
        state.update_request(
            7,
            RequestState::Error,
            Some(17),
            Some("value out of range".into()),
        );
        let status = state.get_request(7).unwrap();
        assert_eq!(status.status, RequestState::Error);
        assert_eq!(status.error_code, Some(17));
        assert_eq!(status.attempts, 1);
    }

    #[test]
    fn write_queue_is_bounded() {
        let bridge = Bridge::new();
        for _ in 0..MAX_QUEUED {
            let id = bridge.queue_write(StoveCommands::PowerLevel, 3).unwrap();
            assert_eq!(
                bridge.state().get_request(id).unwrap().status,
                RequestState::Pending
            );
        }
        assert!(bridge.queue_write(StoveCommands::PowerLevel, 3).is_none());
        assert_eq!(bridge.queue().len(), MAX_QUEUED);
    }

    #[test]
    fn outcomes_are_counted_by_kind() {
        let mut state = SharedState::default();
        state.count_outcome(CommandType::Write, &RequestState::Ok);
        state.count_outcome(CommandType::Execute, &RequestState::Timeout);
        state.count_outcome(CommandType::Read, &RequestState::Ok);
        state.count_outcome(CommandType::Read, &RequestState::Error);
        let stats = &state.connection().stats;
        assert_eq!((stats.writes_ok, stats.writes_failed), (1, 1));
        assert_eq!(
            (stats.reads_ok, stats.reads_refused, stats.reads_failed),
            (1, 1, 0)
        );
    }

    #[test]
    fn latency_is_accumulated() {
        let mut stats = Stats::default();
        stats.record_answer(Duration::from_millis(40));
        stats.record_answer(Duration::from_millis(100));
        assert_eq!(stats.answers, 2);
        assert_eq!(stats.latency_total_ms, 140);
        assert_eq!(stats.latency_max_ms, 100);
    }
}
