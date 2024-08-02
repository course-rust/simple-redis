use super::{extract_simple_frame_data, CRLF_LEN};
use crate::{RespDecode, RespEncode, RespError};
use bytes::BytesMut;

///  Integers This type is just a CRLF terminated string representing an integer, prefixed by a ":" byte.
/// integer: ":[<+|->]<value>\r\n" For example ":0\r\n", or ":1000\r\n" are integer replies.
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}
// - Integers This type is just a CRLF terminated string representing an integer, prefixed by a ":" byte.
//   ":[<+|->]<value>\r\n"
//   For example ":0\r\n", or ":1000\r\n" are integer replies.
impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        // let (end, s) = parse_length(buf, Self::PREFIX)?;
        let end: usize = extract_simple_frame_data(buf, Self::PREFIX)?;
        let data = buf.split_to(end + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);

        Ok(s.parse::<Self>()?)
    }

    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;

    #[test]
    fn test_integer_encode() {
        let s: RespFrame = 123.into();
        assert_eq!(s.encode(), b":+123\r\n");

        let s: RespFrame = (-123).into();
        assert_eq!(s.encode(), b":-123\r\n");
    }
    #[test]
    fn test_integer_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b":+123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 123);

        buf.extend_from_slice(b":-123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -123);

        anyhow::Ok(())
    }
}
