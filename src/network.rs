use anyhow::Result;
use bytes::BytesMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

use crate::cmd::{Command, CommandExecutor};
use crate::{Backend, RespDecode, RespEncode, RespError, RespFrame};

#[derive(Debug)]
struct RespFrameCodec;

#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}
#[derive(Debug)]
struct RedisResponse {
    frame: RespFrame,
}

pub async fn handle_connection(stream: TcpStream, backend: Backend) -> Result<()> {
    // how to get a frame from the stream
    // call request_handler to handle the request
    // send the response back to the stream
    let mut framed = Framed::new(stream, RespFrameCodec);

    loop {
        let cloned_backend = backend.clone(); // Clone 一个 backend 供子任务使用
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let request = RedisRequest {
                    frame,
                    backend: cloned_backend,
                };
                let response = request_handler(request).await?;
                info!("Sending response: {:?}", response);
                // 向 stream 发送响应
                framed.send(response.frame).await?
            }
            Some(Err(err)) => return Err(err),
            None => return Ok(()),
        }
    }
}

// 处理一个请求并返回响应
async fn request_handler(request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (request.frame, request.backend);
    let cmd = Command::try_from(frame)?;
    info!("Executing command: {:?}", cmd);
    let frame = cmd.execute(&backend);
    Ok(RedisResponse { frame })
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        item: RespFrame,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded); // 转化成 bytes 并贝到 dst
        Ok(())
    }
}
impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> std::result::Result<Option<Self::Item>, Self::Error> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
