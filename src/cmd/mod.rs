use lazy_static::lazy_static;
use thiserror::Error;

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

pub trait CommandExecutor {
    fn execute(&self, backend: &Backend) -> RespFrame;
}

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Get {
    key: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Set {
    key: String,
    value: RespFrame,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct HGet {
    key: String,
    field: String,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct HSet {
    key: String,
    field: String,
    value: RespFrame,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct HGetAll {
    key: String,
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(_frame: RespArray) -> Result<Self, Self::Error> {
        todo!()
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
