/// Redis RESP 编码
///
use crate::{RespArray, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet};

use super::{BulkString, RespEncode, SimpleError, SimpleString};

const BUF_CAP: usize = 4096_usize;

///  Simple Strings The general form is `+<string>\r\n`, so "hello world" is encoded as
/// `+hello world<CR><LF>` Or as an escaped string:  `"+hello world\r\n"`
///   "+OK\r\n"
impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

/// Errors "-Error message\r\n"
impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

///  big number "([+|-]<number>\r\n"
///  Integers This type is just a CRLF terminated string representing an integer, prefixed by a ":" byte.
/// integer: ":[<+|->]<value>\r\n" For example ":0\r\n", or ":1000\r\n" are integer replies.
impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}
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
///  Null "_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}
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
///  NullArray "*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}
///  boolean "#<t|f>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}
///  double ",[<+|->]<integral>[.<fractional>][<E|e>[sign][exponent]]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

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
/// set "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_simple_string_encode() {
        let s: RespFrame = SimpleString::new("OK".to_string()).into();
        assert_eq!(s.encode(), b"+OK\r\n");
    }
    #[test]
    fn test_error_message_encode() {
        let s: RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(s.encode(), b"-Error message\r\n");
    }
    #[test]
    fn test_integer_encode() {
        let s: RespFrame = 123.into();
        assert_eq!(s.encode(), b":+123\r\n");

        let s: RespFrame = (-123).into();
        assert_eq!(s.encode(), b":-123\r\n");
    }
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
    fn test_null_encode() {
        let s: RespFrame = RespNull::new().into();
        assert_eq!(s.encode(), b"_\r\n");
    }
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
    fn test_bool_encode() {
        let s: RespFrame = true.into();
        assert_eq!(s.encode(), b"#t\r\n");

        let s: RespFrame = false.into();
        assert_eq!(s.encode(), b"#f\r\n");
    }
    #[test]
    fn test_double_encode() {
        let s: RespFrame = 123.456.into();
        assert_eq!(s.encode(), b",+123.456\r\n");

        let s: RespFrame = (-123.456).into();
        assert_eq!(s.encode(), b",-123.456\r\n");

        let s: RespFrame = 1.23456e+8.into();
        assert_eq!(s.encode(), b",+1.23456e8\r\n");

        let s: RespFrame = (-1.23456e-9).into();
        assert_eq!(s.encode(), b",-1.23456e-9\r\n");
    }
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
    fn test_set_encode() {
        let frame: RespFrame = RespSet::new([
            RespArray::new([1234.into(), true.into()]).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();

        assert_eq!(
            frame.encode(),
            b"~2\r\n*2\r\n:+1234\r\n#t\r\n$5\r\nworld\r\n"
        );
    }
}
