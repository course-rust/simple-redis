use anyhow::Result;
use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use thiserror::Error;

pub use self::{
    array::{RespArray, RespNullArray},
    bulk_string::{BulkString, RespNullBulkString},
    frame::RespFrame,
    map::RespMap,
    null::RespNull,
    set::RespSet,
    simple_error::SimpleError,
    simple_string::SimpleString,
};

mod array;
mod bool;
mod bulk_string;
mod double;
mod frame;
mod integer;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

const BUF_CAP: usize = 4096_usize;
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

/// 编码
#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}
/// 解码
pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}
#[derive(Error, Debug, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(isize),
    #[error("Frame is not complete")]
    NotComplete,

    #[error("parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("parse bulk string error: {0}")]
    ParseBulkStringError(#[from] std::str::Utf8Error),
}

fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<usize, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }
    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: SimpleString({}), got: {:?}",
            prefix, buf
        )));
    }
    let mut end = 0_usize;
    for i in 0..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            end = i;
            break;
        }
    }
    if end == 0 {
        return Err(RespError::NotComplete);
    }
    Ok(end)
}

pub fn calc_total_len(
    buf: &[u8],
    end: usize,
    len: usize,
    prefix: &str,
) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            // For array or set, we need to calculate each element length.
            for _ in 0..len {
                let len = RespFrame::expect_length(data)?;
                data = &data[len..];
                total += len;
            }
            Ok(total)
        }
        "%" => {
            // Find nth CRLF in the buffer. For map, we need to find 2 CRLF for each key-value pair.
            for _ in 0..len {
                let len1 = SimpleString::expect_length(data)?;
                data = &data[len1..];
                total += len1;

                let len2 = RespFrame::expect_length(data)?;
                data = &data[len2..];
                total += len2;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

pub fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end: usize = extract_simple_frame_data(buf, prefix)?;
    let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
    Ok((end, s.parse()?))
}

/// Extracts a fixed amount of data from the buffer.
///
/// # Parameters
///
/// * `buf`: A mutable reference to a `BytesMut` containing the RESP data.
/// * `expect`: A string representing the expected data.
/// * `expect_type`: A string representing the type of data that is expected.
///
/// # Returns
///
/// * `Result<(), RespError>`:
///   - `Ok(())`: If the expected data is successfully extracted from the buffer.
///   - `Err(RespError)`: If the expected data is not found in the buffer or if the buffer is not complete.
pub fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }
    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}, got {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_array_length() -> Result<()> {
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let total_len = calc_total_len(buf, end, len, "*")?;
        assert_eq!(total_len, buf.len());

        let buf = b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n";
        let (end, len) = parse_length(buf, "%")?;
        let total_len = calc_total_len(buf, end, len, "%")?;
        assert_eq!(total_len, buf.len());

        anyhow::Ok(())
    }
}
