use std::str;

use bytes::Bytes;
use clap::{Parser, Subcommand};

use client::Client;
use kv_db::{client, Error, DEFAULT_PORT};

#[derive(Parser, Debug)]
#[clap(name = "kv-db-cli")]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Command {
    Ping {},
    Get {
        key: String,
    },
    Set {
        key: String,
        #[clap(parse(from_str = bytes_from_str))]
        value: Bytes,
    },
}

fn bytes_from_str(src: &str) -> Bytes {
    Bytes::from(src.to_string())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let mut client = Client::connect(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;

    match Cli::parse().command {
        Command::Ping {} => {
            let ping_res = client.ping().await?;
            println!("{}", ping_res);
        }
        Command::Get { key } => {
            let get_res: String = client.get(key.as_str()).await?;
            println!("GET {}: {}", key, get_res);
        }
        Command::Set { key, value } => {
            let set_res = client.set(key.as_str(), value).await?;
            println!("SET {}", set_res);
        }
    }

    Ok(())
}
