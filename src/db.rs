use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileRecord {
    key: String,
    value: Option<String>, // TODO: how to represent other formats? Bytes?
    timestamp: u64,
    is_tombstone: bool,
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

            if record.is_tombstone {
                hydrated_index.remove(&record.key);
            } else {
                let index_record = IndexRecord { offset, len };

                hydrated_index.insert(record.key, index_record);
            }

            offset += len;
        }

        if hydrated_index.is_empty() {
            return Ok(None);
        }

        Ok(Some(hydrated_index))
    }

    fn insert(&self, record: FileRecord) -> Result<(), crate::Error> {
        let mut file = OpenOptions::new().append(true).open(&self.filename)?;

        let serialized_rec = record.serialize_with_escaping()?;

        let offset = file.metadata()?.len();
        let len = serialized_rec.len() as u64;

        let mut index_state_lock = self.index.lock().unwrap();

        index_state_lock
            .records
            .insert(record.key, IndexRecord { offset, len });

        file.write_all(serialized_rec.as_bytes())?;

        drop(index_state_lock);

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<FileRecord>, crate::Error> {
        let index_state = self.index.lock().unwrap();

        if let Some(record_info) = index_state.records.get(key) {
            let mut file = File::open(&self.filename)?;

            file.seek(SeekFrom::Start(record_info.offset.clone()))?;

            let mut buffer = vec![0; record_info.len as usize];
            file.read_exact(&mut buffer)?;

            let string = String::from_utf8(buffer);

            let record: FileRecord = serde_json::from_str(&string.unwrap().as_str()).unwrap();

            if record.is_tombstone {
                return Ok(None);
            }

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, key: String, value: Bytes) -> Result<(), crate::Error> {
        let value = String::from_utf8(value.to_vec()).ok();

        let record = FileRecord::new(key.clone(), value, false);

        self.insert(record)?;

        Ok(())
    }

    // TODO: how to prevent multiple delete records append?
    pub fn delete(&self, key: String) -> Result<Option<()>, crate::Error> {
        let index_state_lock = self.index.lock().unwrap();
        let index_record = index_state_lock.records.get(key.as_str());

        if index_record.is_none() {
            return Ok(None);
        }

        // TODO: how does it work?
        // Release the mutex before notifying the background task. This helps
        // reduce contention by avoiding the background task waking up only to
        // be unable to acquire the mutex due to this function still holding it.
        drop(index_state_lock);

        let record_to_delete = FileRecord::new(key, None, true);

        self.insert(record_to_delete)?;

        Ok(Some(()))
    }
}

impl FileRecord {
    pub fn new(key: String, value: Option<String>, is_tombstone: bool) -> FileRecord {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        let timestamp: u64 = since_the_epoch.as_secs();

        FileRecord {
            key,
            value,
            timestamp,
            is_tombstone,
        }
    }

    pub fn get_val_bytes(&self) -> Option<Bytes> {
        self.value.clone().map(|val| Bytes::from(val.into_bytes()))
    }

    pub fn serialize_with_escaping(&self) -> Result<String, crate::Error> {
        let serialized = serde_json::to_string(&self)?;
        let serialized_with_newline = format!("{}\n", serialized);

        Ok(serialized_with_newline)
    }
}
