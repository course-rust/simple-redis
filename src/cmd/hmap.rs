use crate::cmd::{extract_args, validate_command, CommandExecutor, HGetAll, HSet, RESP_OK};
use crate::{
    cmd::{CommandError, HGet},
    Backend, BulkString, RespArray, RespFrame, RespNull,
};

//===================  实现 CommandExecutor trait for Command
impl CommandExecutor for HGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or(RespFrame::Null(RespNull))
    }
}
impl CommandExecutor for HGetAll {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hgetall(&self.key);
        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    data.push((key, v.value().clone()));
                }
                if self.sort {
                    data.sort_by(|a, b| a.0.cmp(&b.0));
                }
                let ret = data
                    .into_iter()
                    .flat_map(|(k, v)| vec![BulkString::new(k.as_bytes()).into(), v])
                    .collect::<Vec<RespFrame>>();

                RespArray::new(ret).into()
            }
            None => RespFrame::Null(RespNull),
        }
    }
}
impl CommandExecutor for HSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.hset(self.key, self.field, self.value);
        RESP_OK.clone()
    }
}

//===================  实现 TryFrom trait for Command
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
                sort: false,
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
    #[test]
    fn test_hset_hgetall_commands() -> Result<()> {
        let backend = Backend::new();
        let set_cmd = HSet {
            key: "mykey".to_string(),
            field: "myfield".to_string(),
            value: RespFrame::BulkString(BulkString::new(b"myvalue".to_vec())),
        };
        let set_result = set_cmd.execute(&backend);
        assert_eq!(set_result, RESP_OK.clone());

        let get_cmd = HGet {
            key: "mykey".to_string(),
            field: "myfield".to_string(),
        };
        let get_result = get_cmd.execute(&backend);
        assert_eq!(
            get_result,
            RespFrame::BulkString(BulkString::new(b"myvalue".to_vec()))
        );

        let set_cmd = HSet {
            key: "mykey".to_string(),
            field: "hello".to_string(),
            value: RespFrame::BulkString(BulkString::new(b"world".to_vec())),
        };
        set_cmd.execute(&backend);

        let getall_cmd = HGetAll {
            key: "mykey".to_string(),
            sort: true,
        };
        let getall_result = getall_cmd.execute(&backend);
        let expected_result = RespArray::new(vec![
            BulkString::new(b"hello".to_vec()).into(),
            BulkString::new(b"world".to_vec()).into(),
            BulkString::new(b"myfield".to_vec()).into(),
            BulkString::new(b"myvalue".to_vec()).into(),
        ]);
        assert_eq!(getall_result, expected_result.into());

        Ok(())
    }
}
