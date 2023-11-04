pub mod client;
pub mod cmd;
pub mod connection;
pub mod db;
pub mod frame;
pub mod server;

pub type Error = Box<dyn std::error::Error>;

pub const DEFAULT_PORT: u16 = 6379;

/*
    TODO: logger
*/

/*
    🟡 General implementation plan:

    At the very basic implementation transport protocol is going to be capable to
    transport only strings?


    Version 1.0.0
    - Append and read KV from the Flat File ✅
        + Do this with base CLI commands: SET and GET ✅
    - Basic TCP server ✅
        + Basic backbone of the client-server ✅
        + Passing via TCP stream simple string and digit frames as KV value ✅
    - Apply commands to database ✅
        + Append only SET ✅
    - In-memory indexing ✅
        + Initiate hash-map in-memory index ✅
        + Apply GET on database via index ✅
    - Write response to TCP stream; handle not found for GET ✅
    - CLI commands integration with clap crate ✅
    - Delete command ✅
    - Implement simple compaction (without segments) including deleted records cleanup ✅
    - Tests for Db module
    - README


    Version 1.1.1
    - Db index as singletone? -> research needed
    - Structs & implementations ordering and consistency
    - Figure out debug and tracing!!


    Version 1.2.0
    - Concurrent read-write? -> research needed
    - Segment file compaction
    - Scan?
    - More advanced tests
    - Error handling with anyhow?


    Version 1.3.0
    - Timeout Mechanism for connection keep alive
    - Gracefull server shutdown
*/
