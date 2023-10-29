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


    Version 0.1.0
    1. Append and read KV from the Flat File ✅
        - Do this with base CLI commands: SET and GET ✅
    2. Basic TCP server
        - Basic backbone of the client-server ✅
        - Passing via TCP stream simple string and digit frames as KV value ✅
    3. Apply commands to database ✅
        - Append only SET ✅
    4. In-memory indexing ✅
        - Initiate hash-map in-memory index ✅
        - Apply GET on database via index ✅
    5. CLI commands integration (running loop on the client)
    6. CRUD
        - Update
        - Delete
    7. Implement simple compaction
    8. Simple tests

    TODO: structs & implementations ordering and conistency


    Version 0.2.0
    - More advanced tests
    - Concurrent read-write
    - Segment file compaction
    - Error handling with anyhow?
    - Refactor CLI with clap crate, rename & refactor transport_cmd module
    - Timeout Mechanism for connection keep alive
    - Gracefull server shutdown
*/
