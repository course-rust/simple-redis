use std::ops::Deref;

use bytes::BytesMut;

use crate::{RespDecode, RespEncode, RespError};

use super::{extract_simple_frame_data, CRLF_LEN};

#[derive(Debug, PartialEq, Clone)]
pub struct SimpleError(pub(crate) String);

/// Errors "-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}
// - Errors "-Error message\r\n"
impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end]);

        Ok(SimpleError::new(s))
    }

    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}

impl From<&str> for SimpleError {
    fn from(s: &str) -> Self {
        SimpleError(s.to_string())
    }
}

impl Deref for SimpleError {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_error_message_encode() {
        let s: RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(s.encode(), b"-Error message\r\n");
    }

    #[test]
    fn test_simple_error_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error message\r\n");

        let frame = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message"));

        buf.extend_from_slice(b"-Error message\r");
        let ret = SimpleError::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame: SimpleError = SimpleError::decode(&mut buf)?;
        assert_eq!(frame, SimpleError::new("Error message"));

        anyhow::Ok(())
    }
}
