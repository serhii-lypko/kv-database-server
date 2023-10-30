use kv_db::{client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    use client::Client;

    let mut client = Client::connect("127.0.0.1:6379").await?;

    // let ping_res = client.ping().await?;
    // println!("Ping response: {}", ping_res);

    // TODO: how to pass digits?
    // let set_res = client.set("cats_number", "many".into()).await?;
    // dbg!(set_res);

    let get_res = client.get("cats_number").await?;
    dbg!(get_res);

    Ok(())
}
