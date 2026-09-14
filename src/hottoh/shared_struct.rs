use crate::hottoh::hottoh_structs::{DAT0Data, DAT1Data, DAT2Data, INFData};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

/// Number of write requests whose outcome is kept for `GET /api/request/{id}`
const REQUEST_HISTORY: usize = 100;

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Outcome of a write request
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestState {
    /// Waiting in the queue
    Pending,
    /// Sent to the stove, waiting for the answer
    Sent,
    /// The stove answered `OK;`
    Ok,
    /// The stove answered `ERR;<code>;`
    Error,
    /// No answer after all attempts
    Timeout,
}

/// Status of a write request, as returned by `GET /api/request/{id}`
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
    pub pending_writes: usize,
}

/// Shared state between the TCP worker and the HTTP API
#[derive(Debug, Default)]
pub struct SharedState {
    inf: INFData,
    dat0: DAT0Data,
    dat1: DAT1Data,
    dat2: DAT2Data,
    dat0_received: bool,
    dat1_received: bool,
    connection: ConnectionStatus,
    requests: HashMap<u32, RequestStatus>,
    request_order: VecDeque<u32>,
}

impl SharedState {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn set_pending_writes(&mut self, count: usize) {
        self.connection.pending_writes = count;
    }

    /// Registers a new write request as pending
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

    pub fn get_request(&self, request_id: u32) -> Option<&RequestStatus> {
        self.requests.get(&request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_history_is_bounded() {
        let mut state = SharedState::new();
        for id in 0..(REQUEST_HISTORY as u32 + 10) {
            state.track_request(id, "OnOff", "1");
        }
        assert!(state.get_request(0).is_none());
        assert!(state.get_request(REQUEST_HISTORY as u32 + 9).is_some());
        assert_eq!(state.requests.len(), REQUEST_HISTORY);
    }

    #[test]
    fn request_lifecycle() {
        let mut state = SharedState::new();
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
}
