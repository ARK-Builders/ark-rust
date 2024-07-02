use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data_resource::ResourceId;
use rand::prelude::*;
use std::path::Path;

use dev_hash::Blake3;

// Add files to benchmark here
const FILE_PATHS: [&str; 2] =
    ["../test-assets/lena.jpg", "../test-assets/test.pdf"];
// Modify time limit here
const BENCHMARK_TIME_LIMIT: std::time::Duration =
    std::time::Duration::from_secs(20);

fn generate_random_data(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

/// Benchmarks the performance of resource ID creation from file paths and
/// random data.
///
/// - Measures the time taken to create a resource ID from file paths.
/// - Measures the time taken to create a resource ID from random data.
fn bench_resource_id_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_resource_id_creation");
    group.measurement_time(BENCHMARK_TIME_LIMIT);

    // Benchmarks for computing from file paths
    for file_path in FILE_PATHS.iter() {
        assert!(
            Path::new(file_path).is_file(),
            "The file: {} does not exist or is not a file",
            file_path
        );

        let id = format!("compute_from_path:{}", file_path);
        group.bench_function(id, move |b| {
            b.iter(|| {
                <Blake3 as ResourceId>::from_path(black_box(file_path))
                    .expect("from_path returned an error")
            });
        });
    }

    // Benchmarks for computing from random data
    let inputs = [("small", 1024), ("medium", 65536), ("large", 1048576)];

    for (name, size) in inputs.iter() {
        let input_data = generate_random_data(*size);

        let id = format!("compute_from_bytes:{}", name);
        group.bench_function(id, move |b| {
            b.iter(|| {
                <Blake3 as ResourceId>::from_bytes(black_box(&input_data))
                    .expect("from_bytes returned an error")
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_resource_id_creation);
criterion_main!(benches);
