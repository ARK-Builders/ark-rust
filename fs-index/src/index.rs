use std::{
    collections::HashMap,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::anyhow;
use log;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use data_error::{ArklibError, Result};
use data_resource::ResourceId;
use fs_storage::{ARK_FOLDER, INDEX_PATH};

use crate::utils::should_index;

/// Represents a resource in the index
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, Serialize, Deserialize,
)]
pub struct IndexedResource<Id> {
    /// The unique identifier of the resource
    id: Id,
    /// The path of the resource, relative to the root path
    path: PathBuf,
    /// The last modified time of the resource (from the file system metadata)
    last_modified: SystemTime,
}

impl<Id> IndexedResource<Id> {
    /// Create a new indexed resource
    pub fn new(id: Id, path: PathBuf, last_modified: SystemTime) -> Self {
        IndexedResource {
            id,
            path,
            last_modified,
        }
    }

    /// Return the ID of the resource
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Return the path of the resource
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the last modified time of the resource
    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }
}

/// Represents the index of resources in a directory.
///
/// [`ResourceIndex`] provides functionality for managing a directory index,
/// including tracking changes, and querying resources.
///
/// #### Reactive API
/// - [`ResourceIndex::update_all`]: Method to update the index by rescanning
///   files and returning changes (additions/deletions/updates).
///
/// #### Snapshot API
/// - [`ResourceIndex::get_resources_by_id`]: Query resources from the index by
///   ID.
/// - [`ResourceIndex::get_resource_by_path`]: Query a resource from the index
///   by its path.
///
/// #### Track API
/// Allows for fine-grained control over tracking changes in the index
/// - [`ResourceIndex::track_addition`]: Track a newly added file (checks if the
///   file exists in the file system).
/// - [`ResourceIndex::track_removal`]: Track the deletion of a file (checks if
///   the file was actually deleted).
/// - [`ResourceIndex::track_modification`]: Track an update on a single file.
///
/// ## Examples
/// ```no_run
/// use std::path::Path;
/// use fs_index::{ResourceIndex, load_or_build_index};
/// use dev_hash::Crc32;
///
/// // Define the root path
/// let root_path = Path::new("animals");
///
/// // Build the index
/// let index: ResourceIndex<Crc32> = ResourceIndex::build(root_path).expect("Failed to build index");
/// // Store the index
/// index.store().expect("Failed to store index");
///
/// // Load the stored index
/// let mut loaded_index: ResourceIndex<Crc32> = load_or_build_index(root_path, false).expect("Failed to load index");
///
/// // Update the index
/// loaded_index.update_all().expect("Failed to update index");
///
/// // Get a resource by path
/// let _resource = loaded_index
///     .get_resource_by_path("cat.txt")
///     .expect("Resource not found");
///
/// // Track the removal of a file
/// loaded_index
///     .track_removal(Path::new("cat.txt"))
///     .expect("Failed to track removal");
///
/// // Track the addition of a new file
/// loaded_index
///     .track_addition(Path::new("dog.txt"))
///     .expect("Failed to track addition");
///
/// // Track the modification of a file
/// loaded_index
///     .track_modification(Path::new("dog.txt"))
///     .expect("Failed to track modification");
/// ```
#[derive(Clone, Debug)]
pub struct ResourceIndex<Id>
where
    Id: Eq + Hash,
{
    /// The root path of the index (canonicalized)
    pub(crate) root: PathBuf,
    /// A map from resource IDs to resources
    ///
    /// Multiple resources can have the same ID (e.g., due to hash collisions
    /// or files with the same content)
    pub(crate) id_to_resources: HashMap<Id, Vec<IndexedResource<Id>>>,
    /// A map from resource paths to resources
    pub(crate) path_to_resource: HashMap<PathBuf, IndexedResource<Id>>,
}

/// Represents the result of an update operation on the ResourceIndex
#[derive(PartialEq, Debug)]
pub struct IndexUpdate<Id: ResourceId> {
    /// Resources that were added during the update
    added: Vec<IndexedResource<Id>>,
    /// Resources that were modified during the update
    modified: Vec<IndexedResource<Id>>,
    /// Resources that were removed during the update
    removed: Vec<IndexedResource<Id>>,
}

impl<Id: ResourceId> IndexUpdate<Id> {
    /// Return the resources that were added during the update
    pub fn added(&self) -> &Vec<IndexedResource<Id>> {
        &self.added
    }

    /// Return the resources that were modified during the update
    pub fn modified(&self) -> &Vec<IndexedResource<Id>> {
        &self.modified
    }

    /// Return the resources that were removed during the update
    pub fn removed(&self) -> &Vec<IndexedResource<Id>> {
        &self.removed
    }
}

impl<Id: ResourceId> ResourceIndex<Id> {
    /// Return the number of resources in the index
    pub fn len(&self) -> usize {
        self.path_to_resource.len()
    }

    /// Return true if the index is empty
    pub fn is_empty(&self) -> bool {
        self.path_to_resource.is_empty()
    }

    /// Return the root path of the index
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the resources in the index
    pub fn resources(&self) -> Vec<IndexedResource<Id>> {
        // Using path_to_resource so to avoid not collecting duplicates
        self.path_to_resource.values().cloned().collect()
    }

    /// Return the ID collisions
    ///
    /// **Note**: If you are using a cryptographic hash function, collisions
    /// should be files with the same content. If you are using a
    /// non-cryptographic hash function, collisions can be files with the
    /// same content or files whose content hash to the same value.
    pub fn collisions(&self) -> HashMap<Id, Vec<IndexedResource<Id>>> {
        // Filter out IDs with only one resource
        self.id_to_resources
            .iter()
            .filter(|(_, resources)| resources.len() > 1)
            .map(|(id, resources)| (id.clone(), resources.clone()))
            .collect()
    }

    /// Return the number of ID collisions
    ///
    /// **Note**: If you are using a cryptographic hash function, collisions
    /// should be files with the same content. If you are using a
    /// non-cryptographic hash function, collisions can be files with the
    /// same content or files whose content hash to the same value.
    pub fn num_collisions(&self) -> usize {
        self.id_to_resources
            .values()
            .filter(|resources| resources.len() > 1)
            .map(|resources| resources.len())
            .sum()
    }

    /// Save the index to the file system (as a JSON file in
    /// <root_path>/ARK_FOLDER/INDEX_PATH)
    pub fn store(&self) -> Result<()> {
        let ark_folder = self.root.join(ARK_FOLDER);
        let index_path = ark_folder.join(INDEX_PATH);
        log::debug!("Storing index at: {:?}", index_path);

        fs::create_dir_all(&ark_folder)?;
        let index_file = fs::File::create(index_path)?;
        serde_json::to_writer_pretty(index_file, self)?;

        Ok(())
    }

    /// Get resources by their ID
    ///
    /// Returns None if there is no resource with the given ID
    ///
    /// **Note**: This can return multiple resources with the same ID in case of
    /// hash collisions or files with the same content
    pub fn get_resources_by_id(
        &self,
        id: &Id,
    ) -> Option<&Vec<IndexedResource<Id>>> {
        self.id_to_resources.get(id)
    }

    /// Get a resource by its path
    ///
    /// Returns None if the resource does not exist
    ///
    /// **Note**: The path should be relative to the root path
    pub fn get_resource_by_path<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Option<&IndexedResource<Id>> {
        self.path_to_resource.get(path.as_ref())
    }

    /// Build a new index from the given root path
    pub fn build<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        log::debug!("Building index at root path: {:?}", root_path.as_ref());

        // Canonicalize the root path
        let root = fs::canonicalize(&root_path)?;
        let mut id_to_resources = HashMap::new();
        let mut path_to_resource = HashMap::new();

        // Loop through the root path and add resources to the index
        let walker = WalkDir::new(&root)
            .min_depth(1) // Skip the root directory
            .into_iter()
            .filter_entry(should_index); // Skip hidden files
        for entry in walker {
            let entry = entry.map_err(|e| {
                ArklibError::Path(format!("Error walking directory: {}", e))
            })?;
            // Ignore directories
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let metadata = fs::metadata(path)?;
            // Ignore empty files
            if metadata.len() == 0 {
                continue;
            }
            let last_modified = metadata.modified()?;
            let id = Id::from_path(path)?;
            // Path is relative to the root
            let path = path.strip_prefix(&root).map_err(|_| {
                ArklibError::Path("Error stripping prefix".to_string())
            })?;

            // Create the resource and add it to the index
            let resource = IndexedResource {
                id: id.clone(),
                path: path.to_path_buf(),
                last_modified,
            };
            path_to_resource.insert(resource.path.clone(), resource.clone());
            id_to_resources
                .entry(id)
                .or_insert_with(Vec::new)
                .push(resource);
        }

        Ok(ResourceIndex {
            root,
            id_to_resources,
            path_to_resource,
        })
    }

    /// Update the index with the latest information from the file system
    pub fn update_all(&mut self) -> Result<IndexUpdate<Id>> {
        log::debug!("Updating index at root path: {:?}", self.root);

        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();

        let new_index = ResourceIndex::build(&self.root)?;

        // Compare the new index with the old index
        let current_resources = self.resources();
        let new_resources = new_index.resources();
        for resource in new_resources.clone() {
            // If the resource is in the old index, check if it has been
            // modified
            if let Some(current_resource) =
                self.get_resource_by_path(&resource.path)
            {
                if current_resource != &resource {
                    modified.push(resource.clone());
                }
            }
            // If the resource is not in the old index, it has been added
            else {
                added.push(resource.clone());
            }
        }
        for resource in current_resources {
            // If the resource is not in the new index, it has been removed
            if !new_resources.contains(&resource) {
                removed.push(resource.clone());
            }
        }

        // Update the index with the new index and return the result
        *self = new_index;
        Ok(IndexUpdate {
            added,
            modified,
            removed,
        })
    }

    /// Track the addition of a newly added file to the resource index.
    ///
    /// This method checks if the file exists in the file system.
    ///
    /// # Arguments
    /// * `relative_path` - The path of the file to be added (relative to the
    ///   root path of the index).
    ///
    /// # Returns
    /// Returns `Ok(resource)` if the file was successfully added to the index.
    ///
    /// # Errors
    /// - If the file does not exist in the file system.
    /// - If there was an error calculating the checksum of the file.
    pub fn track_addition<P: AsRef<Path>>(
        &mut self,
        relative_path: P,
    ) -> Result<IndexedResource<Id>> {
        log::debug!("Tracking addition of file: {:?}", relative_path.as_ref());

        let path = relative_path.as_ref();
        let full_path = self.root.join(path);
        if !full_path.exists() {
            return Err(ArklibError::Path(format!(
                "File does not exist: {:?}",
                full_path
            )));
        }
        let metadata = fs::metadata(&full_path)?;
        // return an error if the file is empty
        if metadata.len() == 0 {
            return Err(ArklibError::Path(format!(
                "File is empty: {:?}",
                full_path
            )));
        }
        let last_modified = metadata.modified()?;
        let id = Id::from_path(&full_path)?;

        let resource = IndexedResource {
            id: id.clone(),
            path: path.to_path_buf(),
            last_modified,
        };
        self.path_to_resource
            .insert(resource.path.clone(), resource.clone());
        self.id_to_resources
            .entry(id)
            .or_default()
            .push(resource.clone());

        Ok(resource)
    }

    /// Track the removal of a file from the resource index.
    ///
    /// This method checks if the file exists in the file system
    ///
    /// # Arguments
    /// * `relative_path` - The path of the file to be removed (relative to the
    ///   root path of the index).
    ///
    /// # Returns
    /// Returns `Ok(resource)` if the resource was successfully removed from the
    /// index.
    ///
    /// # Errors
    /// - If the file still exists in the file system.
    /// - If the resource does not exist in the index.
    pub fn track_removal<P: AsRef<Path>>(
        &mut self,
        relative_path: P,
    ) -> Result<IndexedResource<Id>> {
        log::debug!("Tracking removal of file: {:?}", relative_path.as_ref());

        let path = relative_path.as_ref();
        let full_path = self.root.join(path);
        if full_path.exists() {
            return Err(ArklibError::Path(format!(
                "File still exists: {:?}",
                full_path
            )));
        }

        // Remove the resource from the index
        let resource = self
            .path_to_resource
            .remove(path)
            .ok_or_else(|| anyhow!("Resource not found: {}", path.display()))?;

        // Remove the resource from the id_to_resources map
        if let Some(resources) = self.id_to_resources.get_mut(&resource.id) {
            resources.retain(|r| r.path != resource.path);
            if resources.is_empty() {
                self.id_to_resources.remove(&resource.id);
            }
        }

        Ok(resource)
    }

    /// Track the modification of a file in the resource index.
    ///
    /// This method checks if the file exists in the file system and removes the
    /// old resource from the index before adding the new resource to the
    /// index.
    ///
    /// # Arguments
    /// * `relative_path` - The relative path of the file to be modified.
    ///
    /// # Returns
    /// Returns `Ok(new_resource)` if the resource was successfully modified in
    /// the index.
    ///
    /// # Errors
    /// - If there was a problem removing the old resource from the index.
    /// - If there was a problem adding the new resource to the index.
    pub fn track_modification<P: AsRef<Path>>(
        &mut self,
        relative_path: P,
    ) -> Result<IndexedResource<Id>> {
        log::debug!(
            "Tracking modification of file: {:?}",
            relative_path.as_ref()
        );

        let path = relative_path.as_ref();
        // Remove the resource from the index
        let resource = self
            .path_to_resource
            .remove(path)
            .ok_or_else(|| anyhow!("Resource not found: {}", path.display()))?;

        // Remove the resource from the id_to_resources map
        if let Some(resources) = self.id_to_resources.get_mut(&resource.id) {
            resources.retain(|r| r.path != resource.path);
            if resources.is_empty() {
                self.id_to_resources.remove(&resource.id);
            }
        }

        // Add the new resource to the index
        let new_resource = self.track_addition(path)?;

        Ok(new_resource)
    }
}
