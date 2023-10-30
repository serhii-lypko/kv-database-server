pub mod client;
pub mod cmd;
pub mod connection;
pub mod db;
pub mod frame;
pub mod server;

pub type Error = Box<dyn std::error::Error>;

/*
    TODO 1.
    Try TDD
    https://www.tedinski.com/2019/03/11/fast-feedback-from-tests.html

    TODO 2.
    Try debugging with breakpoints
*/

/*
    🟡 General implementation plan:

    At the very basic implementation transport protocol is going to be capable to
    transport only strings?


    Version 0.1.0
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
    - CLI commands integration (clap crate + running loop on the client)
    - Delete command
    - Implement simple compaction
    - Simple tests
    - Db index as singletone? -> research needed

    TODO: structs & implementations ordering and consistency


    Version 0.2.0
    - Concurrent read-write? -> research needed
    - Segment file compaction
    - More advanced tests
    - Error handling with anyhow?


    Version 0.3.0
    - Timeout Mechanism for connection keep alive
    - Gracefull server shutdown
*/
