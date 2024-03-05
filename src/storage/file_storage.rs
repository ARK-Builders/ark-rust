use std::fmt::Debug;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;
use std::time::SystemTime;
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{ArklibError, Result};

const LOG_PREFIX: &str = "[file-storage]";
const STORAGE_VERSION: i32 = 2;
const STORAGE_VERSION_PREFIX: &str = "version ";
const KEY_VALUE_SEPARATOR: char = ':';

pub struct FileStorage {
    log_prefix: String,
    label: String,
    path: PathBuf,
    timestamp: SystemTime,
}

impl FileStorage {
    /// Create a new file storage with a diagnostic label and file path
    pub fn new(label: String, path: &Path) -> Self {
        Self {
            label: label.clone(),
            log_prefix: format!("{} {}", LOG_PREFIX, label).to_string(),
            path: PathBuf::from(path),
            timestamp: SystemTime::now(),
        }
    }

    /// Read data from disk
    ///
    /// Data is read as a key value pairs separated by a symbol and stored
    /// in a [HashMap] with a generic key K and V value. A handler
    /// is called on the data after reading it.
    pub fn read_file<K, V>(
        &self,
        mut handle: impl FnMut(HashMap<K, V>),
    ) -> Result<()>
    where
        K: FromStr + std::hash::Hash + std::cmp::Eq + Debug,
        V: FromStr + Debug,
        ArklibError: From<<K as FromStr>::Err>,
        ArklibError: From<<V as FromStr>::Err>,
    {
        let new_timestamp = fs::metadata(&self.path)?.modified()?;
        log::info!(
            "timestamp of storage file {:?} is {:?}",
            self.path,
            self.timestamp
        );

        if self.timestamp >= new_timestamp {
            return Ok(());
        }

        log::info!("the file was modified externally, merging");

        let value_by_id = self.read_file_from_disk()?;
        if !value_by_id.is_empty() {
            handle(value_by_id);
        }

        Ok(())
    }

    fn read_file_from_disk<K, V>(&self) -> Result<HashMap<K, V>>
    where
        K: FromStr + std::hash::Hash + std::cmp::Eq + Debug,
        V: FromStr + Debug,
        ArklibError: From<<K as FromStr>::Err>,
        ArklibError: From<<V as FromStr>::Err>,
    {
        let file = fs::File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        match lines.next() {
            Some(Ok(header)) => {
                self.verify_version(&header)?;

                let mut value_by_id = HashMap::new();
                for line in lines {
                    let line = line?;
                    if line.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> =
                        line.split(KEY_VALUE_SEPARATOR).collect();
                    let id = K::from_str(parts[0])?;
                    let value = V::from_str(parts[1])?;
                    value_by_id.insert(id, value);
                }

                Ok(value_by_id)
            }
            Some(Err(e)) => Err(e.into()),
            None => Err(ArklibError::Storage(
                self.label.clone(),
                "Storage file is missing header".to_owned(),
            )),
        }
    }

    /// Write data to file
    ///
    /// Data is a key-value mapping between [ResourceId] and a generic Value
    pub fn write_file<K, V>(
        &mut self,
        value_by_id: &HashMap<K, V>,
    ) -> Result<()>
    where
        K: Display,
        V: Display,
    {
        fs::create_dir_all(self.path.parent().unwrap())?;
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(
            format!("{}{}\n", STORAGE_VERSION_PREFIX, STORAGE_VERSION)
                .as_bytes(),
        )?;

        for (id, value) in value_by_id {
            writer.write_all(
                format!("{}{}{}\n", id, KEY_VALUE_SEPARATOR, value).as_bytes(),
            )?;
        }

        let new_timestamp = fs::metadata(&self.path)?.modified()?;
        if new_timestamp == self.timestamp {
            return Err(anyhow::anyhow!("Timestamp didn't update").into());
        }
        self.timestamp = new_timestamp;

        log::info!(
            "{} {} entries has been written",
            self.log_prefix,
            value_by_id.len()
        );
        Ok(())
    }

    /// Verify the version stored in the file header
    fn verify_version(&self, header: &str) -> Result<()> {
        if !header.starts_with(STORAGE_VERSION_PREFIX) {
            return Err(ArklibError::Storage(
                self.label.clone(),
                "Unknown storage version".to_owned(),
            ));
        }

        let version = header[STORAGE_VERSION_PREFIX.len()..]
            .parse::<i32>()
            .unwrap();

        match version.cmp(&STORAGE_VERSION) {
            std::cmp::Ordering::Equal => Ok(()),
            std::cmp::Ordering::Less => Err(ArklibError::Storage(
                self.label.clone(),
                "Storage format is newer than the app".to_owned(),
            )),
            std::cmp::Ordering::Greater => Err(ArklibError::Storage(
                self.label.clone(),
                "Storage format is older than the app".to_owned(),
            )),
        }
    }
}

impl Drop for FileStorage {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            log::error!(
                "{} Failed to delete file because of error: {}",
                self.log_prefix,
                e
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use tempfile::TempDir;

    use crate::storage::file_storage::FileStorage;

    #[test]
    fn test_file_storage_write_read() {
        let temp_dir =
            TempDir::new().expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("test_storage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path);

        let mut data_to_write = HashMap::new();
        data_to_write.insert("key1".to_string(), "value1".to_string());
        data_to_write.insert("key2".to_string(), "value2".to_string());

        file_storage
            .write_file(&data_to_write)
            .expect("Failed to write data to disk");

        let data_read: HashMap<_, _> = file_storage
            .read_file_from_disk()
            .expect("Failed to read data from disk");

        assert_eq!(data_read, data_to_write);
    }

    #[test]
    fn test_file_storage_auto_delete() {
        let temp_dir =
            TempDir::new().expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("test_storage.txt");

        // File storage should be dropped and the file deleted after this scope
        {
            let mut file_storage =
                FileStorage::new("TestStorage".to_string(), &storage_path);

            let mut data_to_write = HashMap::new();
            data_to_write.insert("key1".to_string(), "value1".to_string());
            data_to_write.insert("key2".to_string(), "value2".to_string());

            file_storage
                .write_file(&data_to_write)
                .expect("Failed to write data to disk");

            assert_eq!(storage_path.exists(), true);
        }

        assert_eq!(storage_path.exists(), false);
    }
}
