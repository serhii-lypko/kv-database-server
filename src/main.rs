use std::env;
use std::io;

use serde::{Deserialize, Serialize};

pub mod cmd;
pub mod file_manager;

use cmd::Command;
use file_manager::{FileManager, FileRecordInfo};

static PRIMARY_STORE_FILENAME: &'static str = "store.dat";

/*
    std::error::Error is a trait, not a concrete type. When you return a Result from a
    function and you want to use any type that implements the Error trait as the error type

    Box<dyn Error>: This is a trait object that allows you to return any type that
    implements the Error trait. It's dynamically dispatched.

    "Dynamically dispatched" refers to the mechanism by which the appropriate method
    implementation is selected at runtime, as opposed to compile-time. In Rust,
    this is typically achieved through the use of trait objects, such as Box<dyn Trait>

    In a statically dispatched system, the compiler knows at compile-time which method will be called.
    This is the case with generics in Rust. The compiler generates a specific version of the code for
    each concrete type that is used with the generic function, and method calls are resolved at compile-time.

    fn static_dispatch<T: std::fmt::Display>(item: T) {
        println!("{}", item);
    }

    In a dynamically dispatched system, the exact method that needs to be called is determined at runtime
    based on the actual type of the object. This is slower than static dispatch because it involves looking up
    the method in a vtable (a table of function pointers) at runtime, but it allows for more flexibility.

    fn dynamic_dispatch(item: &dyn std::fmt::Display) {
        println!("{}", item);
    }

    A box is used for a type whose size can't be known at compile time. The dyn std::error::Error is a trait object,
    and trait objects do not have a compile-time known size because they can represent any type that implements the
    Error trait. This is why they need to be boxed when used in contexts that require a fixed size, like function return types.
*/
pub type Error = Box<dyn std::error::Error>;

/*
    Можеш подумати над якимось своїм «форматом» даних для зберігання. Далі вже потім будеш думати про
    raw binary чи ні. Тобі всеодно парсити це все в твої растовські структури. Також
    питання швидкості - технічно тобі не так сильно принципово, бо ти зробиш бд лонг раннінг процесом,
    який спарсить потрібні данні при запуску і буде тримати кеш для швидкого доступу. Так що можливо
    краще подумати над стратегією для кеша.

    Про бд - можеш піти далі і зробити, щось схоже на datomic або xtdb кложуровскі, але примітивно
    просто зробити урахування, що кожен рекорд є імутабільним і зробити прохід по часу, щоб в тебе
    була збережена кожна копія їх змін.
*/

/*
    🟡 General implementation plan:

    1. Append and read KV from the Flat File ✅
        1.1. Do this with base CLI commands: SET and GET (🛠️ in progress..)
    2. Keep tracking of data with KV offset & len saving to separate file
        2.1. Caching index data in memory
    3. Impelment CRUD with Append-Only Strategy (incl delete)


    5. Concurrent connections and multiple read/write
    6. Implement compaction (removing stale records) with the running program
*/

/*
    KV offset & len saving strategy:

    Separate Metadata File: Use a separate file to store the metadata. Each entry in this file
    could be a fixed-size record containing the key, its offset, and length. This file can
    be loaded into memory when the program starts.
*/

/*
    Data updating basic strategy:

    Append-Only Strategy
    New Write: Instead of updating the value in-place, you append the new value at the end of
    the data file. Update Metadata: Update the byte offset and length for that key in both the
    in-memory index and the metadata file to point to the new location. Old Data: The old data
    remains in the file but is no longer referenced. Over time, this could lead to wasted space.
    Compaction: Periodically, you could run a compaction process to remove
    unreferenced data and reclaim disk space.
*/

/*
    About Flat File data storing approach

    Key-Value (KV) stores can use a variety of underlying storage mechanisms,
    and a flat file is one of the simplest options. However, not all KV stores
    use flat files. Some may use more complex data structures like B-trees, LSM-trees,
    or hash indexes to manage data on disk. The choice of storage mechanism often depends
    on the specific requirements of the application, such as read/write performance,
    data integrity, and scalability.

    In a flat file-based KV store, each key-value pair is typically written in a
    straightforward manner, one after the other. This is often done in an append-only
    fashion to simplify the write operation and improve performance. Metadata like
    offsets may be kept in memory or in a separate file to enable quick lookups.

    So, while flat files can be used in KV stores, they are not the only option,
    and more advanced KV stores often use specialized data structures to meet
    specific performance and reliability criteria.
*/

// -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --

fn main() -> Result<(), Error> {
    let file_manager = FileManager::new(PRIMARY_STORE_FILENAME);

    let cli_args: Vec<String> = env::args().collect();

    // TODO: how to handle all that outside from main?
    match Command::from_args(cli_args) {
        Ok(command) => {
            match command {
                Command::GET(key) => {
                    // TODO: read record info from index

                    let mock_info = FileRecordInfo {
                        offset: 128,
                        len: 30,
                    };

                    match file_manager.read_record_bytes(&mock_info) {
                        Ok(bytes) => {
                            let kv = KVPair::read_from_bytes(bytes);

                            dbg!(kv);
                        }
                        Err(err) => {
                            println!("Could not read record");
                            dbg!(err);
                        }
                    };

                    // dbg!(record_bytes);
                }
                Command::SET(key, value) => {
                    let kv_pair = KVPair::new(key, value);

                    match file_manager.append_serialized_record(kv_pair.serialize()) {
                        Ok(res) => {
                            // TODO: set record info to index
                            dbg!(res);
                        }
                        Err(err) => {
                            dbg!(err);
                        }
                    }
                }
            };
        }
        Err(err) => {
            eprintln!("Error reading arguments: {}", err);
        }
    };

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KVPair {
    key: String,
    value: String,
}

impl KVPair {
    pub fn new(key: String, value: String) -> Self {
        KVPair { key, value }
    }

    pub fn serialize(&self) -> String {
        let serialized_data = serde_json::to_string(&self);

        match serialized_data {
            Ok(serialized_data) => serialized_data,
            Err(_) => {
                // TODO: handle more gracefully
                panic!("Could not serialize the data");
            }
        }
    }

    pub fn read_from_bytes(bytes: Vec<u8>) -> Result<KVPair, Error> {
        let record_str = String::from_utf8(bytes)?;
        let kv_pair: Result<KVPair, serde_json::Error> = serde_json::from_str(&record_str);

        kv_pair.map_err(|err| Box::new(err) as Error)
    }
}

// TODO: not sure if it's needed
impl TryFrom<Command> for KVPair {
    type Error = Error;

    fn try_from(command: Command) -> Result<Self, Self::Error> {
        match command {
            Command::SET(key, value) => Ok(KVPair { key, value }),
            cmd => Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Conversion from {:?} command to KVPair is impossible", cmd),
            ))),
        }
    }
}
