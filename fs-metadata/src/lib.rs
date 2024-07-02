use data_error::Result;
use fs_atomic_versions::atomic::{modify_json, AtomicFile};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, io::Read, path::Path};

use data_resource::ResourceId;
use fs_storage::ARK_FOLDER;

pub const METADATA_STORAGE_FOLDER: &str = "cache/metadata";

pub fn store_metadata<
    S: Serialize + DeserializeOwned + Clone + Debug,
    P: AsRef<Path>,
    Id: ResourceId,
>(
    root: P,
    id: Id,
    metadata: &S,
) -> Result<()> {
    let file = AtomicFile::new(
        root.as_ref()
            .join(ARK_FOLDER)
            .join(METADATA_STORAGE_FOLDER)
            .join(id.to_string()),
    )?;
    modify_json(&file, |current_meta: &mut Option<S>| {
        let new_meta = metadata.clone();
        match current_meta {
            Some(file_data) => {
                // This is fine because generated metadata must always
                // be generated in same way on any device.
                *file_data = new_meta;
                // Different versions of the lib should
                // not be used on synced devices.
            }
            None => *current_meta = Some(new_meta),
        }
    })?;
    Ok(())
}

/// The file must exist if this method is called
#[allow(dead_code)]
pub fn load_raw_metadata<P: AsRef<Path>, Id: ResourceId>(
    root: P,
    id: Id,
) -> Result<Vec<u8>> {
    let storage = root
        .as_ref()
        .join(ARK_FOLDER)
        .join(METADATA_STORAGE_FOLDER)
        .join(id.to_string());
    let file = AtomicFile::new(storage)?;
    let read_file = file.load()?;
    if let Some(mut real_file) = read_file.open()? {
        let mut content = vec![];
        real_file.read_to_end(&mut content)?;
        Ok(content)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ))?
    }
}

#[cfg(test)]
mod tests {
    use fs_atomic_versions::initialize;

    use super::*;
    use tempdir::TempDir;

    use dev_hash::Crc32;

    use std::collections::HashMap;
    type TestMetadata = HashMap<String, String>;

    #[test]
    fn test_store_and_load() {
        initialize();

        let dir = TempDir::new("arklib_test").unwrap();
        let root = dir.path();
        log::debug!("temporary root: {}", root.display());

        let id = Crc32(0x342a3d4a);

        let mut meta = TestMetadata::new();
        meta.insert("abc".to_string(), "def".to_string());
        meta.insert("xyz".to_string(), "123".to_string());

        store_metadata(root, id.clone(), &meta).unwrap();

        let bytes = load_raw_metadata(root, id).unwrap();
        let prop2: TestMetadata = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(meta, prop2);
    }
}
