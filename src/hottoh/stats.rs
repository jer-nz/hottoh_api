//! Periodic statistics line and process metrics, to follow the stability of the bridge over
//! long periods (grep `Stats` in the log).

use crate::hottoh::shared_struct::{Bridge, ConnectionStatus, Stats};
use log::{error, info};
use serde::Serialize;
use std::fmt::Display;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Resources used by the process (Linux only, `None` elsewhere)
#[derive(Debug, Serialize, Clone, Default)]
pub struct ProcessInfo {
    pub rss_kb: Option<u64>,
    pub threads: Option<u64>,
    pub open_fds: Option<usize>,
}

pub fn process_info() -> ProcessInfo {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |name: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse().ok())
    };
    ProcessInfo {
        rss_kb: field("VmRSS:"),
        threads: field("Threads:"),
        open_fds: std::fs::read_dir("/proc/self/fd")
            .ok()
            .map(|dir| dir.count()),
    }
}

/// Starts the thread logging a statistics line every `interval` (nothing if zero)
pub fn spawn_reporter(bridge: Arc<Bridge>, interval: Duration) -> Option<thread::JoinHandle<()>> {
    if interval.is_zero() {
        return None;
    }
    thread::Builder::new()
        .name("stats".into())
        .spawn(move || {
            let mut previous = Stats::default();
            loop {
                bridge.sleep(interval);
                if !bridge.is_running() {
                    break;
                }
                let pending_writes = bridge.writes().len();
                let connection = bridge.state().connection().clone();
                info!(
                    "{}",
                    report(
                        &previous,
                        &connection,
                        pending_writes,
                        bridge.uptime(),
                        interval,
                        &process_info()
                    )
                );
                previous = connection.stats;
            }
        })
        .map_err(|e| error!("Cannot start the statistics thread: {}", e))
        .ok()
}

fn or_unknown<T: Display>(value: Option<T>) -> String {
    value.map_or_else(|| "?".into(), |v| v.to_string())
}

/// `3d 04:05:06`
fn human_duration(duration: Duration) -> String {
    let s = duration.as_secs();
    format!(
        "{}d {:02}:{:02}:{:02}",
        s / 86_400,
        s / 3600 % 24,
        s / 60 % 60,
        s % 60
    )
}

fn report(
    previous: &Stats,
    connection: &ConnectionStatus,
    pending_writes: usize,
    uptime: Duration,
    period: Duration,
    process: &ProcessInfo,
) -> String {
    let now = &connection.stats;
    let delta = |field: fn(&Stats) -> u64| field(now).saturating_sub(field(previous));
    let answers = delta(|s| s.answers);
    let latency_avg = delta(|s| s.latency_total_ms)
        .checked_div(answers)
        .map_or_else(|| "-".into(), |ms| ms.to_string());
    format!(
        "Stats over {} s: {} requests, {} answers, {} timeouts, {} invalid frames, \
         {} late answers, {} decode errors, writes {} ok/{} refused/{} failed, \
         latency avg {} ms | since start ({}): {} connections, {} disconnections, \
         {} failed connects, {} timeouts, latency max {} ms | connected {}, \
         last answer {}, {} pending writes | rss {} kB, {} threads, {} fds",
        period.as_secs(),
        delta(|s| s.requests),
        answers,
        delta(|s| s.timeouts),
        delta(|s| s.invalid_frames),
        delta(|s| s.late_answers),
        delta(|s| s.decode_errors),
        delta(|s| s.writes_ok),
        delta(|s| s.writes_refused),
        delta(|s| s.writes_failed),
        latency_avg,
        human_duration(uptime),
        connection.connections,
        now.disconnections,
        now.connect_failures,
        now.timeouts,
        now.latency_max_ms,
        connection.connected,
        or_unknown(connection.last_response_at.as_deref()),
        pending_writes,
        or_unknown(process.rss_kb),
        or_unknown(process.threads),
        or_unknown(process.open_fds),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_are_readable() {
        assert_eq!(human_duration(Duration::from_secs(59)), "0d 00:00:59");
        assert_eq!(
            human_duration(Duration::from_secs(3 * 86_400 + 4 * 3600 + 5 * 60 + 6)),
            "3d 04:05:06"
        );
    }

    #[test]
    fn report_shows_period_deltas_and_totals() {
        let previous = Stats {
            requests: 100,
            answers: 98,
            timeouts: 2,
            latency_total_ms: 4000,
            ..Stats::default()
        };
        let connection = ConnectionStatus {
            connected: true,
            connections: 2,
            stats: Stats {
                requests: 150,
                answers: 148,
                timeouts: 2,
                latency_total_ms: 6000,
                latency_max_ms: 900,
                disconnections: 1,
                ..Stats::default()
            },
            ..ConnectionStatus::default()
        };
        let line = report(
            &previous,
            &connection,
            0,
            Duration::from_secs(90),
            Duration::from_secs(60),
            &ProcessInfo::default(),
        );
        assert!(line.starts_with("Stats over 60 s: 50 requests, 50 answers, 0 timeouts"));
        assert!(line.contains("latency avg 40 ms"));
        assert!(line.contains("since start (0d 00:01:30): 2 connections, 1 disconnections"));
        assert!(line.contains("rss ? kB"));
    }

    #[test]
    fn process_info_is_read_on_linux() {
        if cfg!(target_os = "linux") {
            let info = process_info();
            assert!(info.rss_kb.unwrap() > 0);
            assert!(info.threads.unwrap() >= 1);
            assert!(info.open_fds.unwrap() >= 3);
        }
    }
}
