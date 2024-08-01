use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;
use tracing::info;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};

mod hmap;
mod map;

lazy_static! {
    static ref RESP_OK: RespFrame = RespFrame::SimpleString(SimpleString::new("OK".to_string()));
}
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid command format: {0}")]
    InvalidCommandFormat(String),
    #[error("Invalid command arguments: {0}")]
    InvalidCommandArguments(String),
    #[error("Invalid command arguments length: {0}")]
    InvalidCommandArgumentsLength(usize),
    #[error("Command execution error: {0}")]
    ExecutionError(#[from] anyhow::Error),
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[derive(Debug)]
#[enum_dispatch(CommandExecutor)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),

    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Get {
    key: String,
}
#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}
#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}
#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}
#[derive(Debug)]
pub struct HGetAll {
    key: String,
    sort: bool,
}
#[derive(Debug)]
pub struct Unrecognized;

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an array".to_string(),
            )),
        }
    }
}

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        // RespFrame::Error(SimpleError::new("Unrecognized command".to_string()))
        info!("Unrecognized command");
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        let first = v.first();
        match first {
            Some(RespFrame::BulkString(ref cmd)) => {
                let cmd_str = String::from_utf8_lossy(cmd.trim_ascii());
                match cmd_str.to_ascii_lowercase().as_str() {
                    "get" => Ok(Get::try_from(v)?.into()),
                    "set" => Ok(Set::try_from(v)?.into()),
                    "hget" => Ok(HGet::try_from(v)?.into()),
                    "hset" => Ok(HSet::try_from(v)?.into()),
                    "hgetall" => Ok(HGetAll::try_from(v)?.into()),
                    _ => Ok(Unrecognized.into()),
                }
            }
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    // 校验个数
    if value.len() != n_args + names.len() {
        return Err(CommandError::InvalidCommandArguments(format!(
            "{} command must have {} arguments, but got {}",
            names.join(" "),
            n_args + 1,
            value.len()
        )));
    }
    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.as_ref().to_ascii_lowercase() != name.to_ascii_lowercase().as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expect {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must a BulkString as the first argument".to_string(),
                ));
            }
        }
    }

    Ok(())
}

// 抽取参数
fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value
        .iter()
        .skip(start)
        .cloned()
        .collect::<Vec<RespFrame>>())
}
