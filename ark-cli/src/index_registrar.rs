use lazy_static::lazy_static;
extern crate canonical_path;

use data_error::{ArklibError, Result};
use fs_index::{load_or_build_index, ResourceIndex};

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::ResourceId;

use canonical_path::CanonicalPathBuf;

pub type ResourceIndexLock = Arc<RwLock<ResourceIndex<ResourceId>>>;

lazy_static! {
    pub static ref REGISTRAR: RwLock<HashMap<CanonicalPathBuf, ResourceIndexLock>> =
        RwLock::new(HashMap::new());
}

pub fn provide_index<P: AsRef<Path>>(
    root_path: P,
) -> Result<Arc<RwLock<ResourceIndex<ResourceId>>>> {
    let root_path = CanonicalPathBuf::canonicalize(root_path)?;

    {
        let registrar = REGISTRAR.read().map_err(|_| ArklibError::Parse)?;

        if let Some(index) = registrar.get(&root_path) {
            log::info!("Index has been registered before");
            return Ok(index.clone());
        }
    }

    log::info!("Index has not been registered before");
    // If the index has not been registered before, we need to load it, update
    // it and register it
    match load_or_build_index(&root_path, true) {
        Ok(index) => {
            let mut registrar = REGISTRAR.write().map_err(|_| {
                ArklibError::Other(anyhow::anyhow!("Failed to lock registrar"))
            })?;
            let arc = Arc::new(RwLock::new(index));
            registrar.insert(root_path, arc.clone());

            log::info!("Index was registered");
            Ok(arc)
        }
        Err(e) => Err(e),
    }
}
