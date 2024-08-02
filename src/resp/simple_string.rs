use bytes::BytesMut;
use std::ops::Deref;

use super::{extract_simple_frame_data, CRLF_LEN};
use crate::{RespDecode, RespEncode, RespError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleString(pub(crate) String);

///  Simple Strings The general form is `+<string>\r\n`, so "hello world" is encoded as
/// `+hello world<CR><LF>` Or as an escaped string:  `"+hello world\r\n"`
///   "+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}
// - Simple Strings "+OK\r\n"
impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;

        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);

        Ok(SimpleString::new(s))
    }
    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl From<&str> for SimpleString {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string())
    }
}

impl AsRef<[u8]> for SimpleString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SimpleString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;
    use bytes::BufMut;

    use super::*;

    #[test]
    fn test_simple_string_encode() {
        let s: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(s.encode(), b"+OK\r\n");
    }
    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");

        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK".to_string()));

        buf.extend_from_slice(b"+hello\r");

        let ret = SimpleString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("hello"));

        anyhow::Ok(())
    }
}
