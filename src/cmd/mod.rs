use bytes::Bytes;

mod parse;

use crate::connection::Connection;
use crate::db::Db;
use crate::frame::{Frame, FrameErrorKind};
use parse::{Parse, ParseError};

#[derive(Debug)]
pub enum Command {
    Ping(Ping),
    Get(Get),
    Set(Set),
    Delete(Delete),
}

#[derive(Debug, Default)]
pub struct Ping;

#[derive(Debug)]
pub struct Get {
    pub key: String,
}

#[derive(Debug)]
pub struct Set {
    pub key: String,
    pub value: Bytes,
}

#[derive(Debug)]
pub struct Delete {
    pub key: String,
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command, crate::Error> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        let command = match command_name.as_str() {
            "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            "delete" => Command::Delete(Delete::parse_frames(&mut parse)?),
            _ => todo!(),
        };

        parse.finish()?;

        Ok(command)
    }

    pub(crate) async fn apply(self, conn: &mut Connection, db: &Db) -> Result<(), crate::Error> {
        use Command::*;

        match self {
            Ping(cmd) => cmd.apply(conn).await,
            Get(cmd) => cmd.apply(conn, db).await,
            Set(cmd) => cmd.apply(conn, db).await,
            Delete(cmd) => cmd.apply(conn, db).await,
        }
    }
}

impl Ping {
    pub fn new() -> Ping {
        Ping {}
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.push_string("ping".to_string());

        frame
    }

    pub fn parse_frames(parse: &mut Parse) -> Result<Ping, crate::Error> {
        match parse.next_bytes() {
            Ok(_) => Ok(Ping::new()),
            Err(ParseError::EndOfStream) => Ok(Ping::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn apply(self, conn: &mut Connection) -> Result<(), crate::Error> {
        let response = Frame::Simple("PONG".to_string());

        conn.write_frame(&response).await?;

        Ok(())
    }
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
        let key = parse.next_string()?;

        Ok(Get { key })
    }

    pub async fn apply(self, conn: &mut Connection, db: &Db) -> Result<(), crate::Error> {
        let resp_frame = match db.get(self.key.as_str())? {
            Some(record) => {
                let val_bytes = record.get_val_bytes();
                let err_resp = Frame::Error(FrameErrorKind::InternalError);

                val_bytes.map_or(err_resp, Frame::Bulk)
            }
            None => Frame::Error(FrameErrorKind::NotFound),
        };

        conn.write_frame(&resp_frame).await?;

        Ok(())
    }
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

    pub async fn apply(self, conn: &mut Connection, db: &Db) -> Result<(), crate::Error> {
        db.set(self.key, self.value)?;

        let response = Frame::Simple("OK".to_string());
        conn.write_frame(&response).await?;

        Ok(())
    }
}

impl Delete {
    pub fn new(key: impl ToString) -> Delete {
        Delete {
            key: key.to_string(),
        }
    }

    pub fn into_frame(self) -> Frame {
        let mut frame = Frame::array();

        frame.push_string("delete".to_string());
        frame.push_string(self.key);

        frame
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> Result<Delete, crate::Error> {
        let key = parse.next_string()?;

        Ok(Delete { key })
    }

    pub async fn apply(self, conn: &mut Connection, db: &Db) -> Result<(), crate::Error> {
        let resp_frame = match db.delete(self.key)? {
            Some(_) => Frame::Simple("OK".to_string()),
            None => Frame::Error(FrameErrorKind::NotFound),
        };

        conn.write_frame(&resp_frame).await?;

        Ok(())
    }
}
