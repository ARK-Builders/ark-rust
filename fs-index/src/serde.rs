use std::{collections::HashMap, path::PathBuf, time::SystemTime};

use anyhow::Result;
use serde::{
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

use data_resource::ResourceId;

use crate::{index::IndexedResource, ResourceIndex};

/// Data structure for serializing and deserializing the index
#[derive(Serialize, Deserialize)]
struct ResourceIndexData<Id> {
    root: PathBuf,
    resources: HashMap<PathBuf, IndexedResourceData<Id>>,
}

#[derive(Serialize, Deserialize)]
struct IndexedResourceData<Id> {
    id: Id,
    last_modified: u64,
}

/// Custom implementation of [`Serialize`] for [`ResourceIndex`]
///
/// To avoid writing a large repetitive index file with double maps,
/// we are only serializing the root path, and path_to_resource.
///
/// Other fields can be reconstructed from the path_to_resource map.
impl<Id> Serialize for ResourceIndex<Id>
where
    Id: ResourceId,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ResourceIndex", 2)?;
        state.serialize_field("root", &self.root)?;

        let mut resources = HashMap::new();
        for (path, resource) in &self.path_to_resource {
            let id = resource.id().clone();
            let last_modified = resource
                .last_modified()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|e| {
                    serde::ser::Error::custom(format!(
                        "Failed to serialize last_modified: {}",
                        e
                    ))
                })?
                .as_nanos() as u64;

            let resource_data = IndexedResourceData { id, last_modified };
            resources.insert(path.clone(), resource_data);
        }

        state.serialize_field("resources", &resources)?;
        state.end()
    }
}

/// Custom implementation of [`Deserialize`] for [`ResourceIndex`]
///
/// Deserializes the index from the root path and path_to_resource map.
/// Other fields are reconstructed from the path_to_resource map.
impl<'de, Id> Deserialize<'de> for ResourceIndex<Id>
where
    Id: ResourceId,
{
    fn deserialize<D>(deserializer: D) -> Result<ResourceIndex<Id>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let index_data: ResourceIndexData<Id> =
            ResourceIndexData::deserialize(deserializer)?;

        let mut path_to_resource = HashMap::new();
        let mut id_to_resources = HashMap::new();
        for (path, resource_data) in index_data.resources {
            let last_modified = SystemTime::UNIX_EPOCH
                + std::time::Duration::from_nanos(resource_data.last_modified);
            let resource = IndexedResource::new(
                resource_data.id,
                path.clone(),
                last_modified,
            );
            path_to_resource.insert(path, resource.clone());
            id_to_resources
                .entry(resource.id().clone())
                .or_insert_with(Vec::new)
                .push(resource);
        }

        Ok(ResourceIndex {
            root: index_data.root,
            id_to_resources,
            path_to_resource,
        })
    }
}

/// Custom implementation of [`PartialEq`] for [`ResourceIndex`]
///
/// The order of items in hashmaps is not relevant.
/// we just need to compare [`ResourceIndex::resources`] to check if the two
/// indexes are equal.
impl<Id> PartialEq for ResourceIndex<Id>
where
    Id: ResourceId,
{
    fn eq(&self, other: &Self) -> bool {
        let mut resources1 = self.resources();
        let mut resources2 = other.resources();
        resources1.sort_by(|a, b| a.path().cmp(b.path()));
        resources2.sort_by(|a, b| a.path().cmp(b.path()));

        resources1 == resources2 && self.root == other.root
    }
}
