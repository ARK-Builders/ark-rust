# fs-index

The `fs-index` crate is part of the ARK framework, designed to help track resources in an index. This crate provides a robust system for managing a directory index, including tracking changes and querying resources.

## Features

The most important struct in this crate is `ResourceIndex` which comes with:

- **Reactive API**
  - `update_all`: Method to update the index by rescanning files and returning changes (additions/deletions/updates).
- **Snapshot API**
  - `get_resources_by_id`: Query resources from the index by ID.
  - `get_resource_by_path`: Query a resource from the index by its path.
- **Track API**
  - `track_addition`: Track a newly added file (checks if the file exists in the file system).
  - `track_removal`: Track the deletion of a file (checks if the file was actually deleted).
  - `track_modification`: Track an update on a single file.

## Custom Serialization

The `ResourceIndex` struct includes a custom serialization implementation to avoid writing a large repetitive index file with double maps.

## Tests and Benchmarks

- Unit tests are located in `src/tests.rs`.
- The benchmarking suite is in `benches/resource_index_benchmark.rs`, benchmarking all methods of `ResourceIndex`.
  - Run benchmarks with `cargo bench`.

## Examples

To get started, take a look at the examples in the `examples/` directory.

To run a specific example:

```shell
cargo run --example resource_index
```
