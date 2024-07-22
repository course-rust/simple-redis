use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use bytes::BytesMut;
use enum_dispatch::enum_dispatch;

mod decode;
mod encode;

/// 编码
#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}
/// 解码
pub trait RespDecode {
    fn decode(buf: Self) -> Result<RespFrame, String>;
}

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

pub struct SimpleString(String);
pub struct SimpleError(String);
pub struct RespNull;
pub struct RespNullArray;
pub struct RespNullBulkString;
pub struct BulkString(Vec<u8>);
pub struct RespArray(Vec<RespFrame>);
pub struct RespMap(HashMap<String, RespFrame>);
pub struct RespSet(Vec<RespFrame>);

impl RespDecode for BytesMut {
    fn decode(_buf: Self) -> Result<RespFrame, String> {
        todo!()
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}
impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}
impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
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
impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
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
impl RespNull {
    pub fn new() -> Self {
        RespNull
    }
}
impl Default for RespNull {
    fn default() -> Self {
        Self::new()
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
impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(s.into())
    }
}
