use rosbag2meta_rs::{BagMetadata, OfferedQosProfiles, Value, from_reader, from_str};

const CURRENT: &str = include_str!("fixtures/upstream/metadata-v9.yaml");
const EARLY: &str = r#"
rosbag2_bagfile_information:
  version: 1
  storage_identifier: sqlite3
  relative_file_paths: [recording/recording_0.db3]
  duration: {nanoseconds: 100}
  starting_time: {nanoseconds_since_epoch: 1700000000123456789}
  message_count: 1
  topics_with_message_count:
    - topic_metadata:
        name: /chatter
        type: std_msgs/msg/String
        serialization_format: cdr
      message_count: 1
"#;

fn metadata_yaml(version: u32) -> String {
    let mut yaml = EARLY.replace("version: 1", &format!("version: {version}"));
    if version >= 3 {
        yaml.push_str("  compression_format: ''\n  compression_mode: ''\n");
    }
    if version >= 4 {
        yaml = yaml.replace("[recording/recording_0.db3]", "[recording_0.db3]");
        let qos = if version == 9 { "[]" } else { "''" };
        yaml = yaml.replace(
            "serialization_format: cdr",
            &format!("serialization_format: cdr\n        offered_qos_profiles: {qos}"),
        );
    }
    if version >= 5 {
        yaml.push_str("  files: []\n");
    }
    if version >= 6 {
        yaml.push_str("  custom_data: {}\n");
    }
    if version >= 7 {
        yaml = yaml.replace(
            "serialization_format: cdr",
            "serialization_format: cdr\n        type_description_hash: ''",
        );
    }
    if version >= 8 {
        yaml.push_str("  ros_distro: jazzy\n");
    }
    yaml
}

#[test]
fn parses_current_metadata_from_string_and_reader() {
    let metadata = from_str(CURRENT).unwrap();
    assert_eq!(from_reader(CURRENT.as_bytes()).unwrap(), metadata);
    assert_eq!(metadata.version, 9);
    assert_eq!(metadata.storage_identifier, "sqlite3");
    assert_eq!(metadata.duration.nanoseconds, 52_559_897_611);
    assert_eq!(
        metadata.starting_time.nanoseconds_since_epoch,
        1_741_949_479_681_027_429
    );
    assert_eq!(metadata.message_count, 187);
    assert_eq!(metadata.files[0].path, metadata.relative_file_paths[0]);
    assert_eq!(metadata.files[0].message_count, 187);
    assert!(metadata.custom_data.is_empty());
    assert_eq!(metadata.ros_distro, "rolling");
    let topic = &metadata.topics_with_message_count[0];
    assert_eq!(topic.message_count, 27);
    assert_eq!(topic.topic_metadata.topic_type, "std_msgs/msg/String");
    let Some(OfferedQosProfiles::Profiles(profiles)) = &topic.topic_metadata.offered_qos_profiles
    else {
        panic!("expected structured QoS profiles");
    };
    assert_eq!(profiles[0]["depth"].as_u64(), Some(0));
    assert_eq!(profiles[0]["reliability"].as_str(), Some("reliable"));
}

#[test]
fn parses_all_versions_with_their_required_fields() {
    for version in 1..=9 {
        let yaml = metadata_yaml(version);
        let metadata = from_str(&yaml).unwrap();
        assert_eq!(from_reader(yaml.as_bytes()).unwrap(), metadata);
        let document: Value = serde_yaml_ng::from_str(&yaml).unwrap();
        assert_eq!(
            serde_yaml_ng::from_value::<BagMetadata>(
                document["rosbag2_bagfile_information"].clone()
            )
            .unwrap(),
            metadata
        );
        assert_eq!(metadata.version, version);
        assert!(metadata.files.is_empty());
        assert!(metadata.custom_data.is_empty());
        assert_eq!(metadata.ros_distro, if version >= 8 { "jazzy" } else { "" });
        assert!(metadata.compression_mode.is_empty());
        assert_eq!(
            metadata.topics_with_message_count[0]
                .topic_metadata
                .offered_qos_profiles
                .is_none(),
            version < 4
        );
    }
}

#[test]
fn preserves_legacy_qos_string() {
    let yaml = metadata_yaml(5).replace(
        "offered_qos_profiles: ''",
        "offered_qos_profiles: |-\n          - history: 1\n            depth: 10",
    );
    let metadata = from_str(&yaml).unwrap();
    let escaped = metadata_yaml(5).replace(
        "offered_qos_profiles: ''",
        "offered_qos_profiles: \"- history: 1\\n  depth: 10\"",
    );
    assert_eq!(from_str(&escaped).unwrap(), metadata);
    assert_eq!(
        metadata.topics_with_message_count[0]
            .topic_metadata
            .offered_qos_profiles,
        Some(OfferedQosProfiles::Legacy(
            "- history: 1\n  depth: 10".into()
        ))
    );
}

#[test]
fn accepts_empty_qos_in_both_representations() {
    for (version, expected) in [
        (4, OfferedQosProfiles::Legacy(String::new())),
        (9, OfferedQosProfiles::Profiles(Vec::new())),
    ] {
        let yaml = metadata_yaml(version);
        let metadata = from_str(&yaml).unwrap();
        assert_eq!(
            metadata.topics_with_message_count[0]
                .topic_metadata
                .offered_qos_profiles,
            Some(expected)
        );
    }
}

#[test]
fn accepts_empty_metadata_collections_and_null_custom_data() {
    let yaml = CURRENT.replace("custom_data: ~", "custom_data:\n    # empty map");
    assert!(from_str(&yaml).unwrap().custom_data.is_empty());
    let yaml = r#"
rosbag2_bagfile_information:
  version: 9
  storage_identifier: sqlite3
  relative_file_paths: []
  duration: {nanoseconds: 0}
  starting_time: {nanoseconds_since_epoch: 0}
  message_count: 0
  topics_with_message_count: []
  files: []
  custom_data: {}
  compression_format: ''
  compression_mode: ''
  ros_distro: jazzy
"#;
    let metadata = from_str(yaml).unwrap();
    assert!(metadata.topics_with_message_count.is_empty());
    assert_eq!(metadata.message_count, 0);
}

#[test]
fn parses_custom_data() {
    let yaml = CURRENT.replace("custom_data: ~", "custom_data:\n    operator: robot-1");
    assert_eq!(from_str(&yaml).unwrap().custom_data["operator"], "robot-1");
}

#[test]
fn rejects_invalid_documents() {
    for yaml in [
        "[unclosed".to_owned(),
        EARLY.replace("rosbag2_bagfile_information:", "wrong_root:"),
        EARLY.replace("version: 1", "version: 0"),
        EARLY.replace("version: 1", "version: 10"),
        EARLY.replace("  message_count: 1\n", ""),
        EARLY.replace("message_count: 1", "message_count: -1"),
        EARLY.replace("message_count: 1", "message_count: 18446744073709551616"),
        EARLY.replace("nanoseconds: 100", "nanoseconds: nope"),
        EARLY.replace(
            "relative_file_paths: [recording/recording_0.db3]",
            "relative_file_paths: {}",
        ),
        metadata_yaml(9).replace("offered_qos_profiles: []", "offered_qos_profiles: 123"),
    ] {
        assert!(from_str(&yaml).is_err(), "unexpectedly accepted: {yaml}");
    }
}

#[test]
fn ignores_unknown_fields() {
    let yaml = EARLY.replace("version: 1", "version: 1\n  extra_field: true");
    assert_eq!(from_str(&yaml).unwrap(), from_str(EARLY).unwrap());
}

#[test]
fn propagates_reader_errors() {
    struct Broken;
    impl std::io::Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("broken reader"))
        }
    }
    assert!(
        from_reader(Broken)
            .unwrap_err()
            .to_string()
            .contains("broken reader")
    );
}

#[test]
fn requires_versioned_fields_from_their_introduction_and_ignores_them_before() {
    for (name, introduced, in_topic) in [
        ("compression_format", 3, false),
        ("compression_mode", 3, false),
        ("offered_qos_profiles", 4, true),
        ("files", 5, false),
        ("custom_data", 6, false),
        ("type_description_hash", 7, true),
        ("ros_distro", 8, false),
    ] {
        for version in 1..=9 {
            let mut document: Value = serde_yaml_ng::from_str(&metadata_yaml(version)).unwrap();
            let info = &mut document["rosbag2_bagfile_information"];
            let fields = if in_topic {
                &mut info["topics_with_message_count"][0]["topic_metadata"]
            } else {
                info
            }
            .as_mapping_mut()
            .unwrap();
            fields.remove(Value::from(name));
            let yaml = serde_yaml_ng::to_string(&document).unwrap();
            if version >= introduced {
                let error = from_str(&yaml).unwrap_err().to_string();
                assert!(error.contains(name), "v{version}: {error}");
                assert!(from_reader(yaml.as_bytes()).is_err());
            } else {
                assert!(from_str(&yaml).is_ok(), "v{version}: {name}");
                // Even malformed newer fields are ignored by old-version readers.
                let info = &mut document["rosbag2_bagfile_information"];
                let fields = if in_topic {
                    &mut info["topics_with_message_count"][0]["topic_metadata"]
                } else {
                    info
                }
                .as_mapping_mut()
                .unwrap();
                fields.insert(Value::from(name), Value::Bool(true));
                let metadata = from_str(&serde_yaml_ng::to_string(&document).unwrap()).unwrap();
                assert!(metadata.files.is_empty());
                assert!(metadata.custom_data.is_empty());
                assert!(metadata.ros_distro.is_empty());
                let topic = &metadata.topics_with_message_count[0].topic_metadata;
                if name == "offered_qos_profiles" {
                    assert!(topic.offered_qos_profiles.is_none());
                }
                if name == "type_description_hash" {
                    assert!(topic.type_description_hash.is_empty());
                }
            }
        }
    }
}

#[test]
fn checks_qos_container_and_policy_representation_for_each_version() {
    for version in 4..=9 {
        let yaml = metadata_yaml(version);
        let empty = if version < 9 { "''" } else { "[]" };
        let wrong = if version < 9 { "[]" } else { "''" };
        assert!(
            from_str(&yaml.replace(
                &format!("offered_qos_profiles: {empty}"),
                &format!("offered_qos_profiles: {wrong}"),
            ))
            .is_err(),
            "v{version}: wrong QoS container"
        );
        for name in ["history", "reliability", "durability", "liveliness"] {
            let (valid, invalid) = if version < 9 {
                (
                    format!("|-\n          - {name}: 1"),
                    format!("|-\n          - {name}: reliable"),
                )
            } else {
                (
                    format!("[{{{name}: system_default}}]"),
                    format!("[{{{name}: 1}}]"),
                )
            };
            let prefix = format!("offered_qos_profiles: {empty}");
            assert!(
                from_str(&yaml.replace(&prefix, &format!("offered_qos_profiles: {valid}"))).is_ok(),
                "v{version}: {name}"
            );
            let error =
                from_str(&yaml.replace(&prefix, &format!("offered_qos_profiles: {invalid}")))
                    .unwrap_err()
                    .to_string();
            assert!(error.contains(name), "v{version}: {error}");
        }
    }
    assert!(
        from_str(&metadata_yaml(8).replace(
            "offered_qos_profiles: ''",
            "offered_qos_profiles: '[broken'"
        ))
        .is_err()
    );
}

#[test]
fn resolves_file_paths_using_each_versions_base_directory() {
    use std::path::PathBuf;
    for version in 1..=9 {
        let mut metadata = from_str(&metadata_yaml(version)).unwrap();
        for directory in ["/data/recording", "/data/recording/"] {
            assert_eq!(
                metadata.resolved_file_paths(directory),
                [PathBuf::from("/data/recording/recording_0.db3")]
            );
        }
        assert_eq!(
            metadata.resolved_file_paths("recording"),
            [PathBuf::from("recording/recording_0.db3")]
        );
        metadata
            .relative_file_paths
            .push("/other/external.db3".into());
        assert_eq!(
            metadata.resolved_file_paths("/data/recording")[1],
            PathBuf::from("/other/external.db3")
        );
        assert_eq!(
            metadata.relative_file_paths[0],
            if version < 4 {
                "recording/recording_0.db3"
            } else {
                "recording_0.db3"
            }
        );
    }
}
