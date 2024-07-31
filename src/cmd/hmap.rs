use crate::cmd::{extract_args, validate_command, HGetAll, HSet};
use crate::{
    cmd::{CommandError, HGet},
    RespArray, RespFrame,
};

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["HGET"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
            }),
            _ => Err(CommandError::InvalidCommand(
                "Invalid key or field for HGET command".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["HGETALL"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HGetAll {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidCommand(
                "Invalid key for HGETALL command".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["HSET"], 3)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HSet {
                    key: String::from_utf8(key.0)?,
                    field: String::from_utf8(field.0)?,
                    value,
                })
            }
            _ => Err(CommandError::InvalidCommand(
                "Invalid key, field or value for HSET command".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::{BulkString, RespDecode};

    use super::*;

    #[test]
    fn test_hget_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nHGET\r\n$5\r\nmykey\r\n$7\r\nmyfield\r\n");

        let frame = RespArray::decode(&mut buf).unwrap();
        let result: HGet = frame.try_into()?;
        assert_eq!(result.key, "mykey");
        assert_eq!(result.field, "myfield");

        Ok(())
    }

    #[test]
    fn test_hgetall_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nHGETALL\r\n$5\r\nmykey\r\n");

        let frame = RespArray::decode(&mut buf)?;
        let result: HGetAll = frame.try_into()?;
        assert_eq!(result.key, "mykey");

        Ok(())
    }

    #[test]
    fn test_hset_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(
            b"*4\r\n$4\r\nHSET\r\n$5\r\nmykey\r\n$7\r\nmyfield\r\n$7\r\nmyvalue\r\n",
        );

        let frame = RespArray::decode(&mut buf)?;
        let result: HSet = frame.try_into()?;
        assert_eq!(result.key, "mykey");
        assert_eq!(result.field, "myfield");
        assert_eq!(
            result.value,
            RespFrame::BulkString(BulkString::new(b"myvalue".to_vec()))
        );

        Ok(())
    }
}
