use crate::hottoh::hottoh_const::Command::Dat;
use crate::hottoh::hottoh_const::{Command, CommandType};
use crate::hottoh::hottoh_structs::{
    calculate_checksum, CommandData, DAT0Data, DAT1Data, DAT2Data, DATReqResponseData, INFData,
};
use log::warn;
use std::str::FromStr;
use std::time::Instant;
use thiserror::Error;

/// Errors that can occur when processing responses from the stove
#[derive(Error, Debug)]
pub enum ResponseError {
    /// Command not implemented
    #[error("{0}")]
    NotImplemented(String),
    /// Response data has incorrect structure
    #[error("Incorrect response data: {0}")]
    IncorrectResponseStruct(String),
}

/// Request to be sent to the stove
#[derive(Debug)]
pub struct Request {
    req_id: u32,
    command: Command,
    command_type: CommandType,
    params: Vec<String>,
    sent: bool,
    sent_at: Option<Instant>,
    marked_as_deleted: bool,
}

impl PartialEq for Request {
    /// Compares two requests for equality
    ///
    /// Requests are considered equal if they have the same command, command type, and parameters
    ///
    /// # Arguments
    ///
    /// * `other` - The other request to compare with
    ///
    /// # Returns
    ///
    /// * `bool` - True if the requests are equal, false otherwise
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
            && self.command_type == other.command_type
            && self.params == other.params
    }
}

impl Request {
    /// Creates a new request
    ///
    /// # Arguments
    ///
    /// * `req_id` - Request ID
    /// * `command` - Command to send
    /// * `command_type` - Type of command (Read, Write, Execute)
    /// * `params` - Command parameters
    ///
    /// # Returns
    ///
    /// * `Request` - A new request
    pub fn new(
        req_id: u32,
        command: Command,
        command_type: CommandType,
        params: Vec<String>,
    ) -> Self {
        Self {
            req_id,
            command,
            command_type,
            params,
            sent: false,
            sent_at: None,
            marked_as_deleted: false,
        }
    }

    /// Marks the request as sent
    ///
    /// Sets the sent flag to true and records the current time
    pub fn mark_as_sent(&mut self) {
        self.sent = true;
        self.sent_at = Some(Instant::now());
    }

    /// Builds a message to be sent to the stove
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The message as bytes
    pub fn build_message(&self) -> Vec<u8> {
        let cmd_type_str = self.command_type.as_str();
        let command = self.command.as_str();
        let params = self.params.join(";") + ";";
        let length = format!("{:04X}", params.len());

        let crc_input = format!(
            "{:05}C---{}{}{}{}",
            self.req_id, length, command, cmd_type_str, params
        );
        let checksum = calculate_checksum(&crc_input);

        let message = format!(
            "#{:05}C---{}{}{}{}{}\n",
            self.req_id, length, command, cmd_type_str, params, checksum
        );

        message.into_bytes()
    }

    /// Gets the request ID
    ///
    /// # Returns
    ///
    /// * `u32` - The request ID
    pub fn get_req_id(&self) -> u32 {
        self.req_id
    }

    /// Gets the command
    ///
    /// # Returns
    ///
    /// * `&Command` - Reference to the command
    pub fn get_command(&self) -> &Command {
        &self.command
    }

    /// Gets the command type
    ///
    /// # Returns
    ///
    /// * `&CommandType` - Reference to the command type
    pub fn get_command_type(&self) -> &CommandType {
        &self.command_type
    }

    /// Gets the command parameters
    ///
    /// # Returns
    ///
    /// * `&Vec<String>` - Reference to the parameters
    pub fn get_params(&self) -> &Vec<String> {
        &self.params
    }

    /// Checks if the request has been sent
    ///
    /// # Returns
    ///
    /// * `bool` - True if the request has been sent, false otherwise
    pub fn is_sent(&self) -> bool {
        self.sent
    }

    /// Gets the time when the request was sent
    ///
    /// # Returns
    ///
    /// * `Option<Instant>` - The time when the request was sent, or None if not sent
    pub fn get_sent_at(&self) -> Option<Instant> {
        self.sent_at
    }

    /// Checks if the request is marked for deletion
    ///
    /// # Returns
    ///
    /// * `bool` - True if the request is marked for deletion, false otherwise
    pub fn is_marked_as_deleted(&self) -> bool {
        self.marked_as_deleted
    }

    /// Sets the marked for deletion flag
    ///
    /// # Arguments
    ///
    /// * `value` - The new value for the flag
    pub fn set_marked_as_deleted(&mut self, value: bool) {
        self.marked_as_deleted = value;
    }
}

/// Response received from the stove
#[allow(dead_code)]
pub struct Response {
    req_id: u32,
    command: Command,
    command_type: CommandType,
    params_len: u32,
    params: Vec<String>,
    command_data: CommandData,
    crc: String,
    crc_is_valid: bool,
    marked_as_deleted: bool,
}

impl Response {
    /// Converts response data to the appropriate CommandData type
    ///
    /// # Arguments
    ///
    /// * `data` - The response data as string slices
    /// * `command` - The command type
    ///
    /// # Returns
    ///
    /// * `Result<CommandData, ResponseError>` - The parsed command data or an error
    fn command_data_from_vec(
        data: &Vec<&str>,
        command: &Command,
    ) -> Result<CommandData, ResponseError> {
        match command {
            Command::Inf => Ok(CommandData::Inf(INFData::from_slice(data)?)),
            Command::Dat0 => Ok(CommandData::Dat0(DAT0Data::from_slice(data)?)),
            Command::Dat1 => Ok(CommandData::Dat1(DAT1Data::from_slice(data)?)),
            Command::Dat2 => Ok(CommandData::Dat2(DAT2Data::from_slice(data)?)),
            Command::DatReqResponse => Ok(CommandData::DATReqResponse(
                DATReqResponseData::from_slice(data)?,
            )),
            _ => Err(ResponseError::NotImplemented(format!(
                "Not implemented for command: {:?}, data: {:?}",
                command, &data
            ))),
        }
    }

    /// Parses a message from the stove into a Response
    ///
    /// # Arguments
    ///
    /// * `message` - The message to parse
    ///
    /// # Returns
    ///
    /// * `Result<Response, Box<dyn std::error::Error>>` - The parsed response or an error
    pub fn from_message(message: &str) -> Result<Response, Box<dyn std::error::Error>> {
        let req_id = str::parse(&message[1..6]).map_err(|_| "Invalid req_id")?;
        let req_id_char = message
            .chars()
            .nth(6)
            .ok_or("Missing req_id separator character")?;

        let params_len =
            usize::from_str_radix(&message[10..14], 16).map_err(|_| "Invalid param length")?;

        let mut command = Command::from_str(&message[14..17])?;
        let command_type = CommandType::from_str(&message[17..18])?;
        let params_section = &message[18..&message.len() - 6];
        let crc = &message[&message.len() - 5..&message.len() - 1];

        let params: Vec<&str> = params_section.split(';').collect();

        let crc_response = format!(
            "{:05}{}---{:04X}{}{}{}",
            req_id,
            req_id_char,
            params_len,
            command.as_str(),
            command_type.as_str(),
            params.join(";") + ";"
        );
        let crc_is_valid = crc == calculate_checksum(&crc_response).as_str();

        if command == Dat {
            match params.len() {
                36 => command = Command::Dat0,
                11 => command = Command::Dat1,
                22 => command = Command::Dat2,
                1 => command = Command::DatReqResponse,
                _ => {
                    warn!(
                        "Incorrect {} response structure: {}",
                        command.as_str(),
                        &message
                    );
                }
            }
        }

        let command_data = Response::command_data_from_vec(&params, &command)?;

        Ok(Response {
            req_id,
            command,
            command_type,
            params_len: params_len.try_into()?,
            params: params.iter().map(|&s| s.to_string()).collect(),
            command_data,
            crc: crc.to_string(),
            crc_is_valid,
            marked_as_deleted: false,
        })
    }

    /// Gets the request ID
    ///
    /// # Returns
    ///
    /// * `u32` - The request ID
    pub fn get_req_id(&self) -> u32 {
        self.req_id
    }

    /// Gets the command data
    ///
    /// # Returns
    ///
    /// * `&CommandData` - Reference to the command data
    pub fn get_command_data(&self) -> &CommandData {
        &self.command_data
    }

    /// Checks if the CRC is valid
    ///
    /// # Returns
    ///
    /// * `bool` - True if the CRC is valid, false otherwise
    pub fn is_crc_valid(&self) -> bool {
        self.crc_is_valid
    }

    /// Checks if the response is marked for deletion
    ///
    /// # Returns
    ///
    /// * `bool` - True if the response is marked for deletion, false otherwise
    pub fn is_marked_as_deleted(&self) -> bool {
        self.marked_as_deleted
    }

    /// Sets the marked for deletion flag
    ///
    /// # Arguments
    ///
    /// * `value` - The new value for the flag
    pub fn set_marked_as_deleted(&mut self, value: bool) {
        self.marked_as_deleted = value;
    }
}
