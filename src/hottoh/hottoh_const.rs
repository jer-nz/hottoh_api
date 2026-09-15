use serde::{Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Type of command sent to the stove (one character in the frame)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    /// Read data from the stove
    Read,
    /// Write data to the stove
    Write,
    /// Execute a command on the stove
    Execute,
}

impl FromStr for CommandType {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "R" => Ok(CommandType::Read),
            "W" => Ok(CommandType::Write),
            "E" => Ok(CommandType::Execute),
            _ => Err(format!("Invalid command type: {}", input)),
        }
    }
}

impl CommandType {
    /// String representation used in the frame ("R", "W" or "E")
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandType::Read => "R",
            CommandType::Write => "W",
            CommandType::Execute => "E",
        }
    }
}

/// Command (three characters in the frame)
///
/// The Wifier 2.0 firmware 10.5.0 also knows these commands, deliberately not used by this
/// bridge: WIC (writes the Wi-Fi settings whatever the frame type; nothing reads them back),
/// UPG (firmware update; `R` does nothing), DAC (fills the data logger with 3000 fake records),
/// BRQ (changes the relay server), REG (does nothing), and the writes of BAL, CLU and CRW.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Module information (hostname, firmware version, Wi-Fi signal)
    Inf,
    /// Stove data: read a page (`R`) or write a setting (`W`)
    Dat,
    /// Weekly chrono schedule: 7 days of 48 half-hour slots (`R`, `W`)
    Sch,
    /// Module clock (`R`); `W` sets the module clock and the stove clock
    Clk,
    /// Time zone name of the module (`R`); `W` also sets the stove clock
    Tmz,
    /// Security PIN of the cloud relay, used by AppFire in cloud mode (`R`, `W`)
    Pin,
    /// Data logger records from a UTC time (`R`)
    Met,
    /// UTC time of the first and last data logger records (`R`)
    Me0,
    /// Clears the data logger (`E`)
    Mec,
    /// Restarts the Wi-Fi module (`E`, no answer)
    Rst,
    /// Wi-Fi scan (`R` without parameter); suspends the stove communication meanwhile
    Scn,
    /// Balancer server of the HottoH cloud relay (`R`)
    Bal,
    /// URL, path and port of the 4-noks cloud (`R`)
    Clu,
    /// UTC time of the last data logger upload to the 4-noks cloud (`R`)
    Crw,
}

impl Command {
    /// String representation used in the frame
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::Inf => "INF",
            Command::Dat => "DAT",
            Command::Sch => "SCH",
            Command::Clk => "CLK",
            Command::Tmz => "TMZ",
            Command::Pin => "PIN",
            Command::Met => "MET",
            Command::Me0 => "ME0",
            Command::Mec => "MEC",
            Command::Rst => "RST",
            Command::Scn => "SCN",
            Command::Bal => "BAL",
            Command::Clu => "CLU",
            Command::Crw => "CRW",
        }
    }

    /// The module takes longer to answer (the Wi-Fi scan lasts a few seconds)
    pub fn is_slow(&self) -> bool {
        matches!(self, Command::Scn)
    }

    /// Resending after a missing answer would repeat an action with side effects
    pub fn can_retry(&self) -> bool {
        !matches!(self, Command::Scn)
    }

    /// Frames of this command carry a secret and are not written to the log
    pub fn is_sensitive(&self) -> bool {
        matches!(self, Command::Pin)
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "INF" => Ok(Command::Inf),
            "DAT" => Ok(Command::Dat),
            "SCH" => Ok(Command::Sch),
            "CLK" => Ok(Command::Clk),
            "TMZ" => Ok(Command::Tmz),
            "PIN" => Ok(Command::Pin),
            "MET" => Ok(Command::Met),
            "ME0" => Ok(Command::Me0),
            "MEC" => Ok(Command::Mec),
            "RST" => Ok(Command::Rst),
            "SCN" => Ok(Command::Scn),
            "BAL" => Ok(Command::Bal),
            "CLU" => Ok(Command::Clu),
            "CRW" => Ok(Command::Crw),
            _ => Err(format!("Unsupported command: {}", input)),
        }
    }
}

/// Time zones offered by AppFire (`SetTimeZone`). The firmware maps a name to a POSIX rule with
/// its own table and answers `ERR` to a name it does not know.
pub const TIME_ZONES: [&str; 56] = [
    "UTC",
    "Europe/Amsterdam",
    "Europe/Andorra",
    "Europe/Athens",
    "Europe/Belgrade",
    "Europe/Berlin",
    "Europe/Bratislava",
    "Europe/Brussels",
    "Europe/Bucharest",
    "Europe/Budapest",
    "Europe/Chisinau",
    "Europe/Copenhagen",
    "Europe/Dublin",
    "Europe/Gibraltar",
    "Europe/Guernsey",
    "Europe/Helsinki",
    "Europe/IsleofMan",
    "Europe/Istanbul",
    "Europe/Jersey",
    "Europe/Kaliningrad",
    "Europe/Kiev",
    "Europe/Lisbon",
    "Europe/Ljubljana",
    "Europe/London",
    "Europe/Luxembourg",
    "Europe/Madrid",
    "Europe/Malta",
    "Europe/Mariehamn",
    "Europe/Minsk",
    "Europe/Monaco",
    "Europe/Moscow",
    "Europe/Oslo",
    "Europe/Paris",
    "Europe/Podgorica",
    "Europe/Prague",
    "Europe/Riga",
    "Europe/Rome",
    "Europe/Samara",
    "Europe/SanMarino",
    "Europe/Sarajevo",
    "Europe/Simferopol",
    "Europe/Skopje",
    "Europe/Sofia",
    "Europe/Stockholm",
    "Europe/Tallinn",
    "Europe/Tirane",
    "Europe/Uzhgorod",
    "Europe/Vaduz",
    "Europe/Vatican",
    "Europe/Vienna",
    "Europe/Vilnius",
    "Europe/Volgograd",
    "Europe/Warsaw",
    "Europe/Zagreb",
    "Europe/Zaporozhye",
    "Europe/Zurich",
];

/// Current state of the stove ("Status configuration" register)
///
/// Values not listed here are kept as `Unknown` instead of rejecting the whole DAT0 page.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum StoveState {
    #[default]
    Off,
    Starting1,
    Starting2,
    Starting3,
    Starting4,
    Starting5,
    Starting6,
    Starting7,
    Power,
    Stopping1,
    Stopping2,
    EcoStop1,
    EcoStop2,
    EcoStop3,
    LowPellet,
    EndPellet,
    BlackOut,
    AntiFreeze,
    IgnitionFailed,
    NoPellet,
    CoverOpen,
    Unknown(u16),
}

impl From<u16> for StoveState {
    fn from(value: u16) -> Self {
        match value {
            0 => StoveState::Off,
            1 => StoveState::Starting1,
            2 => StoveState::Starting2,
            3 => StoveState::Starting3,
            4 => StoveState::Starting4,
            5 => StoveState::Starting5,
            6 => StoveState::Starting6,
            7 => StoveState::Starting7,
            8 => StoveState::Power,
            9 => StoveState::Stopping1,
            10 => StoveState::Stopping2,
            11 => StoveState::EcoStop1,
            12 => StoveState::EcoStop2,
            13 => StoveState::EcoStop3,
            14 => StoveState::LowPellet,
            15 => StoveState::EndPellet,
            16 => StoveState::BlackOut,
            17 => StoveState::AntiFreeze,
            60 => StoveState::IgnitionFailed,
            61 => StoveState::NoPellet,
            69 => StoveState::CoverOpen,
            other => StoveState::Unknown(other),
        }
    }
}

impl StoveState {
    /// States reported as alarms: pellets low or out, power cut, and the 50 to 99 range that
    /// AppFire shows as alarms (ignition failure, no pellets, door open...)
    pub fn is_alarm(raw: u16) -> bool {
        matches!(raw, 14..=16 | 50..=99)
    }
}

impl fmt::Display for StoveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoveState::Unknown(_) => write!(f, "Unknown"),
            other => write!(f, "{:?}", other),
        }
    }
}

impl Serialize for StoveState {
    /// Serialized as a plain string ("Power", "Off", ..., "Unknown")
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Function mode of the stove ("Function Mode" register)
///
/// The firmware only accepts 0 (manual) and 2 (chrono) when writing.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ChronoMode {
    Manual = 0,
    Chrono = 2,
}

impl ChronoMode {
    /// Value to send in a `DAT W 8;<value>;` frame
    pub fn from_enabled(enabled: bool) -> Self {
        if enabled {
            ChronoMode::Chrono
        } else {
            ChronoMode::Manual
        }
    }
}

/// Manufacturer of the stove ("Customer ID" register)
#[derive(Debug, PartialEq, Eq)]
pub enum StoveManufacturer {
    Cmg = 9,
    Manufacturer65 = 65,
    Manufacturer76 = 76,
    Edilkamin = 85,
    Manufacturer100 = 100,
}

impl StoveManufacturer {
    /// Converts a register value to a known manufacturer
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            9 => Some(StoveManufacturer::Cmg),
            65 => Some(StoveManufacturer::Manufacturer65),
            76 => Some(StoveManufacturer::Manufacturer76),
            85 => Some(StoveManufacturer::Edilkamin),
            100 => Some(StoveManufacturer::Manufacturer100),
            _ => None,
        }
    }
}

/// Setting index used in `DAT W <index>;<value>;` frames
///
/// Ranges checked by firmware 10.5.0 (outside of them the stove answers `ERR;17;`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Complete protocol table; not every index has an HTTP endpoint yet
pub enum StoveCommands {
    /// 0 or 1
    OnOff = 0,
    /// 0 or 1
    EcoMode = 1,
    /// 0..=100
    PowerLevel = 2,
    /// Tenths of °C, 0..=9999
    AmbianceTemperature1 = 3,
    /// Tenths of °C, 0..=9999
    AmbianceTemperature2 = 4,
    /// 0..=101
    FanSpeed1 = 5,
    /// 0..=101
    FanSpeed2 = 6,
    /// 0..=101
    FanSpeed3 = 7,
    /// 0 (manual) or 2 (chrono), see [`ChronoMode`]
    ChronoOnOff = 8,
    /// Tenths of °C, 0..=9999
    ChronoTemperature1 = 9,
    /// Tenths of °C, 0..=9999
    ChronoTemperature2 = 10,
    /// Tenths of °C, 0..=9999
    ChronoTemperature3 = 11,
    /// Accepted but ignored by firmware 10.5.0 (answers `OK;` without writing anything)
    SanTemperature = 12,
    /// Accepted but ignored by firmware 10.5.0 (answers `OK;` without writing anything)
    PufTemperature = 13,
    /// Tenths of °C, 0..=9999
    BoilerTemperature = 14,
    /// 0..=9999
    HottohSetRecipe = 15,
    /// 0..=9999
    HottohSetPelSetpoint = 16,
}

impl StoveCommands {
    /// Human readable name, used in logs and API messages
    pub fn name(&self) -> &'static str {
        match self {
            StoveCommands::OnOff => "OnOff",
            StoveCommands::EcoMode => "EcoMode",
            StoveCommands::PowerLevel => "PowerLevel",
            StoveCommands::AmbianceTemperature1 => "AmbianceTemperature1",
            StoveCommands::AmbianceTemperature2 => "AmbianceTemperature2",
            StoveCommands::FanSpeed1 => "FanSpeed1",
            StoveCommands::FanSpeed2 => "FanSpeed2",
            StoveCommands::FanSpeed3 => "FanSpeed3",
            StoveCommands::ChronoOnOff => "ChronoOnOff",
            StoveCommands::ChronoTemperature1 => "ChronoTemperature1",
            StoveCommands::ChronoTemperature2 => "ChronoTemperature2",
            StoveCommands::ChronoTemperature3 => "ChronoTemperature3",
            StoveCommands::SanTemperature => "SanTemperature",
            StoveCommands::PufTemperature => "PufTemperature",
            StoveCommands::BoilerTemperature => "BoilerTemperature",
            StoveCommands::HottohSetRecipe => "HottohSetRecipe",
            StoveCommands::HottohSetPelSetpoint => "HottohSetPelSetpoint",
        }
    }
}

/// Error codes returned by the firmware in `ERR;<code>;` answers
pub fn error_message(code: Option<i32>) -> &'static str {
    match code {
        Some(6) => "no data logger record at or after this time",
        Some(8) => "time zone not known by the module",
        Some(16) => "module could not read or allocate the data",
        Some(17) => "value out of range or missing",
        Some(18) => "module refused the value (invalid or not applied)",
        Some(19) => "stove did not accept the value (Modbus write failed)",
        _ => "stove returned an error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_stove_state_is_kept() {
        assert_eq!(StoveState::from(8), StoveState::Power);
        assert_eq!(StoveState::from(42), StoveState::Unknown(42));
        assert_eq!(
            serde_json::to_string(&StoveState::Power).unwrap(),
            "\"Power\""
        );
        assert_eq!(
            serde_json::to_string(&StoveState::Unknown(42)).unwrap(),
            "\"Unknown\""
        );
    }

    #[test]
    fn chrono_mode_values_match_firmware() {
        assert_eq!(ChronoMode::from_enabled(true) as i32, 2);
        assert_eq!(ChronoMode::from_enabled(false) as i32, 0);
    }
}
