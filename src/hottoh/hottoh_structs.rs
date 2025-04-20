use super::hottoh_const::*;
use crate::hottoh::tcp_client_structs::ResponseError;
use chrono::{Local, SecondsFormat};
use crc_any::CRCu16;
use serde::{Deserialize, Serialize, Serializer};
use std::str;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct INFData {
    hostname: String,
    version: String,
    signal: String,
    last_updated: String,
}

impl INFData {
    pub fn from_slice(response_data: &[&str]) -> Result<Self, ResponseError> {
        if response_data.len() != 3 {
            return Err(ResponseError::IncorrectResponseStruct(
                "Incorrect number of elements".to_string(),
            ));
        }

        Ok(Self {
            hostname: response_data[0].to_string(),
            version: response_data[1].to_string(),
            signal: response_data[2].to_string(),
            last_updated: Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        })
    }
}

#[derive(Default)]
#[allow(dead_code)]
pub struct DATReqResponseData {
    value: String,
}

impl DATReqResponseData {
    pub fn from_slice(response_data: &[&str]) -> Result<Self, ResponseError> {
        if response_data.len() != 1 {
            return Err(ResponseError::IncorrectResponseStruct(
                "Incorrect number of elements".to_string(),
            ));
        }
        Ok(Self {
            value: response_data[0].to_string(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DAT0Data {
    index_page: u16,
    #[serde(serialize_with = "serialize_stove_manufacturer")]
    index_manufacturer: u16,
    index_bitmap_visible: bool,
    index_valid: bool,
    index_stove_type: u16,
    index_stove_state: StoveState,
    index_stove_on: bool,
    index_eco_mode: bool,
    index_timer_on: u16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t1: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t1_set: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t1_set_min: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t1_set_max: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t2: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t2_set: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t2_set_min: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_ambient_t2_set_max: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_water: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_water_set: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_water_set_min: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_water_set_max: i16,
    #[serde(serialize_with = "serialize_i16_as_f32")]
    index_smoke_t: i16,
    index_power_level: u16,
    index_power_set: u16,
    index_power_min: u16,
    index_power_max: u16,
    index_fan_smoke: u16,
    index_fan_1: u16,
    index_fan_1_set: u16,
    index_fan_1_set_max: u16,
    index_fan_2: u16,
    index_fan_2_set: u16,
    index_fan_2_set_max: u16,
    index_fan_3: u16,
    index_fan_3_set: u16,
    index_fan_3_set_max: u16,
    boiler_enabled: bool,
    domestic_hot_water_enabled: bool,
    fan_number: u16,
    temp_room1_enabled: bool,
    temp_room2_enabled: bool,
    temp_room3_enabled: bool,
    temp_water_enabled: bool,
    pump_enabled: bool,
    last_updated: String,
}

impl DAT0Data {
    pub fn from_slice(response_data: &[&str]) -> Result<Self, ResponseError> {
        if response_data.len() != 36 {
            return Err(ResponseError::IncorrectResponseStruct(
                "Incorrect number of elements in DAT0 struct".to_string(),
            ));
        }
        let index_stove_type: u16 = response_data[4].parse().map_err(|_| {
            ResponseError::IncorrectResponseStruct(format!(
                "Invalid index_stove_type: {}",
                response_data[4]
            ))
        })?;

        // let boiler_enabled = (index_stove_type & (1 << 9)) != 0;
        // let domestic_hot_water_enabled = (index_stove_type & (1 << 10)) != 0;
        // let fan_number = (index_stove_type >> 12) & 0b11;
        // let temp_room1_enabled = (index_stove_type & (1 << 15)) != 0;
        // let temp_room2_enabled = (index_stove_type & (1 << 8)) != 0;
        // let temp_room3_enabled = (index_stove_type & (1 << 7)) != 0;
        // let temp_water_enabled = (index_stove_type & (1 << 14)) != 0;
        // let pump_enabled = (index_stove_type & (1 << 4)) != 0;
        let boiler_enabled = (index_stove_type & (1 << 6)) != 0;
        let domestic_hot_water_enabled = (index_stove_type & (1 << 5)) != 0;
        let fan_number = (index_stove_type >> 2) & 0b11;
        let temp_room1_enabled = (index_stove_type & (1 << 0)) != 0;
        let temp_room2_enabled = (index_stove_type & (1 << 8)) != 0;
        let temp_room3_enabled = (index_stove_type & (1 << 7)) != 0;
        let temp_water_enabled = (index_stove_type & (1 << 1)) != 0;
        let pump_enabled = (index_stove_type & (1 << 4)) != 0;
        Ok(Self {
            index_page: response_data[0].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_page: {}",
                    response_data[0]
                ))
            })?,
            index_manufacturer: response_data[1].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_manufacturer: {}",
                    response_data[1]
                ))
            })?,
            index_bitmap_visible: parse_bool(response_data[2]).map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_bitmap_visible: {}",
                    response_data[2]
                ))
            })?,
            index_valid: parse_bool(response_data[3]).map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_valid: {}",
                    response_data[3]
                ))
            })?,
            index_stove_type,
            index_stove_state: response_data[5].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_stove_state: {}",
                    response_data[5]
                ))
            })?,
            index_stove_on: parse_bool(response_data[6]).map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_stove_on: {}",
                    response_data[6]
                ))
            })?,
            index_eco_mode: parse_bool(response_data[7]).map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_eco_mode: {}",
                    response_data[7]
                ))
            })?,
            index_timer_on: response_data[8].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_timer_on: {}",
                    response_data[8]
                ))
            })?,
            index_ambient_t1: response_data[9].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t1: {}",
                    response_data[9]
                ))
            })?,
            index_ambient_t1_set: response_data[10].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t1_set: {}",
                    response_data[10]
                ))
            })?,
            index_ambient_t1_set_min: response_data[11].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t1_set_min: {}",
                    response_data[11]
                ))
            })?,
            index_ambient_t1_set_max: response_data[12].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t1_set_max: {}",
                    response_data[12]
                ))
            })?,
            index_ambient_t2: response_data[13].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t2: {}",
                    response_data[13]
                ))
            })?,
            index_ambient_t2_set: response_data[14].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t2_set: {}",
                    response_data[14]
                ))
            })?,
            index_ambient_t2_set_min: response_data[15].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t2_set_min: {}",
                    response_data[15]
                ))
            })?,
            index_ambient_t2_set_max: response_data[16].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_ambient_t2_set_max: {}",
                    response_data[16]
                ))
            })?,
            index_water: response_data[17].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_water: {}",
                    response_data[17]
                ))
            })?,
            index_water_set: response_data[18].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_water_set: {}",
                    response_data[18]
                ))
            })?,
            index_water_set_min: response_data[19].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_water_set_min: {}",
                    response_data[19]
                ))
            })?,
            index_water_set_max: response_data[20].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_water_set_max: {}",
                    response_data[20]
                ))
            })?,
            index_smoke_t: response_data[21].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_smoke_t: {}",
                    response_data[21]
                ))
            })?,
            index_power_level: response_data[22].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_power_level: {}",
                    response_data[22]
                ))
            })?,
            index_power_set: response_data[23].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_power_set: {}",
                    response_data[23]
                ))
            })?,
            index_power_min: response_data[24].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_power_min: {}",
                    response_data[24]
                ))
            })?,
            index_power_max: response_data[25].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_power_max: {}",
                    response_data[25]
                ))
            })?,
            index_fan_smoke: response_data[26].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_smoke: {}",
                    response_data[26]
                ))
            })?,
            index_fan_1: response_data[27].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_1: {}",
                    response_data[27]
                ))
            })?,
            index_fan_1_set: response_data[28].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_1_set: {}",
                    response_data[28]
                ))
            })?,
            index_fan_1_set_max: response_data[29].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_1_set_max: {}",
                    response_data[29]
                ))
            })?,
            index_fan_2: response_data[30].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_2: {}",
                    response_data[30]
                ))
            })?,
            index_fan_2_set: response_data[31].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_2_set: {}",
                    response_data[31]
                ))
            })?,
            index_fan_2_set_max: response_data[32].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_2_set_max: {}",
                    response_data[32]
                ))
            })?,
            index_fan_3: response_data[33].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_3: {}",
                    response_data[33]
                ))
            })?,
            index_fan_3_set: response_data[34].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_3_set: {}",
                    response_data[34]
                ))
            })?,
            index_fan_3_set_max: response_data[35].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_fan_3_set_max: {}",
                    response_data[35]
                ))
            })?,
            boiler_enabled,
            domestic_hot_water_enabled,
            fan_number,
            temp_room1_enabled,
            temp_room2_enabled,
            temp_room3_enabled,
            temp_water_enabled,
            pump_enabled,
            last_updated: Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        })
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DAT1Data {
    index_page: i16,
    index_state: bool,
    index_temperature_1: i16,
    index_temperature_1_min: i16,
    index_temperature_1_max: i16,
    index_temperature_2: i16,
    index_temperature_2_min: i16,
    index_temperature_2_max: i16,
    index_temperature_3: i16,
    index_temperature_3_min: i16,
    index_temperature_3_max: i16,
    last_updated: String,
}

impl DAT1Data {
    pub fn from_slice(response_data: &[&str]) -> Result<Self, ResponseError> {
        if response_data.len() != 11 {
            return Err(ResponseError::IncorrectResponseStruct(
                "Incorrect number of elements in DAT1 struct".to_string(),
            ));
        }

        Ok(Self {
            index_page: response_data[0].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_page: {}",
                    response_data[0]
                ))
            })?,
            index_state: parse_bool(response_data[0]).map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_state: {}",
                    response_data[0]
                ))
            })?,
            index_temperature_1: response_data[1].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_1: {}",
                    response_data[1]
                ))
            })?,
            index_temperature_1_min: response_data[2].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_1_min: {}",
                    response_data[2]
                ))
            })?,
            index_temperature_1_max: response_data[3].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_1_max: {}",
                    response_data[3]
                ))
            })?,
            index_temperature_2: response_data[4].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_2: {}",
                    response_data[4]
                ))
            })?,
            index_temperature_2_min: response_data[5].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_2_min: {}",
                    response_data[5]
                ))
            })?,
            index_temperature_2_max: response_data[6].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_2_max: {}",
                    response_data[6]
                ))
            })?,
            index_temperature_3: response_data[7].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_3: {}",
                    response_data[7]
                ))
            })?,
            index_temperature_3_min: response_data[8].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_3_min: {}",
                    response_data[8]
                ))
            })?,
            index_temperature_3_max: response_data[9].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_temperature_3_max: {}",
                    response_data[9]
                ))
            })?,
            last_updated: Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        })
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DAT2Data {
    index_page: i16,
    index_flow_switch: u16,
    index_generic_pump: u16,
    index_airex_1: u16,
    index_airex_2: u16,
    index_airex_3: u16,
    index_puffer: i16,
    index_puffer_set: i16,
    index_puffer_set_min: i16,
    index_puffer_set_max: i16,
    index_boiler: i16,
    index_boiler_set: i16,
    index_boiler_set_min: i16,
    index_boiler_set_max: i16,
    index_dhw: i16,
    index_dhw_set: i16,
    index_dhw_set_min: i16,
    index_dhw_set_max: i16,
    index_room_temp_3: i16,
    index_room_temp_3_set: i16,
    index_room_temp_3_set_min: i16,
    index_room_temp_3_set_max: i16,
    last_updated: String,
}

impl DAT2Data {
    pub fn from_slice(response_data: &[&str]) -> Result<Self, ResponseError> {
        if response_data.len() != 22 {
            return Err(ResponseError::IncorrectResponseStruct(
                "Incorrect number of elements in DAT2 struct".to_string(),
            ));
        }

        Ok(Self {
            index_page: response_data[0].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_page: {}",
                    response_data[0]
                ))
            })?,
            index_flow_switch: response_data[1].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_flow_switch: {}",
                    response_data[1]
                ))
            })?,
            index_generic_pump: response_data[2].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_generic_pump: {}",
                    response_data[2]
                ))
            })?,
            index_airex_1: response_data[3].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_airex_1: {}",
                    response_data[3]
                ))
            })?,
            index_airex_2: response_data[4].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_airex_2: {}",
                    response_data[4]
                ))
            })?,
            index_airex_3: response_data[5].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_airex_3: {}",
                    response_data[5]
                ))
            })?,
            index_puffer: response_data[6].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_puffer: {}",
                    response_data[6]
                ))
            })?,
            index_puffer_set: response_data[7].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_puffer_set: {}",
                    response_data[7]
                ))
            })?,
            index_puffer_set_min: response_data[8].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_puffer_set_min: {}",
                    response_data[8]
                ))
            })?,
            index_puffer_set_max: response_data[9].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_puffer_set_max: {}",
                    response_data[9]
                ))
            })?,
            index_boiler: response_data[10].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_boiler: {}",
                    response_data[10]
                ))
            })?,
            index_boiler_set: response_data[11].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_boiler_set: {}",
                    response_data[11]
                ))
            })?,
            index_boiler_set_min: response_data[12].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_boiler_set_min: {}",
                    response_data[12]
                ))
            })?,
            index_boiler_set_max: response_data[13].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_boiler_set_max: {}",
                    response_data[13]
                ))
            })?,
            index_dhw: response_data[14].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_dhw: {}",
                    response_data[14]
                ))
            })?,
            index_dhw_set: response_data[15].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_dhw_set: {}",
                    response_data[15]
                ))
            })?,
            index_dhw_set_min: response_data[16].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_dhw_set_min: {}",
                    response_data[16]
                ))
            })?,
            index_dhw_set_max: response_data[17].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_dhw_set_max: {}",
                    response_data[17]
                ))
            })?,
            index_room_temp_3: response_data[18].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_room_temp_3: {}",
                    response_data[18]
                ))
            })?,
            index_room_temp_3_set: response_data[19].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_room_temp_3_set: {}",
                    response_data[19]
                ))
            })?,
            index_room_temp_3_set_min: response_data[20].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_room_temp_3_set_min: {}",
                    response_data[20]
                ))
            })?,
            index_room_temp_3_set_max: response_data[21].parse().map_err(|_| {
                ResponseError::IncorrectResponseStruct(format!(
                    "Invalid index_room_temp_3_set_max: {}",
                    response_data[21]
                ))
            })?,
            last_updated: Local::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        })
    }
}

#[allow(dead_code)]
pub enum CommandData {
    Inf(INFData),
    Dat0(DAT0Data),
    Dat1(DAT1Data),
    Dat2(DAT2Data),
    DATReqResponse(DATReqResponseData),
}

pub fn calculate_checksum(data: &str) -> String {
    let mut crc = CRCu16::crc16ccitt_false();
    crc.digest(data.as_bytes());
    format!("{:04X}", crc.get_crc())
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("Invalid boolean string: {}", s)),
    }
}

fn serialize_i16_as_f32<S>(value: &i16, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_as_f32 = (*value as f32) / 10.0;
    serializer.serialize_f32(value_as_f32)
}

fn serialize_stove_manufacturer<S>(manufacturer: &u16, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(manufacturer_enum) = StoveManufacturer::from_u16(*manufacturer) {
        serializer.serialize_str(&format!("{:?}", manufacturer_enum))
    } else {
        serializer.serialize_u16(*manufacturer)
    }
}
