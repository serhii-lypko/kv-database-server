use std::env;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::stdout;
use std::io::{self, Seek, SeekFrom, Write};
use std::str::FromStr;
use std::vec;

pub mod cmd;

use cmd::Command;

use serde::{Deserialize, Serialize};

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
    let cli_args: Vec<String> = env::args().collect();

    match Command::from_args(cli_args) {
        Ok(command) => {
            dbg!(command);
        }
        Err(err) => {
            eprintln!("Error reading arguments: {}", err);
        }
    };

    // let kv_bytes = KVPair::read_bytes(32, 32);

    // let string = String::from_utf8(kv_bytes.unwrap()).unwrap();
    // let deserialized: KVPair = serde_json::from_str(&string).unwrap();

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct KVPair {
    key: String,
    value: String,
}

impl KVPair {
    pub fn append_to_file() -> std::io::Result<()> {
        let pair = KVPair {
            key: "name".to_string(),
            value: "Tom".to_string(),
        };

        let serialized_data = serde_json::to_string(&pair)?;

        let mut file = OpenOptions::new().append(true).open("store.dat")?;

        let record_offset = file.metadata()?.len();
        let record_len = serialized_data.len();

        println!("record_offset {:?}", record_offset);
        println!("record_len {:?}", record_len);

        file.write_all(serialized_data.as_bytes())?;

        Ok(())
    }

    pub fn read_bytes(offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        let mut file = File::open("store.dat")?;

        /*
         * Set cursor for the KV's offset
         * This is done so that the subsequent read operation will begin reading data from that exact position.
         */
        file.seek(SeekFrom::Start(offset))?;

        // Create a buffer and populate buffer with data slice
        let mut buffer = vec![0; len];
        file.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}
