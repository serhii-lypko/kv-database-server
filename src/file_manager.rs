use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{Seek, SeekFrom, Write};
use std::vec;

pub struct FileManager {
    pub filename: &'static str,
}

#[derive(Debug)]
pub struct FileRecordInfo {
    pub offset: u64,
    pub len: usize,
}

impl FileManager {
    pub fn new(filename: &'static str) -> Self {
        Self { filename }
    }

    // TODO: is naming ok?
    pub fn append_serialized_record(&self, record: String) -> std::io::Result<FileRecordInfo> {
        // TODO: create file if not found
        let mut file = OpenOptions::new().append(true).open(&self.filename)?;

        let record_offset = file.metadata()?.len();
        let record_len = record.len();

        let append_info = FileRecordInfo {
            offset: record_offset,
            len: record_len,
        };

        file.write_all(record.as_bytes())?;

        Ok(append_info)
    }

    pub fn read_record_bytes(&self, record_info: &FileRecordInfo) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(&self.filename)?;

        /*
         * Set cursor for the KV's offset
         * This is done so that the subsequent read operation will begin reading data from that exact position.
         */
        file.seek(SeekFrom::Start(record_info.offset))?;

        // Create a buffer with zeros size len and populate it with data slice
        let mut buffer = vec![0; record_info.len];
        file.read_exact(&mut buffer)?;

        Ok(buffer)
    }
}
