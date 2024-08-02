use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError, RespFrame};

use super::{calc_total_len, extract_fixed_data, parse_length, BUF_CAP, CRLF_LEN};

#[derive(Debug, PartialEq, Clone)]
pub struct RespArray(pub(crate) Vec<RespFrame>);
#[derive(Debug, PartialEq, Clone)]
pub struct RespNullArray;

///  Array A `*` character as the first byte, followed by the number of elements in the array as a decimal number,
///   followed by CRLF.
///   An additional RESP type for every element of the Array.
///  `*<number-of-element>\r\n<element-1>...<element-n>`
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

// array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let total_len = calc_total_len(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut frames: Vec<RespFrame> = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_len(buf, end, len, Self::PREFIX)
    }
}

/// - NullArray "*-1\r\n"
impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", "RespNullArray")?;
        Ok(RespNullArray)
    }

    fn expect_length(_buf: &[u8]) -> anyhow::Result<usize, RespError> {
        Ok(5)
    }
}

///  NullArray "*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespNullArray {
    pub fn new() -> Self {
        RespNullArray
    }
}
impl Default for RespNullArray {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BulkString, RespFrame, SimpleString};

    #[test]
    fn test_array_encode() {
        let s: RespFrame = RespArray::new(vec![
            SimpleString::new("set").into(),
            SimpleString::new("hello").into(),
            SimpleString::new("world").into(),
        ])
        .into();
        assert_eq!(s.encode(), b"*3\r\n+set\r\n+hello\r\n+world\r\n");
    }

    #[test]
    fn test_null_array_encode() {
        let s: RespFrame = RespNullArray::new().into();
        assert_eq!(s.encode(), b"*-1\r\n");
    }
    #[test]
    fn test_null_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        let frame = RespNullArray::decode(&mut buf)?;
        assert_eq!(frame, RespNullArray);

        anyhow::Ok(())
    }

    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new([
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into()
            ])
        );

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::new([
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into()
            ])
        );

        anyhow::Ok(())
    }
}
