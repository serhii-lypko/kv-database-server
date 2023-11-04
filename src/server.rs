use std::future::Future;

use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;

use crate::cmd::Command;
use crate::connection::Connection;
use crate::db::{Db, DbHolder};

struct Listener {
    listener: TcpListener,
    db_holder: DbHolder,
}

struct Handler {
    connection: Connection,
    db: Db,
}

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let mut server = Listener {
        listener,
        db_holder: DbHolder::new(),
    };

    let compaction_shutdown_token = CancellationToken::new();

    // NOTE: not sure how read/write is gonna work if compaction will take a while
    // normally there should be log segmenting, with communication on separate files
    // for read/write and compaction repsectively
    let _db_compaction_task = {
        let db = server.db_holder.db.clone();
        let shutdown_token = compaction_shutdown_token.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(20));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(err) = db.run_compaction() {
                            eprintln!("compaction failed");
                            dbg!(err);
                        }
                    }
                    _ = shutdown_token.cancelled() => {
                        println!("Cleanup");
                        break;
                    }
                }
            }
        });
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                eprintln!("failed to accept");
                dbg!(err);
            }
        }
        _ = shutdown => {
            println!("shutting down");
            compaction_shutdown_token.cancel();
        }
    }
}

impl Listener {
    /// Loop will not proceed until a connection is accepted. During this time,
    /// the task is effectively "paused," releasing the CPU to handle other tasks.
    ///
    /// Once a connection is accepted, the loop proceeds to initialize a Handler for
    /// the connection and spawns a new asynchronous task to handle it.
    ///
    /// After spawning the task, the loop immediately goes back to awaiting another
    /// connection. This happens regardless of whether the previously spawned tasks
    /// have completed their execution.
    async fn run(&mut self) -> Result<(), crate::Error> {
        loop {
            let socket = self.accept().await?;

            // println!("-- << -- Create new handler for connection -- >> --");

            let mut handler = Handler {
                connection: Connection::new(socket),
                db: self.db_holder.db(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    dbg!(err);
                }
            });
        }
    }

    // TODO: backing off and retrying
    async fn accept(&mut self) -> Result<TcpStream, crate::Error> {
        match self.listener.accept().await {
            Ok((tcp_stream, _)) => Ok(tcp_stream),
            Err(err) => return Err(err.into()),
        }
    }
}

impl Handler {
    async fn run(&mut self) -> Result<(), crate::Error> {
        // TODO: normally should expect termination signal
        loop {
            let maybe_frame = self.connection.read_frame().await?;

            let frame = match maybe_frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            let cmd = Command::from_frame(frame)?;

            cmd.apply(&mut self.connection, &self.db).await?;
        }
    }
}
