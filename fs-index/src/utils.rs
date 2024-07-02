use std::{fs, io::BufReader, path::Path};

use data_error::{ArklibError, Result};
use data_resource::ResourceId;
use fs_storage::{ARK_FOLDER, INDEX_PATH};

use crate::ResourceIndex;

/// A helper function to check if the entry should be indexed (not hidden)
pub fn should_index(entry: &walkdir::DirEntry) -> bool {
    !entry
        .file_name()
        .to_string_lossy()
        .starts_with('.')
}

/// Load the index from the file system
fn load_index<P: AsRef<Path>, Id: ResourceId>(
    root_path: P,
) -> Result<ResourceIndex<Id>> {
    let index_path = Path::new(ARK_FOLDER).join(INDEX_PATH);
    let index_path = fs::canonicalize(root_path.as_ref())?.join(index_path);
    let index_file = fs::File::open(index_path)?;
    let reader = BufReader::new(index_file);
    let index = serde_json::from_reader(reader)?;

    Ok(index)
}

/// Load the index from the file system, or build a new index if it doesn't
/// exist
///
/// If `update` is true, the index will be updated and stored after loading
/// it.
pub fn load_or_build_index<P: AsRef<Path>, Id: ResourceId>(
    root_path: P,
    update: bool,
) -> Result<ResourceIndex<Id>> {
    log::debug!(
        "Attempting to load or build index at root path: {:?}",
        root_path.as_ref()
    );

    let index_path = Path::new(ARK_FOLDER).join(INDEX_PATH);
    let index_path = fs::canonicalize(root_path.as_ref())?.join(index_path);
    log::trace!("Index path: {:?}", index_path);

    if index_path.exists() {
        log::trace!("Index file exists, loading index");

        let mut index = load_index(root_path)?;
        if update {
            log::trace!("Updating loaded index");

            index.update_all()?;
            index.store()?;
        }
        Ok(index)
    } else {
        log::trace!("Index file does not exist, building index");

        // Build a new index if it doesn't exist and store it
        let index = ResourceIndex::build(root_path.as_ref())?;
        index.store().map_err(|e| {
            ArklibError::Path(format!("Failed to store index: {}", e))
        })?;
        Ok(index)
    }
}
