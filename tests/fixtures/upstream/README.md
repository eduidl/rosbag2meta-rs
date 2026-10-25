# Official rosbag2 fixtures

Source: [ros2/rosbag2](https://github.com/ros2/rosbag2), retrieved on 2026-10-26.
Each YAML file starts with comments identifying its source URL pinned to a commit
and its license. Legacy QoS strings are formatted as YAML block scalars for
readability; their parsed values are unchanged from upstream. All other body
content is preserved. SHA-256 checksums below cover the local files including
the formatting changes and added comments.
The upstream license is Apache-2.0; see the included [LICENSE](LICENSE).

The official v2 fixture is not included because no YAML file for that version
was found in the sources examined. Synthetic cases in `tests/parse.rs` continue
to cover all versions from v1 through v9. The v8 fixture comes from a historical
path that was later removed. Tests using the official fixtures require neither
network access nor a ROS installation.

- [metadata-v1.yaml](metadata-v1.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/c5c7f1879be901ad900c4ccb48974b5c47567a43/rosbag2_tests/resources/cdr_test/metadata.yaml)
  - SHA-256: `e644830da380120fa8b6f125b8d94ece34b150a217b7c78b88af8f0bff80a054`
- [metadata-v3.yaml](metadata-v3.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/e02bc77a3d95e2248adb4de4f676c135d102a312/rosbag2_tests/resources/cdr_test/metadata.yaml)
  - SHA-256: `c25ecf6dd5dd6bea4a1562ee99f7573c46c6921aa498e7b428078ef2e350fd1a`
- [metadata-v4.yaml](metadata-v4.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/910c07f37e483e16f94e593e0a1609919f718286/rosbag2_py/test/resources/sqlite3/talker/metadata.yaml)
  - SHA-256: `04182a92d95431a0f0182bf8d261cbcfa90953950c646e6f244e028c809977bb`
- [metadata-v5.yaml](metadata-v5.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/910c07f37e483e16f94e593e0a1609919f718286/rosbag2_py/test/resources/sqlite3/convert_a/metadata.yaml)
  - SHA-256: `1fa8b08470a755ba153566850709721b390b4f78c91184548be4e158566c7784`
- [metadata-v6.yaml](metadata-v6.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/910c07f37e483e16f94e593e0a1609919f718286/rosbag2_transport/test/resources/mcap/rewriter_b/metadata.yaml)
  - SHA-256: `5a200d1f408835470a317e09e6f2c611c47ee44e798469fdc8c10b53d79514d2`
- [metadata-v7.yaml](metadata-v7.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/910c07f37e483e16f94e593e0a1609919f718286/rosbag2_tests/resources/mcap/cdr_test/metadata.yaml)
  - SHA-256: `450acca8fc90e89c277d08804bc4904abbcafb3cb5525082af063e83faab9fe3`
- [metadata-v8.yaml](metadata-v8.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/3286736d364b2794051020ae8f2b0893521da16f/rosbag2_tests/resources/sqlite3/bag_with_topics_and_service_events/metadata.yaml)
  - SHA-256: `e80a7933a3d62236eef6f683c250925f36fee2e20990136d6e077dd00f275230`
- [metadata-v9.yaml](metadata-v9.yaml) — [Original upstream file](https://github.com/ros2/rosbag2/blob/910c07f37e483e16f94e593e0a1609919f718286/rosbag2_tests/resources/sqlite3/bag_with_topics_and_service_events_and_action/metadata.yaml)
  - SHA-256: `1b3bf1dcf913d57416c33f45525b04c0cb18598c75e9f83d59f605e637d2a2c4`
