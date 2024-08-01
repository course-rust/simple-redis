use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{error, info};

use simple_redis::{network, Backend};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing library
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis-Server is listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    let backend = Backend::new();
    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        let cloned_backend = backend.clone(); // 克隆一个 backend 供子任务使用
        tokio::spawn(async move {
            match network::handle_connection(stream, cloned_backend).await {
                Ok(_) => info!("Connection from {} exited", raddr),
                Err(e) => error!("Error handling connection for {}: {:?}", raddr, e),
            }
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}
