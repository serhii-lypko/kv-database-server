use bytes::Bytes;

mod parse;

use crate::connection::Connection;
use crate::frame::Frame;
use parse::{Parse, ParseError};

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    Ping(Ping),
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command, crate::Error> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match command_name.as_str() {
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            _ => todo!(),
        };

        parse.finish()?;

        Ok(command)
    }

    pub(crate) async fn apply(self, conn: &mut Connection) -> Result<(), crate::Error> {
        use Command::*;

        match self {
            // Get(cmd) => cmd.apply(conn).await,
            Set(cmd) => cmd.apply(conn).await,
            Ping(cmd) => cmd.apply(conn).await,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Ping {
    pub msg: Option<Bytes>,
}

impl Ping {
    pub fn new(msg: Option<Bytes>) -> Ping {
        Ping { msg }
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.push_string("ping".to_string());

        frame
    }

    pub fn parse_frames(parse: &mut Parse) -> Result<Ping, crate::Error> {
        match parse.next_bytes() {
            Ok(msg) => Ok(Ping::new(Some(msg))),
            Err(ParseError::EndOfStream) => Ok(Ping::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn apply(self, conn: &mut Connection) -> Result<(), crate::Error> {
        let response = match self.msg {
            None => Frame::Simple("PONG".to_string()),
            Some(msg) => Frame::Bulk(msg),
        };

        conn.write_frame(&response).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Get {
    pub key: String,
}

impl Get {
    pub fn new(key: impl ToString) -> Get {
        Get {
            key: key.to_string(),
        }
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.push_string("get".to_string());
        frame.push_string(self.key);

        frame
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Get, crate::Error> {
        dbg!(&parse);
        let key = parse.next_string()?;

        Ok(Get { key })
    }
}

#[derive(Debug)]
pub struct Set {
    pub key: String,
    pub value: Bytes,
}

impl Set {
    pub fn new(key: impl ToString, value: Bytes) -> Set {
        Set {
            key: key.to_string(),
            value,
        }
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.push_string("set".to_string());
        frame.push_string(self.key);

        frame.push_bulk(Bytes::from(self.value));

        frame
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Set, crate::Error> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;

        Ok(Set { key, value })
    }

    pub async fn apply(self, conn: &mut Connection) -> Result<(), crate::Error> {
        println!("About to apply SET cmd");
        dbg!(self);

        Ok(())
    }
}
