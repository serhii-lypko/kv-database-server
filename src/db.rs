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
    pub db: Db,
}

#[derive(Clone)]
pub struct Db {
    index: Arc<Mutex<Index>>,
    storage_filename: String,
}

#[derive(Debug)]
struct Index {
    records: HashMap<String, ValueMetadata>,
}

#[derive(Debug, Clone)]
pub struct ValueMetadata {
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
        let filename = "store.dat".to_string();

        DbHolder {
            db: Db::new(filename),
        }
    }

    pub fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    pub fn new(storage_filename: impl Into<String>) -> Db {
        let storage_filename = storage_filename.into();

        let index = Db::rehydrate_index_from_disk(storage_filename.as_str())
            .unwrap_or(None)
            .unwrap_or_else(|| HashMap::new());

        Db {
            index: Arc::new(Mutex::new(Index { records: index })),
            storage_filename,
        }
    }

    pub fn run_compaction(&self) -> Result<(), crate::Error> {
        // The compaction algorithm is straightforward:
        // take in-memory index which represents the most recent state, erase storage file content
        // and then write that index data into file.
        // No doubt it's a crude and fragile solution which has many drawbacks (like complete data loss in case of
        // compaction process interuption), but it works for arudimentary database, created for educational purposes.

        let index_state = self.index.lock().unwrap();

        // TODO: is it possible to avoid clonning records? Arc?
        let records_iter = index_state.records.clone().into_iter();

        drop(index_state);

        let records: Result<Vec<FileRecord>, crate::Error> = records_iter
            .map(|(_, value_metadata)| self.retrieve(&value_metadata))
            .collect();

        match records {
            Ok(records) => {
                let file = OpenOptions::new()
                    .write(true)
                    .open(&self.storage_filename)?;
                file.set_len(0)?;

                for record in records {
                    self.insert(record)?;
                }

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn rehydrate_index_from_disk(
        filename: &str,
    ) -> Result<Option<HashMap<String, ValueMetadata>>, crate::Error> {
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
                let index_record = ValueMetadata { offset, len };

                hydrated_index.insert(record.key, index_record);
            }

            offset += len;
        }

        if hydrated_index.is_empty() {
            return Ok(None);
        }

        Ok(Some(hydrated_index))
    }

    fn retrieve(&self, value_metadata: &ValueMetadata) -> Result<FileRecord, crate::Error> {
        let mut file = File::open(&self.storage_filename)?;

        let mut buffer = vec![0; value_metadata.len as usize];

        file.seek(SeekFrom::Start(value_metadata.offset))?;
        file.read_exact(&mut buffer)?;

        let record: FileRecord = serde_json::from_slice(&buffer)?;

        Ok(record)
    }

    fn insert(&self, file_record: FileRecord) -> Result<(), crate::Error> {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.storage_filename)?;

        let serialized_rec = file_record.serialize_with_escaping()?;

        let offset = file.metadata()?.len();
        let len = serialized_rec.len() as u64;

        file.write_all(serialized_rec.as_bytes())?;

        if !file_record.is_tombstone {
            let mut index_state_lock = self.index.lock().unwrap();

            index_state_lock
                .records
                .insert(file_record.key, ValueMetadata { offset, len });

            drop(index_state_lock);
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<FileRecord>, crate::Error> {
        let index_state = self.index.lock().unwrap();

        if let Some(index_record) = index_state.records.get(key) {
            let file_record = self.retrieve(index_record)?;

            if file_record.is_tombstone {
                return Ok(None);
            }

            Ok(Some(file_record))
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

    pub fn delete(&self, key: String) -> Result<Option<()>, crate::Error> {
        let mut index_state_lock = self.index.lock().unwrap();
        let index_record = index_state_lock.records.get(key.as_str());

        if index_record.is_none() {
            return Ok(None);
        }

        index_state_lock.records.remove(&key);

        drop(index_state_lock);

        let tombstone_record = FileRecord::new(key, None, true);

        self.insert(tombstone_record)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Result<Db, crate::Error> {
        let test_filename = "tests_store.dat";

        let file = OpenOptions::new().write(true).open(test_filename)?;
        file.set_len(0)?;

        Ok(Db::new(test_filename))
    }

    #[test]
    fn test_insertion() -> Result<(), crate::Error> {
        let db = setup_db()?;

        let key = "test_key".to_string();
        let value = Bytes::from("test_value".to_string());

        db.set(key.clone(), value)?;

        let retrieved_record = db.get(&key)?.unwrap();
        assert_eq!(retrieved_record.key, key);
        assert_eq!(retrieved_record.value, Some("test_value".to_string()));

        Ok(())
    }

    #[test]
    fn test_retrieval() -> Result<(), crate::Error> {
        todo!();
    }

    #[test]
    fn test_deletion() -> Result<(), crate::Error> {
        todo!();
    }

    #[test]
    fn test_compaction() -> Result<(), crate::Error> {
        todo!();
    }
}
