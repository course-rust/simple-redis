use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespEncode, RespError, RespFrame, SimpleString};

use super::BUF_CAP;
use super::{calc_total_len, parse_length, CRLF_LEN};

#[derive(Debug, PartialEq, Clone)]
pub struct RespMap(pub(crate) HashMap<String, RespFrame>);

/// map "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
/// key 仅支持 String， 使用SimpleString
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }

        buf
    }
}
// - map "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";

    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_len(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut frame = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frame.insert(key.0, value);
        }
        Ok(frame)
    }

    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_len(buf, end, len, Self::PREFIX)?;
        Ok(total_len)
    }
}

impl Deref for RespMap {
    type Target = HashMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(HashMap::new())
    }
}
impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BulkString, RespMap};
    use bytes::BytesMut;

    // #[test]
    // fn test_map_encode() {
    //     let mut map = RespMap::new();
    //     map.insert("hello".to_string(), BulkString::new("world").into());
    //     map.insert("foo".to_string(), (-123456.789).into());
    //     let frame: RespFrame = map.into();
    //     assert_eq!(
    //         frame.encode(),
    //         b"%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n"
    //     );
    // }

    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");

        let frame = RespMap::decode(&mut buf)?;

        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new(b"world".to_vec()).into(),
        );
        map.insert("foo".to_string(), BulkString::new(b"bar".to_vec()).into());

        assert_eq!(frame, map);

        anyhow::Ok(())
    }
}
