use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use tempfile::TempDir;

use data_resource::ResourceId;
use dev_hash::Crc32;

use super::*;
use crate::{index::IndexedResource, utils::load_or_build_index};

/// A helper function to get [`IndexedResource`] from a file path
fn get_indexed_resource_from_file<P: AsRef<Path>>(
    path: P,
    parent_dir: P,
) -> Result<IndexedResource<Crc32>> {
    let id = Crc32::from_path(&path)?;

    let relative_path = path
        .as_ref()
        .strip_prefix(parent_dir)
        .map_err(|_| anyhow!("Failed to get relative path"))?;

    Ok(IndexedResource::new(
        id,
        relative_path.to_path_buf(),
        fs::metadata(path)?.modified()?,
    ))
}

/// Test storing and loading the resource index.
///
/// ## Test scenario:
/// - Build a resource index in the temporary directory.
/// - Store the index.
/// - Load the stored index.
/// - Assert that the loaded index matches the original index.
#[test]
fn test_store_and_load_index() {
    let temp_dir = TempDir::with_prefix("ark_test_store_and_load_index")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    assert_eq!(index.len(), 1);
    index.store().expect("Failed to store index");

    let loaded_index =
        load_or_build_index(root_path, false).expect("Failed to load index");

    assert_eq!(index, loaded_index);
}

/// Test storing and loading the resource index with collisions.
///
/// ## Test scenario:
/// - Build a resource index in the temporary directory.
/// - Write duplicate files with the same content.
/// - Store the index.
/// - Load the stored index.
/// - Assert that the loaded index matches the original index.
#[test]
fn test_store_and_load_index_with_collisions() {
    let temp_dir =
        TempDir::with_prefix("ark_test_store_and_load_index_with_collisions")
            .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let file_path2 = root_path.join("file2.txt");
    fs::write(&file_path2, "file content").expect("Failed to write to file");

    let file_path3 = root_path.join("file3.txt");
    fs::write(&file_path3, "file content").expect("Failed to write to file");

    let file_path4 = root_path.join("file4.txt");
    fs::write(&file_path4, "file content").expect("Failed to write to file");

    // Now we have 4 files with the same content (same checksum)

    let index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    let checksum =
        Crc32::from_path(&file_path).expect("Failed to get checksum");
    assert_eq!(index.len(), 4);
    assert_eq!(index.collisions().len(), 1);
    assert_eq!(index.collisions()[&checksum].len(), 4);
    index.store().expect("Failed to store index");

    let loaded_index =
        load_or_build_index(root_path, false).expect("Failed to load index");

    assert_eq!(index, loaded_index);
}

/// Test building an index with a file.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains one entry.
/// - Assert that the resource retrieved by path matches the expected resource.
/// - Assert that the resource retrieved by ID matches the expected resource.
#[test]
fn test_build_index_with_file() {
    let temp_dir = TempDir::with_prefix("ark_test_build_index_with_file")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");
    let expected_resource =
        get_indexed_resource_from_file(&file_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");

    let index = ResourceIndex::build(root_path).expect("Failed to build index");
    assert_eq!(index.len(), 1);

    let resource = index
        .get_resource_by_path("file.txt")
        .expect("Failed to get resource");
    assert_eq!(resource, &expected_resource);
}

/// Test building an index with an empty file.
///
/// ## Test scenario:
/// - Create an empty file within the temporary directory.
/// - Create a file with content within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains one entries.
#[test]
fn test_build_index_with_empty_file() {
    let temp_dir = TempDir::with_prefix("ark_test_build_index_with_empty_file")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let empty_file_path = root_path.join("empty_file.txt");
    fs::write(&empty_file_path, "").expect("Failed to write to file");

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    assert_eq!(index.len(), 1);
}

/// Test building an index with a directory.
///
/// ## Test scenario:
/// - Create a subdirectory within the temporary directory.
/// - Create a file within the subdirectory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains one entry.
/// - Assert that the resource retrieved by path matches the expected resource.
/// - Assert that the resource retrieved by ID matches the expected resource.
#[test]
fn test_build_index_with_directory() {
    let temp_dir = TempDir::with_prefix("ark_test_build_index_with_directory")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let dir_path = root_path.join("dir");
    fs::create_dir(&dir_path).expect("Failed to create dir");
    let file_path = dir_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");
    let expected_resource =
        get_indexed_resource_from_file(&file_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");

    let index = ResourceIndex::build(root_path).expect("Failed to build index");
    assert_eq!(index.len(), 1);

    let resource = index
        .get_resource_by_path("dir/file.txt")
        .expect("Failed to get resource");
    assert_eq!(resource, &expected_resource);
}

/// Test building an index with multiple files.
///
/// ## Test scenario:
/// - Create multiple files within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains two entries.
/// - Assert that the resource retrieved by path for each file matches the
///   expected resource.
#[test]
fn test_build_index_with_multiple_files() {
    let temp_dir =
        TempDir::with_prefix("ark_test_build_index_with_multiple_files")
            .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file1_path = root_path.join("file1.txt");
    fs::write(&file1_path, "file1 content").expect("Failed to write to file");
    let file2_path = root_path.join("file2.txt");
    fs::write(&file2_path, "file2 content").expect("Failed to write to file");

    let expected_resource1 =
        get_indexed_resource_from_file(&file1_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");
    let expected_resource2 =
        get_indexed_resource_from_file(&file2_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");

    let index = ResourceIndex::build(root_path).expect("Failed to build index");
    assert_eq!(index.len(), 2);

    let resource = index
        .get_resource_by_path("file1.txt")
        .expect("Failed to get resource");
    assert_eq!(resource, &expected_resource1);

    let resource = index
        .get_resource_by_path("file2.txt")
        .expect("Failed to get resource");
    assert_eq!(resource, &expected_resource2);
}

/// Test building an index with multiple directories.
///
/// ## Test scenario:
/// - Create multiple directories within the temporary directory, each
///   containing a file.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains two entries.
/// - Assert that the resources retrieved by path for each file match the
///   expected resources.
#[test]
fn test_build_index_with_multiple_directories() {
    let temp_dir =
        TempDir::with_prefix("ark_test_build_index_with_multiple_directories")
            .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let dir1_path = root_path.join("dir1");
    fs::create_dir(&dir1_path).expect("Failed to create dir");
    let file1_path = dir1_path.join("file1.txt");
    fs::write(&file1_path, "file1 content").expect("Failed to write to file");

    let dir2_path = root_path.join("dir2");
    fs::create_dir(&dir2_path).expect("Failed to create dir");
    let file2_path = dir2_path.join("file2.txt");
    fs::write(&file2_path, "file2 content").expect("Failed to write to file");

    let expected_resource1 =
        get_indexed_resource_from_file(&file1_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");
    let expected_resource2 =
        get_indexed_resource_from_file(&file2_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");

    let index = ResourceIndex::build(root_path).expect("Failed to build index");
    assert_eq!(index.len(), 2);

    let resource = index
        .get_resource_by_path("dir1/file1.txt")
        .expect("Resource not found");
    assert_eq!(resource, &expected_resource1);

    let resource = index
        .get_resource_by_path("dir2/file2.txt")
        .expect("Resource not found");
    assert_eq!(resource, &expected_resource2);
}

/// Test tracking the removal of a file from the index.
///
/// ## Test scenario:
/// - Create two files within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains two entries.
/// - Remove one of the files.
/// - Track the removal of the file in the index.
/// - Assert that the index contains only one entry after removal.
/// - Assert that the removed file is no longer present in the index, while the
///   other file remains.
#[test]
fn test_track_removal() {
    let temp_dir = TempDir::with_prefix("ark_test_track_removal")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");
    let image_path = root_path.join("image.png");
    fs::write(&image_path, "image content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 2);

    fs::remove_file(&file_path).expect("Failed to remove file");

    let file_relative_path = file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    index
        .track_removal(&file_relative_path)
        .expect("Failed to track removal");

    assert_eq!(index.len(), 1);
    assert!(index.get_resource_by_path("file.txt").is_none());
    assert!(index.get_resource_by_path("image.png").is_some());
}

/// Test tracking the removal of a file that doesn't exist.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index contains only one entry.
/// - Track the removal of a file that doesn't exist in the index.
/// - Assert that the index still contains only one entry.
#[test]
fn test_track_removal_non_existent() {
    let temp_dir = TempDir::with_prefix("ark_test_track_removal_non_existent")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");

    let new_file_relative_path = new_file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    let removal_result = index.track_removal(&new_file_relative_path);
    assert!(removal_result.is_err());
    assert_eq!(index.len(), 1);
}

/// Test tracking the addition of a new file to the index.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains only one entry.
/// - Create a new file in the temporary directory.
/// - Track the addition of the new file in the index.
/// - Assert that the index contains two entries after addition.
/// - Assert that both files are present in the index.
#[test]
fn test_track_addition() {
    let temp_dir = TempDir::with_prefix("ark_test_track_addition")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");
    fs::write(&new_file_path, "new file content")
        .expect("Failed to write to file");

    let new_file_relative_path = new_file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    index
        .track_addition(&new_file_relative_path)
        .expect("Failed to track addition");

    assert_eq!(index.len(), 2);
    assert!(index.get_resource_by_path("file.txt").is_some());
    assert!(index
        .get_resource_by_path("new_file.txt")
        .is_some());
}

/// Test tracking the addition of an empty file to the index.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains only one entry.
/// - Create a new empty file in the temporary directory.
/// - Track the addition of the new file in the index.
/// - Assert that it retuns an error.
#[test]
fn test_track_addition_empty_file() {
    let temp_dir = TempDir::with_prefix("ark_test_track_addition_empty_file")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");
    fs::write(&new_file_path, "").expect("Failed to write to file");

    let new_file_relative_path = new_file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    let addition_result = index.track_addition(&new_file_relative_path);
    assert!(addition_result.is_err());
}

/// Test for tracking addition of a file that doesn't exist
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains only one entry.
/// - Track the addition of a file that doesn't exist in the index.
/// - Assert that the index still contains only one entry.
#[test]
fn test_track_addition_non_existent() {
    let temp_dir = TempDir::with_prefix("ark_test_track_addition_non_existent")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");

    let new_file_relative_path = new_file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    let addition_result = index.track_addition(&new_file_relative_path);
    assert!(addition_result.is_err());
    assert_eq!(index.len(), 1);
}

/// Test tracking the modification of a file in the index.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains only one entry.
/// - Update the content of the file.
/// - Track the modification of the file in the index.
/// - Assert that the index still contains only one entry.
/// - Assert that the modification timestamp of the file in the index matches
///   the actual file's modification timestamp.
#[test]
fn test_track_modification() {
    let temp_dir = TempDir::with_prefix("ark_test_track_modification")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    fs::write(&file_path, "updated file content")
        .expect("Failed to write to file");

    let file_relative_path = file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    index
        .track_modification(&file_relative_path)
        .expect("Failed to track modification");

    assert_eq!(index.len(), 1);
    let resource = index
        .get_resource_by_path("file.txt")
        .expect("Resource not found");
    assert_eq!(
        resource.last_modified(),
        fs::metadata(&file_path)
            .unwrap()
            .modified()
            .unwrap()
    );
}

/// Test that track modification does not add a new file to the index.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains only one entry.
/// - Create a new file in the temporary directory.
/// - Track the modification of the new file in the index.
/// - Assert that the index still contains only one entry.
#[test]
fn test_track_modification_does_not_add() {
    let temp_dir =
        TempDir::with_prefix("ark_test_track_modification_does_not_add")
            .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");
    fs::write(&new_file_path, "new file content")
        .expect("Failed to write to file");

    let new_file_relative_path = new_file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");

    let modification_result = index.track_modification(&new_file_relative_path);
    assert!(modification_result.is_err());
}

/// Test updating the resource index.
///
/// ## Test scenario:
/// - Create files within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains the expected number of entries.
/// - Create a new file, modify an existing file, and remove another file.
/// - Update the resource index.
/// - Assert that the index contains the expected number of entries after the
///   update.
/// - Assert that the entries in the index match the expected state after the
///   update.
#[test]
fn test_resource_index_update() {
    let temp_dir = TempDir::with_prefix("ark_test_resource_index_update")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let image_path = root_path.join("image.png");
    fs::write(&image_path, "image content").expect("Failed to write to file");

    let mut index =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 2);

    // create new file
    let new_file_path = root_path.join("new_file.txt");
    fs::write(&new_file_path, "new file content")
        .expect("Failed to write to file");

    // modify file
    fs::write(&file_path, "updated file content")
        .expect("Failed to write to file");

    // remove file
    fs::remove_file(&image_path).expect("Failed to remove file");

    index
        .update_all()
        .expect("Failed to update index");
    // Index now contains 2 resources (file.txt and new_file.txt)
    assert_eq!(index.len(), 2);

    let resource = index
        .get_resource_by_path("file.txt")
        .expect("Resource not found");
    let expected_resource =
        get_indexed_resource_from_file(&file_path, &root_path.to_path_buf())
            .expect("Failed to get indexed resource");
    assert_eq!(resource, &expected_resource);

    let _resource = index
        .get_resource_by_path("new_file.txt")
        .expect("Resource not found");

    assert!(index.get_resource_by_path("image.png").is_none());
}

/// Test adding colliding files to the index.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains the expected number of entries.
/// - Create a new file with the same checksum as the existing file.
/// - Track the addition of the new file in the index.
/// - Assert that the index contains the expected number of entries after the
///   addition.
/// - Assert index.collisions contains the expected number of entries.
#[test]
fn test_add_colliding_files() {
    let temp_dir = TempDir::with_prefix("ark_test_add_colliding_files")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");
    fs::write(&new_file_path, "file content").expect("Failed to write to file");

    let new_file_relative_path = new_file_path
        .strip_prefix(root_path)
        .expect("Failed to get relative path");
    index
        .track_addition(&new_file_relative_path)
        .expect("Failed to track addition");

    assert_eq!(index.len(), 2);
    assert_eq!(index.collisions().len(), 1);
}

/// Test `ResourceIndex::num_collisions()` method.
///
/// ## Test scenario:
/// - Create a file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains the expected number of entries.
/// - Create 2 new files with the same checksum as the existing file.
/// - Update the index.
/// - Assert that the index contains the expected number of entries after the
///   update.
#[test]
fn test_num_collisions() {
    let temp_dir = TempDir::with_prefix("ark_test_num_collisions")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join("file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let mut index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 1);

    let new_file_path = root_path.join("new_file.txt");
    fs::write(&new_file_path, "file content").expect("Failed to write to file");

    let new_file_path2 = root_path.join("new_file2.txt");
    fs::write(&new_file_path2, "file content")
        .expect("Failed to write to file");

    index
        .update_all()
        .expect("Failed to update index");

    assert_eq!(index.len(), 3);
    assert_eq!(index.num_collisions(), 3);
}

/// Test that we don't index hidden files.
///
/// ## Test scenario:
/// - Create a hidden file within the temporary directory.
/// - Build a resource index in the temporary directory.
/// - Assert that the index initially contains the expected number of entries.
///   (0)
#[test]
fn test_hidden_files() {
    let temp_dir = TempDir::with_prefix("ark_test_hidden_files")
        .expect("Failed to create temp dir");
    let root_path = temp_dir.path();

    let file_path = root_path.join(".hidden_file.txt");
    fs::write(&file_path, "file content").expect("Failed to write to file");

    let index: ResourceIndex<Crc32> =
        ResourceIndex::build(root_path).expect("Failed to build index");
    index.store().expect("Failed to store index");
    assert_eq!(index.len(), 0);
}
