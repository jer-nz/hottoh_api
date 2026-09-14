//! Frame encoding and decoding for the HottoH local TCP protocol.
//!
//! Frame layout (firmware format `#%05d%4s%04X%3s%1s%s%04X\n`):
//!
//! ```text
//! #<req id, 5 digits><desc, 4 chars><params length, 4 hex><CMD><R|W|E><params;...;><CRC, 4 hex>\n
//! ```
//!
//! The CRC covers everything between `#` and the CRC itself. The firmware rejects a frame whose
//! size is not `params length + 23` and answers nothing to invalid frames.

use crate::hottoh::hottoh_const::{Command, CommandType};
use crate::hottoh::hottoh_structs::{
    CommandData, DAT0Data, DAT1Data, DAT2Data, DataError, INFData, WriteResult, calculate_checksum,
};
use std::str::FromStr;
use thiserror::Error;

/// Fixed part of a frame: `#` + id (5) + desc (4) + length (4) + cmd (3) + type (1) + CRC (4) + `\n`
const FRAME_OVERHEAD: usize = 23;

/// Largest buffered data without a complete frame before it is discarded
const MAX_BUFFER: usize = 8 * 1024;

/// Error while decoding a frame
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("frame too short ({0} bytes)")]
    TooShort(usize),
    #[error("frame is not ASCII")]
    NotAscii,
    #[error("frame must start with '#' and end with '\\n'")]
    BadDelimiters,
    #[error("invalid {0} field")]
    BadField(&'static str),
    #[error("announced parameter length {announced} does not match frame size {size}")]
    LengthMismatch { announced: usize, size: usize },
    #[error("CRC mismatch: frame has {received}, computed {computed}")]
    CrcMismatch { received: String, computed: String },
    #[error("{0}")]
    Unsupported(String),
    #[error(transparent)]
    Data(#[from] DataError),
}

/// Request sent to the stove
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    req_id: u32,
    command: Command,
    command_type: CommandType,
    params: Vec<String>,
}

impl Request {
    /// Creates a request. `req_id` is reduced modulo 100000 to fit the 5-digit field.
    pub fn new(
        req_id: u32,
        command: Command,
        command_type: CommandType,
        params: Vec<String>,
    ) -> Self {
        Self {
            req_id: req_id % 100_000,
            command,
            command_type,
            params,
        }
    }

    /// Encodes the request as a frame
    pub fn build_message(&self) -> Vec<u8> {
        let params = self.params.join(";") + ";";
        let body = format!(
            "{:05}C---{:04X}{}{}{}",
            self.req_id,
            params.len(),
            self.command.as_str(),
            self.command_type.as_str(),
            params
        );
        let checksum = calculate_checksum(&body);
        format!("#{}{}\n", body, checksum).into_bytes()
    }

    pub fn get_req_id(&self) -> u32 {
        self.req_id
    }

    pub fn get_command(&self) -> Command {
        self.command
    }

    pub fn get_command_type(&self) -> CommandType {
        self.command_type
    }

    pub fn get_params(&self) -> &[String] {
        &self.params
    }
}

/// Response received from the stove, with a valid CRC
#[derive(Debug)]
pub struct Response {
    req_id: u32,
    command: Command,
    command_type: CommandType,
    params: Vec<String>,
}

impl Response {
    /// Decodes one complete frame (including the trailing `\n`). Never panics.
    pub fn from_message(message: &[u8]) -> Result<Response, ProtocolError> {
        if message.len() < FRAME_OVERHEAD + 1 {
            return Err(ProtocolError::TooShort(message.len()));
        }
        if !message.is_ascii() {
            return Err(ProtocolError::NotAscii);
        }
        // ASCII was checked above, so byte offsets are char boundaries.
        let message = std::str::from_utf8(message).map_err(|_| ProtocolError::NotAscii)?;
        if !message.starts_with('#') || !message.ends_with('\n') {
            return Err(ProtocolError::BadDelimiters);
        }

        let field = |range: std::ops::Range<usize>, name: &'static str| {
            message.get(range).ok_or(ProtocolError::BadField(name))
        };

        let req_id_str = field(1..6, "request id")?;
        if !req_id_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ProtocolError::BadField("request id"));
        }
        let req_id: u32 = req_id_str
            .parse()
            .map_err(|_| ProtocolError::BadField("request id"))?;
        let params_len = usize::from_str_radix(field(10..14, "length")?, 16)
            .map_err(|_| ProtocolError::BadField("length"))?;
        if params_len + FRAME_OVERHEAD != message.len() {
            return Err(ProtocolError::LengthMismatch {
                announced: params_len,
                size: message.len(),
            });
        }

        let body_end = message.len() - 5;
        let body = field(1..body_end, "body")?;
        let received_crc = field(body_end..message.len() - 1, "crc")?;
        let computed_crc = calculate_checksum(body);
        if !received_crc.eq_ignore_ascii_case(&computed_crc) {
            return Err(ProtocolError::CrcMismatch {
                received: received_crc.to_string(),
                computed: computed_crc,
            });
        }

        let command =
            Command::from_str(field(14..17, "command")?).map_err(ProtocolError::Unsupported)?;
        let command_type = CommandType::from_str(field(17..18, "command type")?)
            .map_err(ProtocolError::Unsupported)?;

        // Parameters always end with ';' in the firmware format: drop the last empty item.
        let params_section = field(18..18 + params_len, "parameters")?;
        let params_section = params_section.strip_suffix(';').unwrap_or(params_section);
        let params = if params_section.is_empty() {
            Vec::new()
        } else {
            params_section.split(';').map(str::to_string).collect()
        };

        Ok(Response {
            req_id,
            command,
            command_type,
            params,
        })
    }

    pub fn get_req_id(&self) -> u32 {
        self.req_id
    }

    #[cfg(test)]
    pub fn get_command(&self) -> Command {
        self.command
    }

    #[cfg(test)]
    pub fn get_command_type(&self) -> CommandType {
        self.command_type
    }

    #[cfg(test)]
    pub fn get_params(&self) -> &[String] {
        &self.params
    }

    /// Decodes the parameters according to the command. DAT pages are identified by their
    /// first field (the page number), not by their field count.
    pub fn command_data(&self) -> Result<CommandData, ProtocolError> {
        match (self.command, self.command_type) {
            (Command::Inf, _) => Ok(CommandData::Inf(INFData::from_slice(&self.params)?)),
            (Command::Dat, CommandType::Write) => {
                Ok(CommandData::Write(WriteResult::from_slice(&self.params)?))
            }
            (Command::Dat, CommandType::Read) => match self.params.first().map(String::as_str) {
                Some("0") => Ok(CommandData::Dat0(DAT0Data::from_slice(&self.params)?)),
                Some("1") => Ok(CommandData::Dat1(DAT1Data::from_slice(&self.params)?)),
                Some("2") => Ok(CommandData::Dat2(DAT2Data::from_slice(&self.params)?)),
                other => Err(ProtocolError::Unsupported(format!(
                    "unknown DAT page {:?}",
                    other
                ))),
            },
            (command, command_type) => Err(ProtocolError::Unsupported(format!(
                "{} {} answers are not supported",
                command.as_str(),
                command_type.as_str()
            ))),
        }
    }
}

/// Reassembles frames from a TCP byte stream (frames may be split or coalesced)
#[derive(Debug, Default)]
pub struct FrameBuffer {
    buffer: Vec<u8>,
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends received bytes
    pub fn extend(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        if self.buffer.len() > MAX_BUFFER && !self.buffer.contains(&b'\n') {
            log::warn!(
                "Discarding {} buffered bytes without frame end",
                self.buffer.len()
            );
            self.buffer.clear();
        }
    }

    /// Extracts the next complete frame (from `#` to `\n` included), skipping garbage
    pub fn next_frame(&mut self) -> Option<Vec<u8>> {
        loop {
            let end = self.buffer.iter().position(|&b| b == b'\n')?;
            let line: Vec<u8> = self.buffer.drain(..=end).collect();
            // A frame starts at the last '#' before the end of line.
            if let Some(start) = line.iter().rposition(|&b| b == b'#') {
                return Some(line[start..].to_vec());
            }
            if !line.iter().all(u8::is_ascii_whitespace) {
                log::warn!(
                    "Discarding data without frame start: {:?}",
                    String::from_utf8_lossy(&line)
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INF_FRAME: &[u8] = b"#00001C---0014INFRHOTTOH32;10.5.0;196;BEA1\n";

    #[test]
    fn build_inf_request() {
        let req = Request::new(1, Command::Inf, CommandType::Read, vec![]);
        let msg = String::from_utf8(req.build_message()).unwrap();
        assert!(msg.starts_with("#00001C---0001INFR;"));
        assert!(msg.ends_with('\n'));
        assert_eq!(msg.len(), 1 + FRAME_OVERHEAD);
    }

    #[test]
    fn build_write_request() {
        let req = Request::new(
            123456,
            Command::Dat,
            CommandType::Write,
            vec!["3".into(), "215".into()],
        );
        let msg = String::from_utf8(req.build_message()).unwrap();
        let body = "23456C---0006DATW3;215;";
        assert_eq!(msg, format!("#{}{}\n", body, calculate_checksum(body)));
    }

    #[test]
    fn built_frames_parse_back() {
        let req = Request::new(42, Command::Dat, CommandType::Read, vec!["0".into()]);
        let resp = Response::from_message(&req.build_message()).unwrap();
        assert_eq!(resp.get_req_id(), 42);
        assert_eq!(resp.get_params(), ["0"]);
    }

    #[test]
    fn parse_real_inf_frame() {
        let resp = Response::from_message(INF_FRAME).unwrap();
        assert_eq!(resp.get_req_id(), 1);
        assert_eq!(resp.get_command(), Command::Inf);
        assert!(matches!(resp.command_data(), Ok(CommandData::Inf(_))));
    }

    #[test]
    fn write_answers_are_decoded() {
        for (params, expected) in [
            ("OK;", WriteResult::Ok),
            ("ERR;17;", WriteResult::Error(17)),
        ] {
            let body = format!("00007C---{:04X}DATW{}", params.len(), params);
            let frame = format!("#{}{}\n", body, calculate_checksum(&body));
            let resp = Response::from_message(frame.as_bytes()).unwrap();
            match resp.command_data().unwrap() {
                CommandData::Write(result) => assert_eq!(result, expected),
                other => panic!("unexpected {:?}", other),
            }
        }
    }

    #[test]
    fn truncated_and_garbage_frames_do_not_panic() {
        for len in 0..INF_FRAME.len() {
            let _ = Response::from_message(&INF_FRAME[..len]);
            let _ = Response::from_message(&INF_FRAME[len..]);
        }
        assert!(Response::from_message(b"#\n").is_err());
        assert!(
            Response::from_message("#0000é---0014INFRHOTTOH32;10.5.0;196;BEA1\n".as_bytes())
                .is_err()
        );
    }

    #[test]
    fn bad_crc_and_length_are_rejected() {
        let mut bad_crc = INF_FRAME.to_vec();
        bad_crc[INF_FRAME.len() - 2] = b'0';
        assert!(matches!(
            Response::from_message(&bad_crc),
            Err(ProtocolError::CrcMismatch { .. })
        ));
        let bad_len = b"#00001C---0015INFRHOTTOH32;10.5.0;196;BEA1\n";
        assert!(matches!(
            Response::from_message(bad_len),
            Err(ProtocolError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn frame_buffer_reassembles_split_and_coalesced_frames() {
        let mut buf = FrameBuffer::new();
        buf.extend(&INF_FRAME[..10]);
        assert!(buf.next_frame().is_none());
        buf.extend(&INF_FRAME[10..]);
        buf.extend(INF_FRAME);
        buf.extend(b"garbage#00001");
        assert_eq!(buf.next_frame().unwrap(), INF_FRAME);
        assert_eq!(buf.next_frame().unwrap(), INF_FRAME);
        assert!(buf.next_frame().is_none());
        buf.extend(b"C---\n");
        // Incomplete garbage ending with '\n' is returned as a frame and rejected by the parser.
        let frame = buf.next_frame().unwrap();
        assert!(Response::from_message(&frame).is_err());
    }

    #[test]
    fn frame_buffer_skips_noise_before_frame() {
        let mut buf = FrameBuffer::new();
        buf.extend(b"\r\nnoise\n");
        buf.extend(INF_FRAME);
        assert_eq!(buf.next_frame().unwrap(), INF_FRAME);
    }

    /// Deterministic xorshift generator, so that failures can be reproduced
    fn xorshift(seed: &mut u64) -> u64 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        *seed
    }

    #[test]
    fn random_input_never_panics() {
        let mut seed = 0x9E37_79B9_7F4A_7C15;
        let mut buf = FrameBuffer::new();
        let alphabet = b"#\n;-0123456789ABCDEFINFDATRWOKER\xff ";
        for _ in 0..20_000 {
            let len = (xorshift(&mut seed) % 80) as usize;
            let data: Vec<u8> = (0..len)
                .map(|_| alphabet[(xorshift(&mut seed) % alphabet.len() as u64) as usize])
                .collect();
            let _ = Response::from_message(&data);
            buf.extend(&data);
            while let Some(frame) = buf.next_frame() {
                if let Ok(response) = Response::from_message(&frame) {
                    let _ = response.command_data();
                }
            }
        }
    }

    #[test]
    fn random_parameters_with_valid_crc_never_panic() {
        let mut seed = 42;
        let alphabet = b";-0123456789OKER";
        for i in 0..20_000 {
            // The firmware always ends the parameters with ';', so they are never empty
            let len = 1 + (xorshift(&mut seed) % 120) as usize;
            let params: String = (0..len)
                .map(|_| alphabet[(xorshift(&mut seed) % alphabet.len() as u64) as usize] as char)
                .collect();
            let (command, kind) = [("DAT", "R"), ("DAT", "W"), ("INF", "R"), ("DAT", "E")][i % 4];
            let body = format!("00001C---{:04X}{}{}{}", params.len(), command, kind, params);
            let frame = format!("#{}{}\n", body, calculate_checksum(&body));
            let response = Response::from_message(frame.as_bytes()).unwrap();
            let _ = response.command_data();
        }
    }
}
