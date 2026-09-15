//! Data pages returned by the stove and their decoding.
//!
//! Field order and integer types come from the Wifier 2.0 firmware 10.5.0
//! (`hottoh_rtdata_to_string` and the Modbus parameter table):
//! values printed with `%d` are signed 16-bit, values printed with `%hu` are unsigned 16-bit.
//! Temperatures are sent in tenths of °C and exposed in °C.

use super::hottoh_const::{StoveManufacturer, StoveState};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

/// Error while decoding the parameters of a response
#[derive(Error, Debug, PartialEq, Eq)]
pub enum DataError {
    #[error("{page}: expected {expected} fields, got {got}")]
    FieldCount {
        page: &'static str,
        expected: usize,
        got: usize,
    },
    #[error("{page}: invalid value '{value}' for {field}")]
    InvalidField {
        page: &'static str,
        field: &'static str,
        value: String,
    },
}

/// Sequential reader over the `;`-separated parameters of a response
struct Fields<'a> {
    page: &'static str,
    items: &'a [String],
    pos: usize,
}

impl<'a> Fields<'a> {
    fn new(page: &'static str, items: &'a [String], expected: usize) -> Result<Self, DataError> {
        if items.len() != expected {
            return Err(DataError::FieldCount {
                page,
                expected,
                got: items.len(),
            });
        }
        Ok(Self {
            page,
            items,
            pos: 0,
        })
    }

    fn next_str(&mut self) -> &'a str {
        let value = self.items[self.pos].as_str();
        self.pos += 1;
        value
    }

    fn invalid(&self, field: &'static str, value: &str) -> DataError {
        DataError::InvalidField {
            page: self.page,
            field,
            value: value.to_string(),
        }
    }

    /// Value printed with `%hu`
    fn u16(&mut self, field: &'static str) -> Result<u16, DataError> {
        let value = self.next_str();
        value
            .parse::<u32>()
            .ok()
            .and_then(|v| u16::try_from(v).ok())
            .ok_or_else(|| self.invalid(field, value))
    }

    /// Value printed with `%d` from a signed 16-bit register
    fn i16(&mut self, field: &'static str) -> Result<i16, DataError> {
        let value = self.next_str();
        value
            .parse::<i32>()
            .ok()
            .and_then(|v| i16::try_from(v).ok())
            .ok_or_else(|| self.invalid(field, value))
    }

    /// Bit or flag: 0 is false, any other value is true
    fn bool(&mut self, field: &'static str) -> Result<bool, DataError> {
        Ok(self.u16(field)? != 0)
    }

    /// Signed temperature in tenths of °C, returned in °C
    fn temp_i16(&mut self, field: &'static str) -> Result<f32, DataError> {
        Ok(f32::from(self.i16(field)?) / 10.0)
    }

    /// Temperature sent with `%hu`, returned in °C
    fn temp_u16(&mut self, field: &'static str) -> Result<f32, DataError> {
        Ok(f32::from(self.u16(field)?) / 10.0)
    }
}

/// Current local time, RFC 3339
pub(crate) fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// `INF` answer: `HOTTOH32;<major>.<minor>.<patch>;<signal|nosig>;`
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct INFData {
    pub hostname: String,
    pub version: String,
    /// Wi-Fi signal as reported by the module, or "nosig"
    pub signal: String,
    pub last_updated: String,
}

impl INFData {
    pub fn from_slice(data: &[String]) -> Result<Self, DataError> {
        let mut f = Fields::new("INF", data, 3)?;
        Ok(Self {
            hostname: f.next_str().to_string(),
            version: f.next_str().to_string(),
            signal: f.next_str().to_string(),
            last_updated: now(),
        })
    }
}

/// DAT page 0: main stove data (36 fields)
#[derive(Debug, Serialize, Clone, Default)]
pub struct DAT0Data {
    pub index_page: u16,
    /// "Customer ID" register
    #[serde(serialize_with = "serialize_stove_manufacturer")]
    pub index_manufacturer: u16,
    /// "Init Bitmap" bit
    pub index_bitmap_visible: bool,
    /// Modbus link between the module and the stove board is up
    pub index_valid: bool,
    /// "Stove configuration" bitfield
    pub index_stove_type: u16,
    /// "Status configuration" register, decoded
    pub index_stove_state: StoveState,
    /// "Status configuration" register, raw value
    pub index_stove_state_raw: u16,
    pub index_stove_on: bool,
    pub index_eco_mode: bool,
    /// "Function Mode" register: 0 = manual, 2 = chrono
    pub index_chrono_mode: u16,
    pub index_ambient_t1: f32,
    pub index_ambient_t1_set: f32,
    pub index_ambient_t1_set_min: f32,
    pub index_ambient_t1_set_max: f32,
    pub index_ambient_t2: f32,
    pub index_ambient_t2_set: f32,
    pub index_ambient_t2_set_min: f32,
    pub index_ambient_t2_set_max: f32,
    /// "Heater Temperature" (water of the stove body)
    pub index_water: f32,
    pub index_water_set: f32,
    pub index_water_set_min: f32,
    pub index_water_set_max: f32,
    pub index_smoke_t: f32,
    pub index_power_level: u16,
    pub index_power_set: u16,
    pub index_power_min: u16,
    pub index_power_max: u16,
    /// "Temperature Fan Speed Smoke"
    pub index_fan_smoke: u16,
    /// Fan 1 set point (the firmware sends the set point here, not the actual speed:
    /// see `index_fan_1_speed` in DAT page 2)
    pub index_fan_1: u16,
    pub index_fan_1_set: u16,
    pub index_fan_1_set_max: u16,
    /// Fan 2 set point (see `index_fan_1`)
    pub index_fan_2: u16,
    pub index_fan_2_set: u16,
    pub index_fan_2_set_max: u16,
    /// Fan 3 set point (see `index_fan_1`)
    pub index_fan_3: u16,
    pub index_fan_3_set: u16,
    pub index_fan_3_set_max: u16,
    pub boiler_enabled: bool,
    pub domestic_hot_water_enabled: bool,
    pub fan_number: u16,
    pub temp_room1_enabled: bool,
    pub temp_room2_enabled: bool,
    pub temp_room3_enabled: bool,
    pub temp_water_enabled: bool,
    pub pump_enabled: bool,
    pub last_updated: String,
}

impl DAT0Data {
    pub fn from_slice(data: &[String]) -> Result<Self, DataError> {
        let mut f = Fields::new("DAT0", data, 36)?;
        let index_page = f.u16("index_page")?;
        let index_manufacturer = f.u16("index_manufacturer")?;
        let index_bitmap_visible = f.bool("index_bitmap_visible")?;
        let index_valid = f.bool("index_valid")?;
        let index_stove_type = f.u16("index_stove_type")?;
        let index_stove_state_raw = f.u16("index_stove_state")?;

        // Bit layout of the "Stove configuration" register, taken from the original
        // hottohpy project (not verified in the module firmware, which only forwards it).
        let t = index_stove_type;
        Ok(Self {
            index_page,
            index_manufacturer,
            index_bitmap_visible,
            index_valid,
            index_stove_type,
            index_stove_state: StoveState::from(index_stove_state_raw),
            index_stove_state_raw,
            index_stove_on: f.bool("index_stove_on")?,
            index_eco_mode: f.bool("index_eco_mode")?,
            index_chrono_mode: f.u16("index_chrono_mode")?,
            index_ambient_t1: f.temp_i16("index_ambient_t1")?,
            index_ambient_t1_set: f.temp_i16("index_ambient_t1_set")?,
            index_ambient_t1_set_min: f.temp_i16("index_ambient_t1_set_min")?,
            index_ambient_t1_set_max: f.temp_i16("index_ambient_t1_set_max")?,
            index_ambient_t2: f.temp_i16("index_ambient_t2")?,
            index_ambient_t2_set: f.temp_i16("index_ambient_t2_set")?,
            index_ambient_t2_set_min: f.temp_i16("index_ambient_t2_set_min")?,
            index_ambient_t2_set_max: f.temp_i16("index_ambient_t2_set_max")?,
            index_water: f.temp_i16("index_water")?,
            index_water_set: f.temp_i16("index_water_set")?,
            index_water_set_min: f.temp_i16("index_water_set_min")?,
            index_water_set_max: f.temp_i16("index_water_set_max")?,
            index_smoke_t: f.temp_i16("index_smoke_t")?,
            index_power_level: f.u16("index_power_level")?,
            index_power_set: f.u16("index_power_set")?,
            index_power_min: f.u16("index_power_min")?,
            index_power_max: f.u16("index_power_max")?,
            index_fan_smoke: f.u16("index_fan_smoke")?,
            index_fan_1: f.u16("index_fan_1")?,
            index_fan_1_set: f.u16("index_fan_1_set")?,
            index_fan_1_set_max: f.u16("index_fan_1_set_max")?,
            index_fan_2: f.u16("index_fan_2")?,
            index_fan_2_set: f.u16("index_fan_2_set")?,
            index_fan_2_set_max: f.u16("index_fan_2_set_max")?,
            index_fan_3: f.u16("index_fan_3")?,
            index_fan_3_set: f.u16("index_fan_3_set")?,
            index_fan_3_set_max: f.u16("index_fan_3_set_max")?,
            boiler_enabled: t & (1 << 6) != 0,
            domestic_hot_water_enabled: t & (1 << 5) != 0,
            fan_number: (t >> 2) & 0b11,
            temp_room1_enabled: t & (1 << 0) != 0,
            temp_room2_enabled: t & (1 << 8) != 0,
            temp_room3_enabled: t & (1 << 7) != 0,
            temp_water_enabled: t & (1 << 1) != 0,
            pump_enabled: t & (1 << 4) != 0,
            last_updated: now(),
        })
    }
}

/// DAT page 1: chrono programs (11 fields)
#[derive(Debug, Clone, Serialize, Default)]
pub struct DAT1Data {
    pub index_page: u16,
    /// "Function Mode" register: 0 = manual, 2 = chrono (same value as DAT0)
    pub index_chrono_mode: u16,
    pub index_program_1_temp: f32,
    pub index_program_1_temp_min: f32,
    pub index_program_1_temp_max: f32,
    pub index_program_2_temp: f32,
    pub index_program_2_temp_min: f32,
    pub index_program_2_temp_max: f32,
    pub index_program_3_temp: f32,
    pub index_program_3_temp_min: f32,
    pub index_program_3_temp_max: f32,
    pub last_updated: String,
}

impl DAT1Data {
    pub fn from_slice(data: &[String]) -> Result<Self, DataError> {
        let mut f = Fields::new("DAT1", data, 11)?;
        Ok(Self {
            index_page: f.u16("index_page")?,
            index_chrono_mode: f.u16("index_chrono_mode")?,
            index_program_1_temp: f.temp_u16("index_program_1_temp")?,
            index_program_1_temp_min: f.temp_u16("index_program_1_temp_min")?,
            index_program_1_temp_max: f.temp_u16("index_program_1_temp_max")?,
            index_program_2_temp: f.temp_u16("index_program_2_temp")?,
            index_program_2_temp_min: f.temp_u16("index_program_2_temp_min")?,
            index_program_2_temp_max: f.temp_u16("index_program_2_temp_max")?,
            index_program_3_temp: f.temp_u16("index_program_3_temp")?,
            index_program_3_temp_min: f.temp_u16("index_program_3_temp_min")?,
            index_program_3_temp_max: f.temp_u16("index_program_3_temp_max")?,
            last_updated: now(),
        })
    }
}

/// DAT page 2: hydraulic data and actual fan speeds (22 fields)
#[derive(Debug, Clone, Serialize, Default)]
pub struct DAT2Data {
    pub index_page: u16,
    pub index_flow_switch: u16,
    pub index_generic_pump: u16,
    /// Actual speed of fan 1 ("Fan 1" register)
    pub index_fan_1_speed: u16,
    pub index_fan_2_speed: u16,
    pub index_fan_3_speed: u16,
    pub index_puffer: f32,
    pub index_puffer_set: f32,
    pub index_puffer_set_min: f32,
    pub index_puffer_set_max: f32,
    pub index_boiler: f32,
    pub index_boiler_set: f32,
    pub index_boiler_set_min: f32,
    pub index_boiler_set_max: f32,
    /// "Sanitary Water Temperature"
    pub index_dhw: f32,
    pub index_dhw_set: f32,
    pub index_dhw_set_min: f32,
    pub index_dhw_set_max: f32,
    pub index_room_temp_3: f32,
    pub index_room_temp_3_set: f32,
    pub index_room_temp_3_set_min: f32,
    pub index_room_temp_3_set_max: f32,
    pub last_updated: String,
}

impl DAT2Data {
    pub fn from_slice(data: &[String]) -> Result<Self, DataError> {
        let mut f = Fields::new("DAT2", data, 22)?;
        Ok(Self {
            index_page: f.u16("index_page")?,
            index_flow_switch: f.u16("index_flow_switch")?,
            index_generic_pump: f.u16("index_generic_pump")?,
            index_fan_1_speed: f.u16("index_fan_1_speed")?,
            index_fan_2_speed: f.u16("index_fan_2_speed")?,
            index_fan_3_speed: f.u16("index_fan_3_speed")?,
            index_puffer: f.temp_i16("index_puffer")?,
            index_puffer_set: f.temp_i16("index_puffer_set")?,
            index_puffer_set_min: f.temp_i16("index_puffer_set_min")?,
            index_puffer_set_max: f.temp_i16("index_puffer_set_max")?,
            index_boiler: f.temp_i16("index_boiler")?,
            index_boiler_set: f.temp_i16("index_boiler_set")?,
            index_boiler_set_min: f.temp_i16("index_boiler_set_min")?,
            index_boiler_set_max: f.temp_i16("index_boiler_set_max")?,
            index_dhw: f.temp_i16("index_dhw")?,
            index_dhw_set: f.temp_i16("index_dhw_set")?,
            index_dhw_set_min: f.temp_i16("index_dhw_set_min")?,
            index_dhw_set_max: f.temp_i16("index_dhw_set_max")?,
            index_room_temp_3: f.temp_i16("index_room_temp_3")?,
            index_room_temp_3_set: f.temp_i16("index_room_temp_3_set")?,
            index_room_temp_3_set_min: f.temp_i16("index_room_temp_3_set_min")?,
            index_room_temp_3_set_max: f.temp_i16("index_room_temp_3_set_max")?,
            last_updated: now(),
        })
    }
}

/// Decoded content of a polled page
#[derive(Debug)]
pub enum CommandData {
    Inf(INFData),
    Dat0(DAT0Data),
    Dat1(DAT1Data),
    Dat2(DAT2Data),
}

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF, no reflection, no final XOR),
/// as computed by `interpreter_compute_crc` in the module firmware.
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// CRC of a frame body, formatted as sent on the wire (4 uppercase hex digits)
pub fn calculate_checksum(data: &str) -> String {
    format!("{:04X}", crc16_ccitt_false(data.as_bytes()))
}

fn serialize_stove_manufacturer<S>(manufacturer: &u16, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match StoveManufacturer::from_u16(*manufacturer) {
        Some(m) => serializer.serialize_str(&format!("{:?}", m)),
        None => serializer.serialize_u16(*manufacturer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(s: &str) -> Vec<String> {
        s.split(';').map(str::to_string).collect()
    }

    #[test]
    fn crc_standard_vector() {
        assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
    }

    #[test]
    fn inf_page() {
        let inf = INFData::from_slice(&fields("HOTTOH32;10.5.0;196")).unwrap();
        assert_eq!(inf.version, "10.5.0");
        assert_eq!(inf.signal, "196");
    }

    #[test]
    fn dat0_page_types_and_scaling() {
        let raw = "0;9;0;1;33;8;1;0;2;215;220;50;300;-15;0;0;0;0;0;0;0;1450;\
                   3;3;1;5;1200;3;3;5;0;0;0;0;0;0";
        let d = DAT0Data::from_slice(&fields(raw)).unwrap();
        assert_eq!(d.index_manufacturer, 9);
        assert!(d.index_valid);
        assert_eq!(d.index_stove_state, StoveState::Power);
        assert_eq!(d.index_chrono_mode, 2);
        assert_eq!(d.index_ambient_t1, 21.5);
        assert_eq!(d.index_ambient_t2, -1.5);
        assert_eq!(d.index_smoke_t, 145.0);
        assert_eq!(d.index_power_max, 5);
        assert_eq!(d.index_fan_1_set_max, 5);
    }

    #[test]
    fn dat0_wrong_count_is_an_error() {
        assert!(matches!(
            DAT0Data::from_slice(&fields("0;9;0")),
            Err(DataError::FieldCount { got: 3, .. })
        ));
    }

    #[test]
    fn dat1_page_is_aligned() {
        let d = DAT1Data::from_slice(&fields("1;2;200;70;300;180;70;300;65535;70;300")).unwrap();
        assert_eq!(d.index_chrono_mode, 2);
        assert_eq!(d.index_program_1_temp, 20.0);
        assert_eq!(d.index_program_2_temp, 18.0);
        assert_eq!(d.index_program_3_temp, 6553.5);
        assert_eq!(d.index_program_3_temp_max, 30.0);
    }

    #[test]
    fn dat2_page() {
        let raw = "2;0;1;3;0;0;450;600;300;800;-10;0;0;0;0;0;0;0;195;200;70;300";
        let d = DAT2Data::from_slice(&fields(raw)).unwrap();
        assert_eq!(d.index_fan_1_speed, 3);
        assert_eq!(d.index_puffer, 45.0);
        assert_eq!(d.index_boiler, -1.0);
        assert_eq!(d.index_room_temp_3_set_max, 30.0);
    }

    #[test]
    fn out_of_range_values_are_rejected_not_wrapped() {
        let raw = "0;70000;0;1;33;8;1;0;2;215;220;50;300;-15;0;0;0;0;0;0;0;1450;\
                   3;3;1;5;1200;3;3;5;0;0;0;0;0;0";
        assert!(matches!(
            DAT0Data::from_slice(&fields(raw)),
            Err(DataError::InvalidField {
                field: "index_manufacturer",
                ..
            })
        ));
    }
}
