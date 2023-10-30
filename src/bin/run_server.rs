use tokio::net::TcpListener;
use tokio::signal;

use kv_db::DEFAULT_PORT;
use kv_db::{server, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}
