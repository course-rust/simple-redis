use crate::cmd::RESP_OK;
use crate::{
    cmd::{extract_args, validate_command, CommandError, CommandExecutor, Get, Set},
    Backend, RespArray, RespFrame, RespNull,
};

//===================  实现 CommandExecutor trait for Command
impl CommandExecutor for Get {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend.get(&self.key).unwrap_or(RespFrame::Null(RespNull))
    }
}
impl CommandExecutor for Set {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend.set(self.key.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

// =========================== 实现 TryFrom trait for Command
impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Get {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidCommand("Invalid key".to_string())),
        }
    }
}
impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;
        let args = extract_args(value, 1)?;
        let mut args = args.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(Set {
                key: String::from_utf8(key.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidCommand(
                "Invalid key or value".to_string(),
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
    fn test_get_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        let result: Get = frame.try_into()?; // Get::try_from(frame)?
        assert_eq!(result.key, "hello");

        Ok(())
    }
    #[test]
    fn test_set_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;
        let result: Set = frame.try_into()?;
        assert_eq!(result.key, "hello");
        assert_eq!(
            result.value,
            RespFrame::BulkString(BulkString::new(b"world".to_vec()))
        );

        Ok(())
    }
    #[test]
    fn test_set_get_execute() -> Result<()> {
        let backend = Backend::new();
        let set_cmd = Set {
            key: "hello".to_string(),
            value: RespFrame::BulkString(BulkString::new(b"world".to_vec())),
        };
        let result = set_cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let get_cmd = Get {
            key: "hello".to_string(),
        };
        let result = get_cmd.execute(&backend);
        assert_eq!(
            result,
            RespFrame::BulkString(BulkString::new(b"world".to_vec()))
        );
        Ok(())
    }
}
