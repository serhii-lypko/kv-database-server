use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::cmd::{Delete, Get, Ping, Set};
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
        let frame = Ping::new().into_frame();
        self.connection.write_frame(&frame).await?;

        if let Frame::Simple(string) = self.read_response().await? {
            return Ok(string);
        }

        Err("Internal error".into())
    }

    // NOTE: should return bulk as bytes instead of to-string converting?
    pub async fn get(&mut self, key: &str) -> Result<String, crate::Error> {
        let frame = Get::new(key).into_frame();
        self.connection.write_frame(&frame).await?;

        match self.read_response().await? {
            Frame::Bulk(bytes) => {
                let string = String::from_utf8(bytes.to_vec())?;
                return Ok(string);
            }
            Frame::Simple(string) => {
                return Ok(string);
            }
            Frame::Error(error_kind) => {
                return Ok(format!("Error: {}", error_kind));
            }
            _ => Err("Internal error".into()),
        }
    }

    pub async fn set(&mut self, key: &str, value: Bytes) -> Result<String, crate::Error> {
        let frame = Set::new(key, value).into_frame();
        self.connection.write_frame(&frame).await?;

        if let Frame::Simple(string) = self.read_response().await? {
            return Ok(string);
        }

        Err("Internal error".into())
    }

    pub async fn delete(&mut self, key: &str) -> Result<String, crate::Error> {
        let frame = Delete::new(key).into_frame();
        self.connection.write_frame(&frame).await?;

        match self.read_response().await? {
            Frame::Simple(string) => Ok(string),
            Frame::Error(error_kind) => Ok(format!("Error: {}", error_kind)),
            _ => Err("Internal error".into()),
        }
    }

    async fn read_response(&mut self) -> Result<Frame, crate::Error> {
        let response = self.connection.read_frame().await?;

        match response {
            Some(frame) => Ok(frame),
            _ => todo!(),
        }
    }
}
