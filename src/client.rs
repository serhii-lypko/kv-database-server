use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::cmd::{Get, Ping, Set};
use crate::connection::Connection;
use crate::frame::Frame;

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client, crate::Error> {
        let socket = TcpStream::connect(addr).await?;
        let connection = Connection::new(socket);

        Ok(Client { connection })
    }

    pub async fn ping(&mut self) -> Result<String, crate::Error> {
        let frame = Ping::new(None).into_frame();
        self.connection.write_frame(&frame).await?;

        if let Frame::Simple(string) = self.read_response().await? {
            return Ok(string);
        }

        Err("Internal error".into())
    }

    pub async fn get(&mut self, key: &str) -> Result<(), crate::Error> {
        let frame = Get::new(key).into_frame();
        self.connection.write_frame(&frame).await?;

        // TODO: read response

        Ok(())
    }

    pub async fn set(&mut self, key: &str, value: Bytes) -> Result<(), crate::Error> {
        let frame = Set::new(key, value).into_frame();
        self.connection.write_frame(&frame).await?;

        // TODO: read response

        Ok(())
    }

    async fn read_response(&mut self) -> Result<Frame, crate::Error> {
        let response = self.connection.read_frame().await?;

        match response {
            Some(frame) => Ok(frame),
            _ => todo!(),
        }
    }
}
