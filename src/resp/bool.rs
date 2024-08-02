use bytes::BytesMut;

use crate::{RespDecode, RespEncode, RespError};

use super::extract_fixed_data;

///  boolean "#<t|f>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}
// - boolean "#<t|f>\r\n"
impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }

    fn expect_length(_buf: &[u8]) -> anyhow::Result<usize, RespError> {
        Ok(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespFrame;
    use bytes::BufMut;

    #[test]
    fn test_bool_encode() {
        let s: RespFrame = true.into();
        assert_eq!(s.encode(), b"#t\r\n");

        let s: RespFrame = false.into();
        assert_eq!(s.encode(), b"#f\r\n");
    }
    #[test]
    fn test_boolean_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"#t\r\n");

        let frame = bool::decode(&mut buf)?;
        assert!(frame);

        buf.extend_from_slice(b"#f\r\n");

        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        buf.extend_from_slice(b"#f\r");
        let ret = bool::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        anyhow::Ok(())
    }
}
