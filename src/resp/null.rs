use super::extract_fixed_data;
use crate::{RespDecode, RespEncode, RespError};
use bytes::BytesMut;

#[derive(Debug, PartialEq, Clone)]
pub struct RespNull;

///  Null "_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

// - Null "_\r\n"
impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";

    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        extract_fixed_data(buf, "_\r\n", "RespNull")?;
        Ok(RespNull)
    }

    fn expect_length(_buf: &[u8]) -> anyhow::Result<usize, RespError> {
        Ok(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;

    #[test]
    fn test_null_encode() {
        let s: RespFrame = RespNull.into();
        assert_eq!(s.encode(), b"_\r\n");
    }
    #[test]
    fn test_null_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"_\r\n");

        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);

        anyhow::Ok(())
    }
}
