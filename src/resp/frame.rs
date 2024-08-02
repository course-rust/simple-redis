use super::{
    BulkString, RespArray, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet,
    SimpleError, SimpleString,
};
use crate::{RespDecode, RespError};
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

///
/// - Simple Strings "+OK\r\n"
/// - Errors "-Error message\r\n"
/// - Integers This type is just a CRLF terminated string representing an integer, prefixed by a ":" byte.
///   For example ":0\r\n", or ":1000\r\n" are integer replies.
/// - Bulk Strings "$6\r\nfoobar\r\n" $0\r\n\r\n"
/// - NullBulkString "$-1\r\n"
/// - Array A `*` character as the first byte, followed by the number of elements in the array as a decimal number,
///   followed by CRLF.
///   An additional RESP type for every element of the Array.
/// - NullArray "*-1\r\n"
/// - Null "_\r\n"
/// - boolean "#<t|f>\r\n"
/// - double ",[<+|->]<integral>[.<fractional>][<E|e>[sign][exponent]]\r\n"
/// - big number "([+|-]<number>\r\n"
/// - map "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
/// - set "~<number-of-elements>\r\n<element-1>...<element-n>"
///
#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, Clone)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),

    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";

    fn decode(buf: &mut BytesMut) -> anyhow::Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;

                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => {
                // try null bulk string first
                match RespNullArray::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = BulkString::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'*') => {
                // try null array first
                match RespNullArray::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespError::NotComplete) => Err(RespError::NotComplete),
                    Err(_) => {
                        let frame = RespArray::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = f64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            None => Err(RespError::NotComplete),
            _ => Err(RespError::InvalidFrameType(format!(
                "Invalid frame type: {:?}",
                buf
            ))),
        }
    }
    fn expect_length(buf: &[u8]) -> anyhow::Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),

            _ => Err(RespError::NotComplete),
        }
    }
}

impl From<&str> for RespFrame {
    fn from(s: &str) -> Self {
        SimpleString(s.to_string()).into()
    }
}
impl From<&[u8]> for RespFrame {
    fn from(s: &[u8]) -> Self {
        BulkString(s.to_vec()).into()
    }
}
impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        BulkString(s.to_vec()).into()
    }
}
