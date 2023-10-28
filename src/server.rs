use std::future::Future;

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};

use crate::cmd::Command;
use crate::connection::Connection;

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
}

#[derive(Debug)]
struct Handler {
    connection: Connection,
}

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let mut server = Listener { listener };

    // select gives running task an opportunity to finish their execution
    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                println!("failed to accept");
                dbg!(err);
            }
        }
        _ = shutdown => {
            // The shutdown signal has been received.
            println!("shutting down");
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
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    dbg!(err);
                }
            });
        }
    }

    /*
        TODO: separate method makes sence with backing off and retrying
    */
    async fn accept(&mut self) -> Result<TcpStream, crate::Error> {
        match self.listener.accept().await {
            Ok((tcp_stream, _)) => return Ok(tcp_stream),
            //
            // TODO: how does this conversion work?
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

            cmd.apply(&mut self.connection).await?;
        }
    }
}
