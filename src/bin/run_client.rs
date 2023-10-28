use kv_db::{client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    use client::Client;

    let mut client = Client::connect("127.0.0.1:6379").await?;

    // let ping_res = client.ping().await?;
    // println!("Ping response: {}", ping_res);

    // client.get("hello").await?;

    let set_res = client.set("hello", "world".into()).await?;

    Ok(())
}
