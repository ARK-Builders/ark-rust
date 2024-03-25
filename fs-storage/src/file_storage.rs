use std::fmt::Debug;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;
use std::time::SystemTime;
use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use data_error::{ArklibError, Result};

const STORAGE_VERSION: i32 = 2;
const STORAGE_VERSION_PREFIX: &str = "version ";
const KEY_VALUE_SEPARATOR: char = ':';

pub struct FileStorage {
    label: String,
    path: PathBuf,
    timestamp: SystemTime,
}

impl FileStorage {
    /// Create a new file storage with a diagnostic label and file path
    pub fn new(label: String, path: &Path) -> Self {
        Self {
            label,
            path: PathBuf::from(path),
            timestamp: SystemTime::now(),
        }
    }

    /// Check if underlying file has been updated
    ///
    /// This check can be used before reading the file.
    pub fn is_file_updated(&self) -> Result<bool> {
        let file_timestamp = fs::metadata(&self.path)?.modified()?;
        Ok(self.timestamp < file_timestamp)
    }

    /// Read data from disk
    ///
    /// Data is read as a key value pairs separated by a symbol and stored
    /// in a [BTreeMap] with a generic key K and V value. A handler
    /// is called on the data after reading it.
    pub fn read_file<K, V>(&mut self) -> Result<BTreeMap<K, V>>
    where
        K: FromStr + std::hash::Hash + std::cmp::Eq + Debug + std::cmp::Ord,
        V: FromStr + Debug,
        ArklibError: From<<K as FromStr>::Err>,
        ArklibError: From<<V as FromStr>::Err>,
    {
        let file = fs::File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let new_timestamp = fs::metadata(&self.path)?.modified()?;
        match lines.next() {
            Some(header) => {
                let header = header?;
                self.verify_version(&header)?;

                let mut value_by_id = BTreeMap::new();
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

                self.timestamp = new_timestamp;
                Ok(value_by_id)
            }
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
        value_by_id: &BTreeMap<K, V>,
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
            return Err("Timestamp didn't update".into());
        }
        self.timestamp = new_timestamp;

        log::info!(
            "{} {} entries has been written",
            self.label,
            value_by_id.len()
        );
        Ok(())
    }

    pub fn erase(&self) {
        if let Err(e) = fs::remove_file(&self.path) {
            log::error!(
                "{} Failed to delete file because of error: {}",
                self.label,
                e
            )
        }
    }

    /// Verify the version stored in the file header
    fn verify_version(&self, header: &str) -> Result<()> {
        if !header.starts_with(STORAGE_VERSION_PREFIX) {
            return Err(ArklibError::Storage(
                self.label.clone(),
                "Unknown storage version prefix".to_owned(),
            ));
        }

        let version = header[STORAGE_VERSION_PREFIX.len()..]
            .parse::<i32>()
            .map_err(|_err| {
                <&str as Into<ArklibError>>::into(
                    "Unable to parse storage version",
                )
            })?;

        if version != STORAGE_VERSION {
            return Err(ArklibError::Storage(
                self.label.clone(),
                format!(
                    "Storage version mismatch: expected {}, found {}",
                    STORAGE_VERSION, version
                ),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tempdir::TempDir;

    use crate::file_storage::FileStorage;

    #[test]
    fn test_file_storage_write_read() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("test_storage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path);

        let mut data_to_write = BTreeMap::new();
        data_to_write.insert("key1".to_string(), "value1".to_string());
        data_to_write.insert("key2".to_string(), "value2".to_string());

        file_storage
            .write_file(&data_to_write)
            .expect("Failed to write data to disk");

        let data_read: BTreeMap<_, _> = file_storage
            .read_file()
            .expect("Failed to read data from disk");

        assert_eq!(data_read, data_to_write);
    }

    #[test]
    fn test_file_storage_auto_delete() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("test_storage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path);

        let mut data_to_write = BTreeMap::new();
        data_to_write.insert("key1".to_string(), "value1".to_string());
        data_to_write.insert("key2".to_string(), "value2".to_string());

        file_storage
            .write_file(&data_to_write)
            .expect("Failed to write data to disk");

        assert_eq!(storage_path.exists(), true);

        file_storage.erase();

        assert_eq!(storage_path.exists(), false);
    }
}
