# rosbag2meta-rs

A Rust library for reading rosbag2 `metadata.yaml` files without a ROS installation.

```rust,no_run
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metadata = rosbag2meta_rs::from_reader(File::open("metadata.yaml")?)?;
    println!("{} messages", metadata.message_count);
    // Apply the different path bases used before and after version 4.
    let paths = metadata.resolved_file_paths("/data/recording");
    println!("{paths:?}");
    for topic in metadata.topics_with_message_count {
        println!("{}: {}", topic.topic_metadata.name, topic.message_count);
    }
    Ok(())
}
```

Use `rosbag2meta_rs::from_str(&yaml)` to parse a string. Both entry points
expect a YAML document rooted at `rosbag2_bagfile_information` and return
`Result<BagMetadata, rosbag2meta_rs::Error>`.

## Parsed information

- Storage identifier, relative file paths, duration, start time, and message count
- Topic names, types, serialization formats, type description hashes, and message counts
- Compression settings, per-file information, custom data, and ROS distribution
- QoS profiles preserved as `OfferedQosProfiles`: strings through version 8 and sequences in version 9

Start times use `i64`; durations and message counts use `u64`, preserving
nanosecond precision. Each QoS profile in a sequence is a `BTreeMap<String, Value>`.
Legacy strings are also parsed internally for format validation, but the returned
metadata retains the original string.

## Compatibility and validation

Metadata versions 1–9 are accepted; other versions return an error.
Core fields and fields introduced by the declared version are required.
Empty strings, sequences, and maps are allowed. `custom_data: null` is treated
as an empty map. Fields introduced after the declared version are ignored,
matching the official reader: returned strings, sequences, and maps default
to empty values, and QoS defaults to `None`. Unknown keys are also ignored.

Version changes are cumulative:

- v1: Initial format.
- v2: The official C++ implementation changed integer handling for counts and
  times from `size_t` to `uint64_t`. YAML keys are unchanged; this library uses
  64-bit integers starting with v1.
- v3: Requires `compression_format` and `compression_mode`.
- v4: Requires each topic's `offered_qos_profiles`, stored as a string containing
  YAML. The base for `relative_file_paths` changes from the bag directory's
  parent to the bag directory itself.
- v5: Requires `files`.
- v6: Requires `custom_data`.
- v7: Requires each topic's `type_description_hash`.
- v8: Requires `ros_distro`.
- v9: Changes `offered_qos_profiles` to a direct sequence. The values of `history`,
  `reliability`, `durability`, and `liveliness` change from integers to strings.

QoS validation checks the container format and, when present, the integer or
string representation of the four policies above. It does not require every
policy field, validate policy values or DDS semantics, or normalize infinite
durations. Message count totals and file existence are not checked.
Reading SQLite3/MCAP contents, writing YAML, and a CLI are outside the scope.

Pass the directory containing `metadata.yaml` to
`BagMetadata::resolved_file_paths(bag_directory)`. It returns paths using the
version's base directory without changing the stored strings. Absolute paths
are preserved. This method does not access files or enforce directory containment.

## References and fixtures

The parser follows the official public types and YAML conversion implementation
(reviewed on 2026-10-26).

- [BagMetadata definition](https://github.com/ros2/rosbag2/blob/rolling/rosbag2_storage/include/rosbag2_storage/bag_metadata.hpp)
- [YAML fields and version handling](https://github.com/ros2/rosbag2/blob/rolling/rosbag2_storage/include/rosbag2_storage/yaml.hpp)
- [metadata.yaml root structure](https://github.com/ros2/rosbag2/blob/rolling/rosbag2_storage/src/rosbag2_storage/metadata_io.cpp)
- [v2 integer width change](https://github.com/ros2/rosbag2/commit/4bb9d892154c15fc2670891178ede003f82c6a46)
- [QoS version differences](https://github.com/ros2/rosbag2/blob/rolling/rosbag2_storage/src/rosbag2_storage/qos.cpp)
- [File path bases](https://github.com/ros2/rosbag2/blob/rolling/rosbag2_cpp/src/rosbag2_cpp/readers/sequential_reader.cpp)

Official test YAML files for v1, v3, v4, v5, v6, v7, v8, and v9 are included in
[`tests/fixtures/upstream`](tests/fixtures/upstream/README.md), and tested for parsing. Legacy QoS strings use YAML block scalars for readability
while preserving their parsed values; other body content is unchanged.
Each YAML file starts with comments identifying
its source URL pinned to a commit and its license. The provenance document also
records SHA-256 checksums.

An official v2 YAML file was not found in the sources examined; that version
is tested with synthetic cases. The cases generated in `tests/parse.rs` are
also synthetic.

## Development

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```
