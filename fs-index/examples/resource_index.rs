use std::path::Path;

use anyhow::Result;

use dev_hash::Blake3;
use fs_index::ResourceIndex;

/// A simple example of how to use [`ResourceIndex`] to index a directory.
fn main() -> Result<()> {
    // Create a new `ResourceIndex` from the directory "test-assets" using
    // blake3 as the hashing algorithm.
    let mut index: ResourceIndex<Blake3> =
        ResourceIndex::build(Path::new("test-assets"))?;

    // Print the indexed resources.
    for resource in index.resources() {
        println!("{:?}", resource);
    }

    // Save the index to a file.
    index.store()?;

    // Get resources by their id.
    let id = Blake3(
        "172b4bf148e858b13dde0fc6613413bcb7552e5c4e5c45195ac6c80f20eb5ff5"
            .to_string(),
    );
    let resources = index.get_resources_by_id(&id).ok_or_else(|| {
        anyhow::anyhow!("Resource with id {:?} not found", id)
    })?;
    for resource in resources {
        println!("{:?}", resource);
    }

    // Get resources by their path.
    let path = Path::new("lena.jpg");
    let resource = index.get_resource_by_path(path).ok_or_else(|| {
        anyhow::anyhow!("Resource with path {:?} not found", path)
    })?;
    println!("{:?}", resource);

    // Update the index.
    index.update_all()?;

    Ok(())
}
