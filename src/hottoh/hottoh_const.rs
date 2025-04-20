use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::IntoStaticStr;

/// Type of command to be sent to the stove
#[derive(Debug, PartialEq)]
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

    /// Converts a string to a CommandType
    ///
    /// # Arguments
    ///
    /// * `input` - The string to convert ("R", "W", or "E")
    ///
    /// # Returns
    ///
    /// * `Result<CommandType, String>` - The parsed CommandType or an error
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "R" => Ok(CommandType::Read),
            "W" => Ok(CommandType::Write),
            "E" => Ok(CommandType::Execute),
            _ => Err(format!("Invalid command: {}", input)),
        }
    }
}

impl CommandType {
    /// Converts a CommandType to its string representation
    ///
    /// # Returns
    ///
    /// * `&'static str` - The string representation ("R", "W", or "E")
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandType::Read => "R",
            CommandType::Write => "W",
            CommandType::Execute => "E",
        }
    }
}

/// Command to be sent to the stove
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Information command (hostname, version, signal)
    Inf,
    /// Data command (generic)
    Dat,
    /// Data page 0 (main stove data)
    Dat0,
    /// Data page 1 (additional temperature data)
    Dat1,
    /// Data page 2 (additional pump and valve data)
    Dat2,
    /// Response to a data request
    DatReqResponse,
}

impl Command {
    /// Converts a Command to its string representation
    ///
    /// # Returns
    ///
    /// * `&'static str` - The string representation of the command
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::Inf => "INF",
            Command::Dat => "DAT",
            Command::Dat0 => "DAT0",
            Command::Dat1 => "DAT1",
            Command::Dat2 => "DAT2",
            Command::DatReqResponse => "DATReqResponse",
        }
    }
}

impl FromStr for Command {
    type Err = String;

    /// Converts a string to a Command
    ///
    /// # Arguments
    ///
    /// * `input` - The string to convert
    ///
    /// # Returns
    ///
    /// * `Result<Command, String>` - The parsed Command or an error
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "INF" => Ok(Command::Inf),
            "DAT" => Ok(Command::Dat),
            "DAT0" => Ok(Command::Dat0),
            "DAT1" => Ok(Command::Dat1),
            "DAT2" => Ok(Command::Dat2),
            "DATReqResponse" => Ok(Command::DatReqResponse),
            _ => Err(format!("Invalid command: {}", input)),
        }
    }
}

/// Current state of the stove
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub enum StoveState {
    #[default]
    Off = 0,
    Starting1 = 1,
    Starting2 = 2,
    Starting3 = 3,
    Starting4 = 4,
    Starting5 = 5,
    Starting6 = 6,
    Starting7 = 7,
    Power = 8,
    Stopping1 = 9,
    Stopping2 = 10,
    EcoStop1 = 11,
    EcoStop2 = 12,
    EcoStop3 = 13,
    LowPellet = 14,
    EndPellet = 15,
    BlackOut = 16,
    AntiFreeze = 17,
    IgnitionFailed = 60,
    NoPellet = 61,
    CoverOpen = 69,
}

/// Error type for StoveState parsing
#[derive(Debug)]
pub struct StoveStateError;

impl FromStr for StoveState {
    type Err = StoveStateError;

    /// Converts a string to a StoveState
    ///
    /// # Arguments
    ///
    /// * `s` - The string to convert (numeric value)
    ///
    /// # Returns
    ///
    /// * `Result<StoveState, StoveStateError>` - The parsed StoveState or an error
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i32>() {
            Ok(num) => match num {
                0 => Ok(StoveState::Off),
                1 => Ok(StoveState::Starting1),
                2 => Ok(StoveState::Starting2),
                3 => Ok(StoveState::Starting3),
                4 => Ok(StoveState::Starting4),
                5 => Ok(StoveState::Starting5),
                6 => Ok(StoveState::Starting6),
                7 => Ok(StoveState::Starting7),
                8 => Ok(StoveState::Power),
                9 => Ok(StoveState::Stopping1),
                10 => Ok(StoveState::Stopping2),
                11 => Ok(StoveState::EcoStop1),
                12 => Ok(StoveState::EcoStop2),
                13 => Ok(StoveState::EcoStop3),
                14 => Ok(StoveState::LowPellet),
                15 => Ok(StoveState::EndPellet),
                16 => Ok(StoveState::BlackOut),
                17 => Ok(StoveState::AntiFreeze),
                60 => Ok(StoveState::IgnitionFailed),
                61 => Ok(StoveState::NoPellet),
                69 => Ok(StoveState::CoverOpen),
                _ => Err(StoveStateError),
            },
            Err(_) => Err(StoveStateError),
        }
    }
}

/// Chrono mode of the stove
#[derive(PartialEq)]
#[allow(dead_code)]
pub enum StoveChronoMode {
    ChronoOff = 0,
    ChronoSleep = 1,
    ChronoOn1 = 2,
    ChronoOn2 = 3,
    ChronoOn3 = 4,
    ChronoOn4 = 5,
}

/// Manufacturer of the stove
#[derive(Debug, PartialEq)]
pub enum StoveManufacturer {
    Cmg = 9,
    Manufacturer65 = 65,
    Manufacturer76 = 76,
    Edilkamin = 85,
    Manufacturer100 = 100,
}

impl StoveManufacturer {
    /// Converts a u16 value to a StoveManufacturer
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric value representing the manufacturer
    ///
    /// # Returns
    ///
    /// * `Option<StoveManufacturer>` - The manufacturer if recognized, None otherwise
    pub(crate) fn from_u16(value: u16) -> Option<Self> {
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

/// Commands that can be sent to the stove
#[derive(IntoStaticStr, Debug, PartialEq)]
#[allow(dead_code)]
pub enum StoveCommands {
    OnOff = 0,
    EcoMode = 1,
    PowerLevel = 2,
    AmbianceTemperature1 = 3,
    AmbianceTemperature2 = 4,
    FanSpeed1 = 5,
    FanSpeed2 = 6,
    FanSpeed3 = 7,
    ChronoOnOff = 8,
    ChronoTemperature1 = 9,
    ChronoTemperature2 = 10,
    ChronoTemperature3 = 11,
    SanTemperature = 12,       // not tested
    PufTemperature = 13,       // not tested
    BoilerTemperature = 14,    // not tested
    HottohSetRecipe = 15,      // unknown
    HottohSetPelSetpoint = 16, // unknown
}
