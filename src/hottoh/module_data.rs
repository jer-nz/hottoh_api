//! Answers of the module commands other than INF and DAT, as implemented by the Wifier 2.0
//! firmware 10.5.0 and used by the AppFire application.

use crate::hottoh::hottoh_const::StoveState;
use crate::hottoh::hottoh_structs::DataError;
use chrono::{DateTime, Local, SecondsFormat};
use serde::Serialize;
use utoipa::ToSchema;

/// Days of the chrono schedule, in firmware order (AppFire: index 0 is Sunday)
pub const DAYS: [&str; 7] = [
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
];
/// Half-hour slots per day
pub const SLOTS_PER_DAY: usize = 48;
const SLOT_MINUTES: u16 = 30;
/// Highest program number of a slot (0 = no program, 1 to 3 = chrono programs)
pub const MAX_PROGRAM: u8 = 3;

/// Answer to a `W` or `E` frame
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// `OK;`
    Ok,
    /// `ERR;<code>;` (the data logger answers `ERR;` without code to invalid parameters)
    Error(Option<i32>),
}

impl Outcome {
    /// `None` when the parameters are neither `OK` nor `ERR...`
    pub fn from_params(params: &[String]) -> Option<Self> {
        match params.first().map(String::as_str) {
            Some("OK") => Some(Outcome::Ok),
            Some("ERR") => Some(Outcome::Error(
                params.get(1).and_then(|code| code.parse().ok()),
            )),
            _ => None,
        }
    }
}

/// String parameter as the firmware expects it: `\"value\"`
pub fn quote(value: &str) -> String {
    format!("\\\"{}\\\"", value)
}

/// Removes the `\"` around a string parameter
pub fn unquote(value: &str) -> String {
    value.replace("\\\"", "")
}

fn parse<T: std::str::FromStr>(
    page: &'static str,
    field: &'static str,
    value: &str,
) -> Result<T, DataError> {
    value.parse().map_err(|_| DataError::InvalidField {
        page,
        field,
        value: value.to_string(),
    })
}

fn expect_count(page: &'static str, params: &[String], expected: usize) -> Result<(), DataError> {
    if params.len() == expected {
        Ok(())
    } else {
        Err(DataError::FieldCount {
            page,
            expected,
            got: params.len(),
        })
    }
}

/// UTC time as a local RFC 3339 date, `None` for 0 (unset)
pub fn local_time(utc: u32) -> Option<String> {
    (utc != 0)
        .then(|| DateTime::from_timestamp(i64::from(utc), 0))
        .flatten()
        .map(|t| {
            t.with_timezone(&Local)
                .to_rfc3339_opts(SecondsFormat::Secs, true)
        })
}

/// Weekly chrono schedule: program (0 to 3) of each half-hour slot, 7 days
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklySchedule {
    pub days: [[u8; SLOTS_PER_DAY]; 7],
}

impl Default for WeeklySchedule {
    fn default() -> Self {
        Self {
            days: [[0; SLOTS_PER_DAY]; 7],
        }
    }
}

impl WeeklySchedule {
    /// `SCH R` answer: `0;` (schedule number, always 0) then 336 slots, day by day
    pub fn from_params(params: &[String]) -> Result<Self, DataError> {
        expect_count("SCH", params, 1 + 7 * SLOTS_PER_DAY)?;
        let mut schedule = Self::default();
        for (i, value) in params[1..].iter().enumerate() {
            let program: u8 = parse("SCH", "slot", value)?;
            if program > MAX_PROGRAM {
                return Err(DataError::InvalidField {
                    page: "SCH",
                    field: "slot",
                    value: value.clone(),
                });
            }
            schedule.days[i / SLOTS_PER_DAY][i % SLOTS_PER_DAY] = program;
        }
        Ok(schedule)
    }

    /// `SCH W` parameters, in the same layout as the answer
    pub fn to_params(&self) -> Vec<String> {
        std::iter::once("0".to_string())
            .chain(self.days.iter().flatten().map(u8::to_string))
            .collect()
    }
}

/// Program running between two times of a day
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize, ToSchema)]
pub struct ProgramRange {
    /// `HH:MM`, on a half hour
    #[schema(example = "06:30")]
    pub start: String,
    /// `HH:MM`, on a half hour, after `start` (`24:00` for the end of the day)
    #[schema(example = "08:00")]
    pub end: String,
    /// Chrono program, 1 to 3 (see `/api/dat/1` for their temperatures)
    #[schema(example = 1)]
    pub program: u8,
}

fn slot_time(slot: usize) -> String {
    let minutes = slot as u16 * SLOT_MINUTES;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// `HH:MM` on a half hour to a slot index, `24:00` giving [`SLOTS_PER_DAY`]
fn time_slot(time: &str) -> Result<usize, String> {
    let invalid = || format!("invalid time '{}': expected HH:MM on a half hour", time);
    let (hours, minutes) = time.split_once(':').ok_or_else(invalid)?;
    if hours.len() != 2 || minutes.len() != 2 {
        return Err(invalid());
    }
    let hours: u16 = hours.parse().map_err(|_| invalid())?;
    let minutes: u16 = minutes.parse().map_err(|_| invalid())?;
    let total = hours * 60 + minutes;
    if minutes >= 60 || !minutes.is_multiple_of(SLOT_MINUTES) || total > 24 * 60 {
        return Err(invalid());
    }
    Ok(usize::from(total / SLOT_MINUTES))
}

/// Groups consecutive slots with the same program (slots without program are left out)
pub fn slots_to_ranges(slots: &[u8; SLOTS_PER_DAY]) -> Vec<ProgramRange> {
    let mut ranges = Vec::new();
    let mut start = 0;
    while start < SLOTS_PER_DAY {
        let program = slots[start];
        let end = (start..SLOTS_PER_DAY)
            .find(|&i| slots[i] != program)
            .unwrap_or(SLOTS_PER_DAY);
        if program != 0 {
            ranges.push(ProgramRange {
                start: slot_time(start),
                end: slot_time(end),
                program,
            });
        }
        start = end;
    }
    ranges
}

/// Builds the slots of a day from ranges; slots outside every range get no program
pub fn ranges_to_slots(ranges: &[ProgramRange]) -> Result<[u8; SLOTS_PER_DAY], String> {
    let mut slots = [0u8; SLOTS_PER_DAY];
    let mut used = [false; SLOTS_PER_DAY];
    for range in ranges {
        if !(1..=MAX_PROGRAM).contains(&range.program) {
            return Err(format!(
                "program must be between 1 and {} (got {})",
                MAX_PROGRAM, range.program
            ));
        }
        let (start, end) = (time_slot(&range.start)?, time_slot(&range.end)?);
        if start >= end {
            return Err(format!(
                "range {}-{}: end must be after start",
                range.start, range.end
            ));
        }
        for slot in start..end {
            if used[slot] {
                return Err(format!(
                    "range {}-{} overlaps another range",
                    range.start, range.end
                ));
            }
            used[slot] = true;
            slots[slot] = range.program;
        }
    }
    Ok(slots)
}

/// `CLK R` answer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ClockInfo {
    /// Clock of the Wi-Fi module, UTC seconds (0 when unavailable)
    pub module_utc: u32,
    /// Clock of the module as a local date (its own time zone applies on the stove side)
    pub module_time: Option<String>,
    /// Stove clock as last read over Modbus: local wall time encoded as seconds
    pub stove_local: u32,
    /// `stove_local` as `YYYY-MM-DDTHH:MM:SS`, in the time zone of the module
    pub stove_time: Option<String>,
}

impl ClockInfo {
    pub fn from_params(params: &[String]) -> Result<Self, DataError> {
        expect_count("CLK", params, 2)?;
        let module_utc = parse("CLK", "module_utc", &params[0])?;
        let stove_local: u32 = parse("CLK", "stove_local", &params[1])?;
        Ok(Self {
            module_utc,
            module_time: local_time(module_utc),
            stove_local,
            stove_time: (stove_local != 0)
                .then(|| DateTime::from_timestamp(i64::from(stove_local), 0))
                .flatten()
                .map(|t| t.naive_utc().format("%Y-%m-%dT%H:%M:%S").to_string()),
        })
    }
}

/// `ME0 R` answer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct DatalogBounds {
    /// UTC seconds of the oldest record (0 when the log is empty)
    pub first_utc: u32,
    pub first_time: Option<String>,
    /// UTC seconds of the newest record
    pub last_utc: u32,
    pub last_time: Option<String>,
}

impl DatalogBounds {
    pub fn from_params(params: &[String]) -> Result<Self, DataError> {
        expect_count("ME0", params, 2)?;
        let first_utc = parse("ME0", "first_utc", &params[0])?;
        let last_utc = parse("ME0", "last_utc", &params[1])?;
        Ok(Self {
            first_utc,
            first_time: local_time(first_utc),
            last_utc,
            last_time: local_time(last_utc),
        })
    }
}

/// One data logger record (`MET R`, 6 fields). The module samples the stove every 15 minutes
/// into `/log/log.bin`.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DatalogRecord {
    pub utc: u32,
    pub time: Option<String>,
    /// "Power Level" register
    pub power_level: u16,
    /// Room 1 temperature, °C
    pub room_temp: f32,
    /// Heater (water) temperature, °C
    pub water_temp: f32,
    /// Smoke temperature, °C
    pub smoke_temp: f32,
    /// Stove state at that time (AppFire shows 50 to 99 as alarms)
    #[schema(value_type = String)]
    pub state: StoveState,
    pub state_raw: u16,
}

impl DatalogRecord {
    pub fn list_from_params(params: &[String]) -> Result<Vec<Self>, DataError> {
        if !params.len().is_multiple_of(6) {
            return Err(DataError::FieldCount {
                page: "MET",
                expected: params.len().div_ceil(6) * 6,
                got: params.len(),
            });
        }
        let temp = |field, value: &String| -> Result<f32, DataError> {
            Ok(f32::from(parse::<i16>("MET", field, value)?) / 10.0)
        };
        params
            .chunks(6)
            .map(|r| {
                let utc = parse("MET", "utc", &r[0])?;
                let state_raw = parse("MET", "state", &r[5])?;
                Ok(Self {
                    utc,
                    time: local_time(utc),
                    power_level: parse("MET", "power_level", &r[1])?,
                    room_temp: temp("room_temp", &r[2])?,
                    water_temp: temp("water_temp", &r[3])?,
                    smoke_temp: temp("smoke_temp", &r[4])?,
                    state: StoveState::from(state_raw),
                    state_raw,
                })
            })
            .collect()
    }
}

/// `TMZ R` answer: `\"zone\";`, `ERR;8;\"zone\";` (zone unknown to the firmware table) or
/// `ERR;0;` (no zone set). Returns the zone and whether the firmware knows it.
pub fn time_zone_from_answer(
    outcome_code: Option<i32>,
    params: &[String],
) -> Result<(Option<String>, bool), DataError> {
    let invalid = || DataError::InvalidField {
        page: "TMZ",
        field: "answer",
        value: params.join(";"),
    };
    match (outcome_code, params) {
        (None, [zone]) => Ok((Some(unquote(zone)), true)),
        (Some(8), [_, _, zone]) => Ok((Some(unquote(zone)), false)),
        (Some(0), [_, _]) => Ok((None, false)),
        _ => Err(invalid()),
    }
}

/// Access point seen by a Wi-Fi scan (`SCN R`, 4 fields per network)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct WifiNetwork {
    /// BSSID, `AA:BB:CC:DD:EE:FF`
    pub bssid: String,
    pub ssid: String,
    /// Signal, dBm
    pub rssi: i32,
    /// `WPA-PSK`, `WPA2-PSK`, `WPA/WPA2-PSK`, `UNKNOWN` (open or WEP), `MAX` (enterprise), or
    /// `INVALID` when the module sent something else
    pub security: String,
    /// Text sent by the module. Its table of names stops before WPA3: for a WPA3 or WPA2/WPA3
    /// network it reads past the table and sends whatever is there
    pub security_raw: String,
}

/// Security names of the firmware table
const SECURITY_NAMES: [&str; 5] = ["UNKNOWN", "WPA-PSK", "WPA2-PSK", "WPA/WPA2-PSK", "MAX"];

impl WifiNetwork {
    pub fn list_from_params(params: &[String]) -> Result<Vec<Self>, DataError> {
        if !params.len().is_multiple_of(4) {
            return Err(DataError::FieldCount {
                page: "SCN",
                expected: params.len().div_ceil(4) * 4,
                got: params.len(),
            });
        }
        params
            .chunks(4)
            .map(|n| {
                Ok(Self {
                    bssid: n[0].clone(),
                    ssid: unquote(&n[1]),
                    rssi: parse("SCN", "rssi", &n[2])?,
                    security: if SECURITY_NAMES.contains(&n[3].as_str()) {
                        n[3].clone()
                    } else {
                        "INVALID".into()
                    },
                    security_raw: n[3].clone(),
                })
            })
            .collect()
    }
}

/// Server as `host` and `port`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

/// `BAL R` answer: `\"host\";port;`, balancer of the HottoH cloud relay (AppFire cloud mode)
pub fn balancer_from_params(params: &[String]) -> Result<Server, DataError> {
    expect_count("BAL", params, 2)?;
    Ok(Server {
        host: unquote(&params[0]),
        port: parse("BAL", "port", &params[1])?,
    })
}

/// 4-noks cloud server, where the module used to upload its data logger
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct CloudServer {
    pub url: String,
    pub path: String,
    pub port: u16,
}

/// `CLU R` answer: `\"url\";\"path\";port;`
pub fn cloud_server_from_params(params: &[String]) -> Result<CloudServer, DataError> {
    expect_count("CLU", params, 3)?;
    Ok(CloudServer {
        url: unquote(&params[0]),
        path: unquote(&params[1]),
        port: parse("CLU", "port", &params[2])?,
    })
}

/// `CRW R` answer: `utc;`
pub fn cloud_last_upload_from_params(params: &[String]) -> Result<u32, DataError> {
    expect_count("CRW", params, 1)?;
    parse("CRW", "utc", &params[0])
}

/// Version `major.minor.patch`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u16, pub u16, pub u16);

impl Version {
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().split('.').map(|p| p.parse::<u16>().ok());
        let version = Version(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }

    /// Versions that could follow this one: next patch, next minor, next major
    pub fn successors(&self) -> [Version; 3] {
        let Version(major, minor, patch) = *self;
        [
            Version(major, minor, patch.saturating_add(1)),
            Version(major, minor.saturating_add(1), 0),
            Version(major.saturating_add(1), 0, 0),
        ]
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// `PIN R` answer: `\"pin\";`
pub fn pin_from_params(params: &[String]) -> Result<String, DataError> {
    expect_count("PIN", params, 1)?;
    Ok(unquote(&params[0]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(s: &str) -> Vec<String> {
        s.split(';').map(str::to_string).collect()
    }

    #[test]
    fn outcomes() {
        assert_eq!(Outcome::from_params(&fields("OK")), Some(Outcome::Ok));
        assert_eq!(
            Outcome::from_params(&fields("ERR;18")),
            Some(Outcome::Error(Some(18)))
        );
        assert_eq!(
            Outcome::from_params(&fields("ERR")),
            Some(Outcome::Error(None))
        );
        assert_eq!(Outcome::from_params(&fields("0;1")), None);
    }

    #[test]
    fn quoting_matches_appfire() {
        assert_eq!(quote("Europe/Paris"), r#"\"Europe/Paris\""#);
        assert_eq!(unquote(r#"\"12345\""#), "12345");
    }

    fn sample_schedule() -> WeeklySchedule {
        let mut schedule = WeeklySchedule::default();
        // Monday 06:30-08:00 program 1, 18:00-22:30 program 2
        schedule.days[1][13..16].fill(1);
        schedule.days[1][36..45].fill(2);
        // Saturday until midnight, program 3
        schedule.days[6][40..48].fill(3);
        schedule
    }

    #[test]
    fn schedule_round_trip() {
        let schedule = sample_schedule();
        let params = schedule.to_params();
        assert_eq!(params.len(), 337);
        assert_eq!(params[0], "0");
        assert_eq!(params[1 + SLOTS_PER_DAY + 13], "1");
        assert_eq!(WeeklySchedule::from_params(&params).unwrap(), schedule);
    }

    #[test]
    fn schedule_rejects_bad_answers() {
        let mut params = sample_schedule().to_params();
        params[5] = "4".into();
        assert!(WeeklySchedule::from_params(&params).is_err());
        params.pop();
        assert!(matches!(
            WeeklySchedule::from_params(&params),
            Err(DataError::FieldCount { .. })
        ));
    }

    #[test]
    fn ranges_round_trip() {
        let schedule = sample_schedule();
        let monday = slots_to_ranges(&schedule.days[1]);
        assert_eq!(
            monday,
            vec![
                ProgramRange {
                    start: "06:30".into(),
                    end: "08:00".into(),
                    program: 1
                },
                ProgramRange {
                    start: "18:00".into(),
                    end: "22:30".into(),
                    program: 2
                },
            ]
        );
        assert_eq!(slots_to_ranges(&schedule.days[6])[0].end, "24:00");
        assert!(slots_to_ranges(&schedule.days[0]).is_empty());
        assert_eq!(ranges_to_slots(&monday).unwrap(), schedule.days[1]);
    }

    #[test]
    fn invalid_ranges_are_refused() {
        let range = |start: &str, end: &str, program| ProgramRange {
            start: start.into(),
            end: end.into(),
            program,
        };
        for ranges in [
            vec![range("06:15", "08:00", 1)],
            vec![range("08:00", "06:00", 1)],
            vec![range("06:00", "06:00", 1)],
            vec![range("06:00", "24:30", 1)],
            vec![range("6:00", "08:00", 1)],
            vec![range("06:00", "08:00", 0)],
            vec![range("06:00", "08:00", 4)],
            vec![range("06:00", "08:00", 1), range("07:30", "09:00", 2)],
        ] {
            assert!(ranges_to_slots(&ranges).is_err(), "{:?}", ranges);
        }
        assert!(ranges_to_slots(&[range("00:00", "24:00", 2)]).is_ok());
    }

    #[test]
    fn clock_and_bounds() {
        let clock = ClockInfo::from_params(&fields("1757930400;1757937600")).unwrap();
        assert_eq!(clock.module_utc, 1_757_930_400);
        assert_eq!(clock.stove_time.as_deref(), Some("2025-09-15T12:00:00"));
        let clock = ClockInfo::from_params(&fields("0;0")).unwrap();
        assert!(clock.module_time.is_none() && clock.stove_time.is_none());

        let bounds = DatalogBounds::from_params(&fields("1757000000;1757930400")).unwrap();
        assert_eq!(bounds.last_utc, 1_757_930_400);
        assert!(bounds.first_time.is_some());
        assert!(DatalogBounds::from_params(&fields("x;1")).is_err());
    }

    #[test]
    fn datalog_records() {
        let records = DatalogRecord::list_from_params(&fields(
            "1757930400;3;215;0;1450;8;1757930460;0;-5;0;30;60",
        ))
        .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].room_temp, 21.5);
        assert_eq!(records[0].smoke_temp, 145.0);
        assert_eq!(records[0].state, StoveState::Power);
        assert_eq!(records[1].room_temp, -0.5);
        assert_eq!(records[1].state, StoveState::IgnitionFailed);
        assert!(DatalogRecord::list_from_params(&[]).unwrap().is_empty());
        assert!(DatalogRecord::list_from_params(&fields("1;2;3")).is_err());
    }

    #[test]
    fn time_zone_answers() {
        assert_eq!(
            time_zone_from_answer(None, &fields(r#"\"Europe/Paris\""#)).unwrap(),
            (Some("Europe/Paris".into()), true)
        );
        assert_eq!(
            time_zone_from_answer(Some(8), &fields(r#"ERR;8;\"Mars/Olympus\""#)).unwrap(),
            (Some("Mars/Olympus".into()), false)
        );
        assert_eq!(
            time_zone_from_answer(Some(0), &fields("ERR;0")).unwrap(),
            (None, false)
        );
        assert!(time_zone_from_answer(Some(16), &fields("ERR;16")).is_err());
    }

    #[test]
    fn wifi_scan_answer() {
        let networks = WifiNetwork::list_from_params(&fields(
            r#"AA:BB:CC:DD:EE:01;\"Freebox-77D221\";-63;WPA2-PSK;AA:BB:CC:DD:EE:02;\"\";-90;UNKNOWN"#,
        ))
        .unwrap();
        assert_eq!(networks.len(), 2);
        assert_eq!(networks[0].ssid, "Freebox-77D221");
        assert_eq!(networks[0].rssi, -63);
        assert_eq!(networks[1].ssid, "");
        assert_eq!(networks[0].security, "WPA2-PSK");
        let wpa3 =
            WifiNetwork::list_from_params(&fields(r#"72:7F:F0:7C:DE:FB;\"x\";-95;4"#)).unwrap();
        assert_eq!(
            (wpa3[0].security.as_str(), wpa3[0].security_raw.as_str()),
            ("INVALID", "4")
        );
        assert!(WifiNetwork::list_from_params(&[]).unwrap().is_empty());
        assert!(WifiNetwork::list_from_params(&fields("a;b;c")).is_err());
    }

    #[test]
    fn cloud_answers() {
        assert_eq!(
            balancer_from_params(&fields(r#"\"app1.hottoh.it\";50612"#)).unwrap(),
            Server {
                host: "app1.hottoh.it".into(),
                port: 50612
            }
        );
        let cloud = cloud_server_from_params(&fields(
            r#"\"http://hottoh.4-cloud.org\";\"/hottoh/datalogger2/serverCgi2.php\";80"#,
        ))
        .unwrap();
        assert_eq!(cloud.port, 80);
        assert_eq!(cloud.path, "/hottoh/datalogger2/serverCgi2.php");
        assert_eq!(cloud_last_upload_from_params(&fields("0")).unwrap(), 0);
        assert!(balancer_from_params(&fields("x")).is_err());
    }

    #[test]
    fn versions() {
        let v = Version::parse("10.5.0").unwrap();
        assert_eq!(v.to_string(), "10.5.0");
        assert!(Version(10, 6, 0) > v && Version(9, 9, 9) < v);
        assert_eq!(
            v.successors(),
            [Version(10, 5, 1), Version(10, 6, 0), Version(11, 0, 0)]
        );
        assert!(Version::parse("10.5").is_none());
        assert!(Version::parse("10.5.0.1").is_none());
        assert!(Version::parse("nosig").is_none());
    }

    #[test]
    fn pin_answer() {
        assert_eq!(pin_from_params(&fields(r#"\"a1b2c3\""#)).unwrap(), "a1b2c3");
        assert!(pin_from_params(&[]).is_err());
    }
}
