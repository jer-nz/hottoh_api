//! History of the stove alarms (ignition failure, no pellets, power cut...), built from the state
//! polled in DAT page 0 and kept in a JSON file so that it survives restarts.

use crate::hottoh::hottoh_const::StoveState;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::path::Path;

/// Alarms kept in the history
const KEPT: usize = 100;

/// One alarm: the stove stayed in an alarm state from `started_at` to `ended_at`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct AlarmEvent {
    /// Decoded state ("IgnitionFailed", "NoPellet", ..., "Unknown")
    pub state: String,
    /// Raw state register
    pub state_raw: u16,
    pub started_at: String,
    /// `None` while the alarm is still active
    pub ended_at: Option<String>,
}

/// Alarm history, oldest first
#[derive(Debug, Default)]
pub struct AlarmLog {
    events: VecDeque<AlarmEvent>,
}

impl AlarmLog {
    pub fn from_events(events: Vec<AlarmEvent>) -> Self {
        let mut events: VecDeque<AlarmEvent> = events.into();
        while events.len() > KEPT {
            events.pop_front();
        }
        Self { events }
    }

    /// Takes the current state of the stove into account. Returns true when the history changed.
    pub fn update(&mut self, state_raw: u16, now: &str) -> bool {
        let alarm = StoveState::is_alarm(state_raw);
        match self.events.back_mut().filter(|e| e.ended_at.is_none()) {
            Some(open) if open.state_raw == state_raw => return false,
            Some(open) => open.ended_at = Some(now.to_string()),
            None if !alarm => return false,
            None => {}
        }
        if alarm {
            self.events.push_back(AlarmEvent {
                state: StoveState::from(state_raw).to_string(),
                state_raw,
                started_at: now.to_string(),
                ended_at: None,
            });
            if self.events.len() > KEPT {
                self.events.pop_front();
            }
        }
        true
    }

    /// Alarm still active
    pub fn current(&self) -> Option<&AlarmEvent> {
        self.events.back().filter(|e| e.ended_at.is_none())
    }

    pub fn newest_first(&self) -> Vec<&AlarmEvent> {
        self.events.iter().rev().collect()
    }

    pub fn events(&self) -> Vec<AlarmEvent> {
        self.events.iter().cloned().collect()
    }
}

/// Reads the history file; a missing or unreadable file gives an empty history
pub fn load(path: &Path) -> Vec<AlarmEvent> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            warn!("Ignoring alarm history {}: {}", path.display(), e);
            Vec::new()
        }),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            warn!("Cannot read alarm history {}: {}", path.display(), e);
            Vec::new()
        }
    }
}

/// Writes the history file (through a temporary file, so that it is never left half written)
pub fn save(path: &Path, events: &[AlarmEvent]) -> io::Result<()> {
    let text = serde_json::to_string_pretty(events).map_err(io::Error::other)?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, text)?;
    fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarms_are_opened_changed_and_closed() {
        let mut log = AlarmLog::default();
        assert!(!log.update(8, "t0"));
        assert!(log.update(60, "t1"));
        assert!(!log.update(60, "t2"));
        assert_eq!(log.current().unwrap().state, "IgnitionFailed");
        // Another alarm closes the first one
        assert!(log.update(61, "t3"));
        assert!(log.update(0, "t4"));
        assert!(log.current().is_none());
        let events = log.newest_first();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].state_raw, 61);
        assert_eq!(events[0].ended_at.as_deref(), Some("t4"));
        assert_eq!(events[1].ended_at.as_deref(), Some("t3"));
        // States that are not alarms are not recorded
        assert!(!log.update(17, "t5"));
    }

    #[test]
    fn history_is_bounded_and_saved() {
        let mut log = AlarmLog::default();
        for i in 0..(KEPT + 5) {
            log.update(60, &format!("a{}", i));
            log.update(0, &format!("b{}", i));
        }
        assert_eq!(log.events().len(), KEPT);
        let dir = std::env::temp_dir().join(format!("hottoh-alarms-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("alarms.json");
        save(&path, &log.events()).unwrap();
        assert_eq!(load(&path), log.events());
        fs::remove_dir_all(&dir).unwrap();
        assert!(load(&dir.join("missing.json")).is_empty());
    }
}
