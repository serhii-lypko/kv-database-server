use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

// TODO: have index as a singleton?
pub struct DbHolder {
    db: Db,
}

#[derive(Clone)]
pub struct Db {
    index: Arc<Mutex<Index>>,
    filename: &'static str,
}

#[derive(Debug)]
struct Index {
    records: HashMap<String, IndexRecord>,
}

#[derive(Debug)]
pub struct IndexRecord {
    offset: u64,
    len: u64,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileRecord {
    key: String,
    value: Option<String>, // TODO: how to represent other formats? Bytes?
    timestamp: u64,
    deleted: bool,
}

impl DbHolder {
    pub fn new() -> DbHolder {
        DbHolder { db: Db::new() }
    }

    pub fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    pub fn new() -> Db {
        let filename = "store.dat";

        let index = Db::rehydrate_index_from_disk(filename)
            .unwrap_or(None)
            .unwrap_or_else(|| HashMap::new());

        Db {
            index: Arc::new(Mutex::new(Index { records: index })),
            filename,
        }
    }

    fn rehydrate_index_from_disk(
        filename: &str,
    ) -> Result<Option<HashMap<String, IndexRecord>>, crate::Error> {
        let file = File::open(&filename)?;
        let reader = BufReader::new(file);

        let mut hydrated_index = HashMap::new();

        let mut offset = 0;

        for line in reader.lines() {
            let line = line?;

            // +1 for escaping byte
            let len = line.len() as u64 + 1;

            let record: FileRecord = serde_json::from_str(&line)?;

            if record.deleted {
                hydrated_index.remove(&record.key);
            } else {
                let index_record = IndexRecord {
                    offset,
                    len,
                    timestamp: record.timestamp,
                };

                hydrated_index.insert(record.key, index_record);
            }

            offset += len;
        }

        if hydrated_index.is_empty() {
            return Ok(None);
        }

        Ok(Some(hydrated_index))
    }

    pub fn get(&self, key: &str) -> Result<Option<FileRecord>, crate::Error> {
        let index_state = self.index.lock().unwrap();

        // TODO: how to handle record not found? what kind of error to return?
        if let Some(record_info) = index_state.records.get(key) {
            let mut file = File::open(&self.filename)?;

            file.seek(SeekFrom::Start(record_info.offset.clone()))?;

            let mut buffer = vec![0; record_info.len as usize];
            file.read_exact(&mut buffer)?;

            let string = String::from_utf8(buffer);

            let record: FileRecord = serde_json::from_str(&string.unwrap().as_str()).unwrap();

            if record.deleted {
                return Ok(None);
            }

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, key: String, value: Bytes) -> Result<(), crate::Error> {
        let mut file = OpenOptions::new().append(true).open(&self.filename)?;

        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let timestamp: u64 = since_the_epoch.as_secs();

        let serialized_rec_with_escapring =
            FileRecord::prepare_for_writing(key.clone(), value, timestamp)?;

        let offset = file.metadata()?.len();
        let len = serialized_rec_with_escapring.len() as u64;

        let mut index_state = self.index.lock().unwrap();

        index_state.records.insert(
            key,
            IndexRecord {
                offset,
                len,
                timestamp,
            },
        );

        file.write_all(serialized_rec_with_escapring.as_bytes())?;

        drop(index_state);

        Ok(())
    }

    /*
        Deleting algorithm

        1. Append tombstone
        2. Delete key from index
        3. Hydration: delete key if found as tombstone
        4. ⭐️ Compaction: remove all records before tombstone
    */
    pub fn delete(&self, key: String) -> Result<(), crate::Error> {
        println!("Gonn delete record on DB level");

        Ok(())
    }
}

impl FileRecord {
    pub fn get_val_bytes(&self) -> Option<Bytes> {
        self.value.clone().map(|val| Bytes::from(val.into_bytes()))
    }

    pub fn prepare_for_writing(
        key: String,
        value: Bytes,
        timestamp: u64,
    ) -> Result<String, crate::Error> {
        let value_string = String::from_utf8(value.to_vec())?;

        let record = FileRecord {
            key,
            value: Some(value_string),
            timestamp,
            deleted: false,
        };

        let serialized = serde_json::to_string(&record)?;
        let serialized_with_newline = format!("{}\n", serialized);

        Ok(serialized_with_newline)
    }
}
