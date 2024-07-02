# ARK

<div align="center">
  <img src="logo.svg" alt="ARK Logo" width="75%" />

[![License](https://img.shields.io/github/license/ARK-Builders/ark-rust.svg)](LICENSE)
[![Build Status](https://github.com/ARK-Builders/ark-rust/actions/workflows/build.yml/badge.svg)](https://github.com/ARK-Builders/ark-rust/actions/workflows/build.yml)

**The home of the ARK framework**

</div>

Being implemented in Rust, it provides us capability to port any apps using ARK to all common platforms. Right now, Android is supported by using the [ark-android](https://github.com/ARK-Builders/ark-android) project. And for Linux/macOS/Windows, the framework can be used as-is and easily embedded into an app, e.g. built with [Tauri](https://tauri.app/).

Development docs will come sometime.

**The Concept of the Framework**

The framework is supposed to help in solving the following problems:

1. Management of user data, stored as simple files on the disk, as well as various kinds of metadata: tags, scores, arbitrary properties like movie title or description. Such a metadata is persisted to filesystem for easier sync, backup and migration by any 3rd-party tool.
2. Sync of both user data and metadata, across all user devices in P2P fashion. Cache syncing might be implemented later, too.
3. Version tracking for user data: not only text-files, but also images, videos etc.

The core crate is `fs-index` which provides us with [content addressing](https://en.wikipedia.org/wiki/Content-addressable_storage) and allows easier storage and versions tracking. Another important crate is `fs-storage` which has the purpose of storing and transactional access to different kinds of metadata in the hidden `.ark` folder. Don't miss the `ark-cli` which is a Swiss Army Knife for flexible usage in scripts consuming data from ARK-enabled folders.

## Crates

<div align="center">

| Package         | Description                              |
| --------------- | ---------------------------------------- |
| `ark-cli`       | The CLI tool to interact with ark crates |
| `data-resource` | Resource hashing and ID construction     |
| `fs-index`      | Resource Index construction and updating |
| `fs-storage`    | Filesystem storage for resources         |
| `fs-metadata`   | Metadata management                      |
| `fs-properties` | Properties management                    |
| `data-link`     | Linking resources                        |
| `data-pdf`      | PDF handling                             |
| `data-error`    | Error handling                           |
| `data-json`     | JSON serialization and deserialization   |

</div>

## Build

Like most of Rust projects:

```bash
cargo build --release
```

Run unit tests:

```bash
cargo test
```

## Development

For easier testing and debugging, we have the [ARK-CLI](https://github.com/ARK-Builders/ARK-CLI) tool working with ARK-enabled folders.

## Benchmarks

`fs-index` relies on the `criterion` crate for benchmarking to ensure optimal performance. Benchmarks are crucial for evaluating the efficiency of various functionalities within the library.

### Running Benchmarks

To execute the benchmarks, run this command:

```bash
cargo bench
```

This command runs all benchmarks and generates a report in HTML format located at `target/criterion/report`. If you wish to run a specific benchmark, you can specify its name as an argument as in:

```bash
cargo bench resource_index
```

### Benchmarking Local Files

Our benchmark suite includes tests on local files and directories. These benchmarks are located in the `benches/` directory of some crates. Each benchmark sets a time limit using `group.measurement_time()`, which you can adjust manually based on your requirements.

You have the flexibility to benchmark specific files or folders by modifying the variables within the benchmark files. By default, the benchmarks operate on the [`test-assets/`](test-assets/) directory and its contents. You can change the directory/files by setting the `DIR_PATH` and `FILE_PATHS` variables to the desired values.

For pre-benchmark assessment of required time to index a huge local folder, you can modify `test_build_resource_index` test case in `src/index.rs`.

### Generating Profiles for Benchmarks

[flamegraph](https://github.com/flamegraph-rs/flamegraph) is a Cargo command that uses perf/DTrace to profile your code and then displays the results in a flame graph. It works on Linux and all platforms that support DTrace (macOS, FreeBSD, NetBSD, and possibly Windows).

To install `flamegraph`, run:

```bash
cargo install flamegraph
```

To generate a flame graph for `resource_index_benchmark`, use the following command:

```bash
cargo flamegraph --bench resource_index_benchmark -o resource_index_benchmark.svg -- --bench
```

> [!NOTE]
> Running `cargo flamegraph` on macOS requires `DTrace`. MacOS System Integrity Protection (`SIP`) is enabled by default, which restricts most uses of `dtrace`.
>
> To work around this issue, you have two options:
>
> - Boot into recovery mode and disable some SIP protections.
> - Run as superuser to enable DTrace. This can be achieved by using `cargo flamegraph --root ...`.
>
> For further details, please refer to https://github.com/flamegraph-rs/flamegraph?tab=readme-ov-file#dtrace-on-macos

## Bindings

`ark` includes support for Java bindings using the [`jni-rs`](https://github.com/jni-rs/jni-rs) crate, which uses the Java Native Interface (JNI) to allow Rust functions to be called from Java. The Java bindings are implemented as a Gradle project, located in the `java/` directory.
