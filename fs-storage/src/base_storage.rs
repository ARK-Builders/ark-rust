use data_error::Result;
use std::collections::BTreeMap;

#[cfg(feature = "jni-bindings")]
use jnix::{FromJava, IntoJava};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "jni-bindings", derive(FromJava, IntoJava))]
#[jnix(class_name = "dev/arkbuilders/core/FileStorage$SyncStatus")]
/// Represents the synchronization status of the storage.
pub enum SyncStatus {
    /// No synchronization needed.
    InSync,
    /// In-memory key-value mapping is stale.
    MappingStale,
    /// External file system storage is stale.
    StorageStale,
    /// In-memory key-value mapping and external file system storage diverge.
    Diverge,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::InSync => write!(f, "InSync"),
            SyncStatus::MappingStale => write!(f, "MappingStale"),
            SyncStatus::StorageStale => write!(f, "StorageStale"),
            SyncStatus::Diverge => write!(f, "Diverge"),
        }
    }
}

/// The `BaseStorage` trait represents a key-value mapping that is written to
/// the file system.
///
/// This trait provides methods to create or update entries in the internal
/// mapping, remove entries from the internal mapping, determine if the
/// in-memory model or the underlying storage requires syncing, scan and load
/// the mapping from the filesystem, write the mapping to the filesystem, and
/// remove all stored data.
///
/// The trait also includes a method to merge values from another key-value
/// mapping.
///
/// Note: The trait does not write to storage by default. It is up to the
/// implementor to decide when to read or write to storage
/// based on `SyncStatus`. This is to allow for trading off between performance
/// and consistency.
pub trait BaseStorage<K, V>: AsRef<BTreeMap<K, V>> {
    /// Create or update an entry in the internal mapping.
    fn set(&mut self, id: K, value: V);

    /// Remove an entry from the internal mapping.
    fn remove(&mut self, id: &K) -> Result<()>;

    /// Get [`SyncStatus`] of the storage
    fn sync_status(&self) -> Result<SyncStatus>;

    /// Sync the in-memory storage with the storage on disk
    fn sync(&mut self) -> Result<()>;

    /// Scan and load the key-value mapping
    /// from pre-configured location in the filesystem.
    fn read_fs(&mut self) -> Result<&BTreeMap<K, V>>;

    /// Write the internal key-value mapping
    /// to pre-configured location in the filesystem.
    fn write_fs(&mut self) -> Result<()>;

    /// Erase data stored on the filesystem
    fn erase(&self) -> Result<()>;

    /// Merge values from another key-value mapping.
    fn merge_from(&mut self, other: impl AsRef<BTreeMap<K, V>>) -> Result<()>;
}
