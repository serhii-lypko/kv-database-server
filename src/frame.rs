use bytes::{Buf, Bytes};
use std::io::Cursor;

use std::fmt;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

#[derive(Debug, Clone)]
pub enum Frame {
    Simple(String),    // +
    Integer(u64),      // :
    Bulk(Bytes),       // $
    Array(Vec<Frame>), // *
}

impl Frame {
    pub fn array() -> Frame {
        Frame::Array(vec![])
    }

    pub fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Bulk(bytes));
            }
            _ => panic!("Not an array frame"),
        }
    }

    pub fn push_string(&mut self, string: String) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Simple(string));
            }
            _ => panic!("Not an array frame"),
        }
    }

    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_descriptor(src)? {
            // simple
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            // bulk
            b'$' => {
                let len = get_decimal(src)? as usize;

                // skip that number of bytes + 2 (\r\n).
                skip(src, len + 2)
            }
            // array
            b'*' => {
                let len = get_decimal(src)?;

                for _ in 0..len {
                    Frame::check(src)?;
                }

                Ok(())
            }
            _ => todo!(),
        }
    }

    // PING: "*  __  1\r\n  __  $4\r\n  __  ping\r\n"
    // SET: "*  __  3\r\n  __  +set\r\n  __  +hello\r\n  __  $5\r\  __  nworld\r\n"

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_descriptor(src)? {
            b'+' => {
                let bytes_vec = get_line(src)?.to_vec();
                let string = String::from_utf8(bytes_vec)?;

                Ok(Frame::Simple(string))
            }
            b'$' => {
                let len = get_decimal(src)? as usize;
                let n = len + 2;

                if src.remaining() < n {
                    return Err(Error::Incomplete);
                }

                let data = Bytes::copy_from_slice(&src.chunk()[..len]);

                // skip that number of bytes + 2 (\r\n).
                skip(src, n)?;

                Ok(Frame::Bulk(data))
            }
            b'*' => {
                let len = get_decimal(src)?;
                let mut array: Vec<Frame> = Vec::with_capacity(len as usize);

                for _ in 0..len {
                    array.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(array))
            }
            _ => todo!(),
        }
    }
}

fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(src)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

/// A "line" refers to a sequence of bytes that is terminated by a carriage return
/// (\r, ASCII 13) followed by a line feed (\n, ASCII 10). This is a common way to denote
/// the end of a line in many text-based protocols and file formats, especially on Unix-like systems.
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);

            return Ok(&src.get_ref()[start..i]);
        }
    }

    return Err(Error::Incomplete);
}

fn get_descriptor(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Other(crate::Error),
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}

impl std::error::Error for Error {}
