use std::env;
use std::io;

use tokio::net::TcpListener;
use tokio::signal;

use serde::{Deserialize, Serialize};

use kv_db::{server, Error};

const PRIMARY_STORE_FILENAME: &'static str = "store.dat";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}

// TODO: reading CLI arguments could be implemented using clap

// fn main_2() -> Result<(), Error> {
//     let file_manager = FileManager::new(PRIMARY_STORE_FILENAME);

//     let cli_args: Vec<String> = env::args().collect();

//     // TODO: how to handle all that outside from main?
//     match Command::from_args(cli_args) {
//         Ok(command) => {
//             match command {
//                 Command::GET(key) => {
//                     // TODO: read record info from index

//                     let mock_info = FileRecordInfo {
//                         offset: 128,
//                         len: 30,
//                     };

//                     match file_manager.read_record_bytes(&mock_info) {
//                         Ok(bytes) => {
//                             let kv = KVPair::read_from_bytes(bytes);

//                             dbg!(kv);
//                         }
//                         Err(err) => {
//                             println!("Could not read record");
//                             dbg!(err);
//                         }
//                     };

//                     // dbg!(record_bytes);
//                 }
//                 Command::SET(key, value) => {
//                     let kv_pair = KVPair::new(key, value);

//                     match file_manager.append_serialized_record(kv_pair.serialize()) {
//                         Ok(res) => {
//                             // TODO: set record info to index
//                             dbg!(res);
//                         }
//                         Err(err) => {
//                             dbg!(err);
//                         }
//                     }
//                 }
//             };
//         }
//         Err(err) => {
//             eprintln!("Error reading arguments: {}", err);
//         }
//     };

//     Ok(())
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct KVPair {
//     key: String,
//     value: String,
// }

// impl KVPair {
//     pub fn new(key: String, value: String) -> Self {
//         KVPair { key, value }
//     }

//     pub fn serialize(&self) -> String {
//         let serialized_data = serde_json::to_string(&self);

//         match serialized_data {
//             Ok(serialized_data) => serialized_data,
//             Err(_) => {
//                 // TODO: handle more gracefully
//                 panic!("Could not serialize the data");
//             }
//         }
//     }

//     pub fn read_from_bytes(bytes: Vec<u8>) -> Result<KVPair, Error> {
//         let record_str = String::from_utf8(bytes)?;
//         let kv_pair: Result<KVPair, serde_json::Error> = serde_json::from_str(&record_str);

//         kv_pair.map_err(|err| Box::new(err) as Error)
//     }
// }

// TODO: not sure if it's needed
// impl TryFrom<Command> for KVPair {
//     type Error = Error;

//     fn try_from(command: Command) -> Result<Self, Self::Error> {
//         match command {
//             Command::SET(key, value) => Ok(KVPair { key, value }),
//             cmd => Err(Box::new(io::Error::new(
//                 io::ErrorKind::InvalidData,
//                 format!("Conversion from {:?} command to KVPair is impossible", cmd),
//             ))),
//         }
//     }
// }
