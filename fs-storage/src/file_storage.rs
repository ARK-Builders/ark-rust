use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::time::SystemTime;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::base_storage::{BaseStorage, SyncStatus};
use crate::monoid::Monoid;
use crate::utils::read_version_2_fs;
use data_error::{ArklibError, Result};

/*
Note on `FileStorage` Versioning:

`FileStorage` is a basic key-value storage system that persists data to disk.

In version 2, `FileStorage` stored data in a plaintext format.
Starting from version 3, data is stored in JSON format.

For backward compatibility, we provide a helper function `read_version_2_fs` to read version 2 format.
*/
const STORAGE_VERSION: i32 = 3;

/// Represents a file storage system that persists data to disk.
pub struct FileStorage<K, V>
where
    K: Ord,
{
    /// Label for logging
    label: String,
    /// Path to the underlying file where data is persisted
    path: PathBuf,
    /// Last modified time of internal mapping. This becomes equal to
    /// `written_to_disk` only when data is written or read from disk.
    modified: SystemTime,
    /// Last time the data was written to disk. This becomes equal to
    /// `modified` only when data is written or read from disk.
    written_to_disk: SystemTime,
    data: FileStorageData<K, V>,
}

/// A struct that represents the data stored in a [`FileStorage`] instance.
///
///
/// This is the data that is serialized and deserialized to and from disk.
#[derive(Serialize, Deserialize)]
pub struct FileStorageData<K, V>
where
    K: Ord,
{
    version: i32,
    entries: BTreeMap<K, V>,
}

impl<K, V> AsRef<BTreeMap<K, V>> for FileStorageData<K, V>
where
    K: Ord,
{
    fn as_ref(&self) -> &BTreeMap<K, V> {
        &self.entries
    }
}

impl<K, V> FileStorage<K, V>
where
    K: Ord
        + Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::str::FromStr,
    V: Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::str::FromStr
        + Monoid<V>,
{
    /// Create a new file storage with a diagnostic label and file path
    /// The storage will be initialized using the disk data, if the path exists
    ///
    /// Note: if the file storage already exists, the data will be read from the file
    /// without overwriting it.
    pub fn new(label: String, path: &Path) -> Result<Self> {
        let time = SystemTime::now();
        let mut storage = Self {
            label,
            path: PathBuf::from(path),
            modified: time,
            written_to_disk: time,
            data: FileStorageData {
                version: STORAGE_VERSION,
                entries: BTreeMap::new(),
            },
        };

        if Path::exists(path) {
            storage.read_fs()?;
        }

        Ok(storage)
    }

    /// Load mapping from file
    fn load_fs_data(&self) -> Result<FileStorageData<K, V>> {
        if !self.path.exists() {
            return Err(ArklibError::Storage(
                self.label.clone(),
                "File does not exist".to_owned(),
            ));
        }

        // First check if the file starts with "version: 2"
        let file_content = std::fs::read_to_string(&self.path)?;
        if file_content.starts_with("version: 2") {
            // Attempt to parse the file using the legacy version 2 storage format of FileStorage.
            match read_version_2_fs(&self.path) {
                Ok(data) => {
                    log::info!(
                        "Version 2 storage format detected for {}",
                        self.label
                    );
                    let data = FileStorageData {
                        version: 2,
                        entries: data,
                    };
                    return Ok(data);
                }
                Err(_) => {
                    return Err(ArklibError::Storage(
                        self.label.clone(),
                        "Storage seems to be version 2, but failed to parse"
                            .to_owned(),
                    ));
                }
            };
        }

        let file = fs::File::open(&self.path)?;
        let data: FileStorageData<K, V> = serde_json::from_reader(file)
            .map_err(|err| {
                ArklibError::Storage(self.label.clone(), err.to_string())
            })?;
        let version = data.version;
        if version != STORAGE_VERSION {
            return Err(ArklibError::Storage(
                self.label.clone(),
                format!(
                    "Storage version mismatch: expected {}, got {}",
                    STORAGE_VERSION, version
                ),
            ));
        }

        Ok(data)
    }
}

impl<K, V> BaseStorage<K, V> for FileStorage<K, V>
where
    K: Ord
        + Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::str::FromStr,
    V: Clone
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::str::FromStr
        + Monoid<V>,
{
    /// Set a key-value pair in the internal mapping
    fn set(&mut self, key: K, value: V) {
        self.data.entries.insert(key, value);
        self.modified = std::time::SystemTime::now();
    }

    /// Remove an entry from the internal mapping given a key
    fn remove(&mut self, id: &K) -> Result<()> {
        self.data.entries.remove(id).ok_or_else(|| {
            ArklibError::Storage(self.label.clone(), "Key not found".to_owned())
        })?;
        self.modified = std::time::SystemTime::now();
        Ok(())
    }

    /// Compare the timestamp of the storage file
    /// with the timestamp of the in-memory storage and the last written
    /// to time to determine if either of the two requires syncing.
    fn sync_status(&self) -> Result<SyncStatus> {
        let file_updated = fs::metadata(&self.path)?.modified()?;

        // Determine the synchronization status based on the modification times
        // Conditions:
        // 1. If both the in-memory storage and the storage on disk have been modified
        //    since the last write, then the storage is diverged.
        // 2. If only the in-memory storage has been modified since the last write,
        //    then the storage on disk is stale.
        // 3. If only the storage on disk has been modified since the last write,
        //    then the in-memory storage is stale.
        // 4. If neither the in-memory storage nor the storage on disk has been modified
        //    since the last write, then the storage is in sync.
        let status = match (
            self.modified > self.written_to_disk,
            file_updated > self.written_to_disk,
        ) {
            (true, true) => SyncStatus::Diverge,
            (true, false) => SyncStatus::StorageStale,
            (false, true) => SyncStatus::MappingStale,
            (false, false) => SyncStatus::InSync,
        };

        log::info!("{} sync status is {}", self.label, status);
        Ok(status)
    }

    /// Sync the in-memory storage with the storage on disk
    fn sync(&mut self) -> Result<()> {
        match self.sync_status()? {
            SyncStatus::InSync => Ok(()),
            SyncStatus::MappingStale => self.read_fs().map(|_| ()),
            SyncStatus::StorageStale => self.write_fs().map(|_| ()),
            SyncStatus::Diverge => {
                let data = self.load_fs_data()?;
                self.merge_from(&data)?;
                self.write_fs()?;
                Ok(())
            }
        }
    }

    /// Read the data from file
    fn read_fs(&mut self) -> Result<&BTreeMap<K, V>> {
        let data = self.load_fs_data()?;

        // Update file storage with loaded data
        self.modified = fs::metadata(&self.path)?.modified()?;
        self.written_to_disk = self.modified;
        self.data = data;

        Ok(&self.data.entries)
    }

    /// Get a value from the internal mapping
    fn get(&self, id: &K) -> Result<&V> {
        self.data.entries.get(id).ok_or_else(|| {
            ArklibError::Storage(self.label.clone(), "Key not found".to_owned())
        })
    }

    /// Write the data to file
    ///
    /// Update the modified timestamp in file metadata to avoid OS timing issues
    /// https://github.com/ARK-Builders/ark-rust/pull/63#issuecomment-2163882227
    fn write_fs(&mut self) -> Result<()> {
        let parent_dir = self.path.parent().ok_or_else(|| {
            ArklibError::Storage(
                self.label.clone(),
                "Failed to get parent directory".to_owned(),
            )
        })?;
        fs::create_dir_all(parent_dir)?;
        let mut file = File::create(&self.path)?;
        file.write_all(serde_json::to_string_pretty(&self.data)?.as_bytes())?;
        file.flush()?;

        let new_timestamp = SystemTime::now();
        file.set_modified(new_timestamp)?;
        file.sync_all()?;

        self.modified = new_timestamp;
        self.written_to_disk = new_timestamp;

        log::info!(
            "{} {} entries have been written",
            self.label,
            self.data.entries.len()
        );
        Ok(())
    }

    /// Erase the file from disk
    fn erase(&self) -> Result<()> {
        fs::remove_file(&self.path).map_err(|err| {
            ArklibError::Storage(self.label.clone(), err.to_string())
        })
    }

    /// Merge the data from another storage instance into this storage instance
    fn merge_from(&mut self, other: impl AsRef<BTreeMap<K, V>>) -> Result<()>
    where
        V: Monoid<V>,
    {
        let other_entries = other.as_ref();
        for (key, value) in other_entries {
            if let Some(existing_value) = self.data.entries.get(key) {
                let resolved_value = V::combine(existing_value, value);
                self.set(key.clone(), resolved_value);
            } else {
                self.set(key.clone(), value.clone())
            }
        }
        self.modified = std::time::SystemTime::now();
        Ok(())
    }
}

impl<K, V> AsRef<BTreeMap<K, V>> for FileStorage<K, V>
where
    K: Ord,
{
    fn as_ref(&self) -> &BTreeMap<K, V> {
        &self.data.entries
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs};
    use tempdir::TempDir;

    use crate::{
        base_storage::{BaseStorage, SyncStatus},
        file_storage::FileStorage,
    };

    #[test]
    fn test_file_storage_write_read() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("test_storage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path).unwrap();

        file_storage.set("key1".to_string(), "value1".to_string());
        file_storage.set("key2".to_string(), "value2".to_string());

        assert!(file_storage.remove(&"key1".to_string()).is_ok());
        file_storage
            .write_fs()
            .expect("Failed to write data to disk");
        let data_read: &BTreeMap<_, _> = file_storage
            .read_fs()
            .expect("Failed to read data from disk");

        assert_eq!(data_read.len(), 1);
        assert_eq!(data_read.get("key2").map(|v| v.as_str()), Some("value2"))
    }

    #[test]
    fn test_file_storage_auto_delete() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("test_storage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path).unwrap();

        file_storage.set("key1".to_string(), "value1".to_string());
        file_storage.set("key1".to_string(), "value2".to_string());
        assert!(file_storage.write_fs().is_ok());
        assert_eq!(storage_path.exists(), true);

        if let Err(err) = file_storage.erase() {
            panic!("Failed to delete file: {:?}", err);
        }
        assert!(!storage_path.exists());
    }

    #[test]
    fn test_file_metadata_timestamp_updated() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("teststorage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path).unwrap();
        file_storage.write_fs().unwrap();

        file_storage.set("key1".to_string(), "value1".to_string());
        let before_write = fs::metadata(&storage_path)
            .unwrap()
            .modified()
            .unwrap();
        file_storage.write_fs().unwrap();
        let after_write = fs::metadata(&storage_path)
            .unwrap()
            .modified()
            .unwrap();
        println!(
            "before_write: {:?}, after_write: {:?}",
            before_write, after_write
        );
        assert!(before_write < after_write);
    }

    #[test]
    fn test_file_storage_is_storage_updated() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path = temp_dir.path().join("teststorage.txt");

        let mut file_storage =
            FileStorage::new("TestStorage".to_string(), &storage_path).unwrap();
        file_storage.write_fs().unwrap();
        assert_eq!(file_storage.sync_status().unwrap(), SyncStatus::InSync);

        file_storage.set("key1".to_string(), "value1".to_string());
        assert_eq!(
            file_storage.sync_status().unwrap(),
            SyncStatus::StorageStale
        );
        file_storage.write_fs().unwrap();
        assert_eq!(file_storage.sync_status().unwrap(), SyncStatus::InSync);

        // External data manipulation
        let mut mirror_storage =
            FileStorage::new("MirrorTestStorage".to_string(), &storage_path)
                .unwrap();
        assert_eq!(mirror_storage.sync_status().unwrap(), SyncStatus::InSync);

        mirror_storage.set("key1".to_string(), "value3".to_string());
        assert_eq!(
            mirror_storage.sync_status().unwrap(),
            SyncStatus::StorageStale
        );

        mirror_storage.write_fs().unwrap();
        assert_eq!(mirror_storage.sync_status().unwrap(), SyncStatus::InSync);

        // receive updates from external data manipulation
        assert_eq!(
            file_storage.sync_status().unwrap(),
            SyncStatus::MappingStale
        );
        file_storage.read_fs().unwrap();
        assert_eq!(file_storage.sync_status().unwrap(), SyncStatus::InSync);
        assert_eq!(mirror_storage.sync_status().unwrap(), SyncStatus::InSync);
    }

    #[test]
    fn test_monoid_combine() {
        let temp_dir =
            TempDir::new("tmp").expect("Failed to create temporary directory");
        let storage_path1 = temp_dir.path().join("teststorage1.txt");
        let storage_path2 = temp_dir.path().join("teststorage2.txt");

        let mut file_storage_1 =
            FileStorage::new("TestStorage1".to_string(), &storage_path1)
                .unwrap();

        let mut file_storage_2 =
            FileStorage::new("TestStorage2".to_string(), &storage_path2)
                .unwrap();

        file_storage_1.set("key1".to_string(), 2);
        file_storage_1.set("key2".to_string(), 6);

        file_storage_2.set("key1".to_string(), 3);
        file_storage_2.set("key3".to_string(), 9);

        file_storage_1
            .merge_from(&file_storage_2)
            .unwrap();
        assert_eq!(file_storage_1.as_ref().get("key1"), Some(&3));
        assert_eq!(file_storage_1.as_ref().get("key2"), Some(&6));
        assert_eq!(file_storage_1.as_ref().get("key3"), Some(&9));
    }
}
