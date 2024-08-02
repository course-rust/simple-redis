use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError};

use super::{extract_fixed_data, parse_length, CRLF_LEN};

#[derive(Debug, PartialEq, Clone)]
pub struct BulkString(pub(crate) Vec<u8>);
#[derive(Debug, PartialEq, Clone)]
pub struct RespNullBulkString;

/// Bulk Strings `"$6\r\nfoobar\r\n"` `"$0\r\n\r\n"`
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}
///  NullBulkString "$-1\r\n"
impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}
// Bulk Strings `"$6\r\nfoobar\r\n"` `"$0\r\n\r\n"`
impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";

    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;

        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let data_str = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data_str[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}
// NullBulkString "$-1\r\n"
impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$";

    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(buf, "$-1\r\n", "RespNullBulkString")?;
        Ok(RespNullBulkString::new())
    }

    fn expect_length(_buf: &[u8]) -> anyhow::Result<usize, RespError> {
        Ok(5)
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Vec<u8>> for BulkString {
    fn as_ref(&self) -> &Vec<u8> {
        &self.0
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}
impl From<&[u8]> for BulkString {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec())
    }
}

impl RespNullBulkString {
    pub fn new() -> Self {
        RespNullBulkString
    }
}
impl Default for RespNullBulkString {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;

    #[test]
    fn test_bulk_string_encode() {
        let s: RespFrame = BulkString::new(b"hello".to_vec()).into();
        assert_eq!(s.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_null_bulk_string_encode() {
        let s: RespFrame = RespNullBulkString::new().into();
        assert_eq!(s.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        anyhow::Ok(())
    }

    #[test]
    fn test_null_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(frame, RespNullBulkString);

        anyhow::Ok(())
    }
}
