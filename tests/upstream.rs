use rosbag2meta_rs::{OfferedQosProfiles, from_reader, from_str};

#[test]
fn parses_official_metadata_fixtures() {
    for (version, yaml, storage, message_count, topic_count) in [
        (
            1,
            include_str!("fixtures/upstream/metadata-v1.yaml"),
            "sqlite3",
            7,
            2,
        ),
        (
            3,
            include_str!("fixtures/upstream/metadata-v3.yaml"),
            "sqlite3",
            7,
            2,
        ),
        (
            4,
            include_str!("fixtures/upstream/metadata-v4.yaml"),
            "sqlite3",
            20,
            3,
        ),
        (
            5,
            include_str!("fixtures/upstream/metadata-v5.yaml"),
            "sqlite3",
            150,
            2,
        ),
        (
            6,
            include_str!("fixtures/upstream/metadata-v6.yaml"),
            "mcap",
            75,
            2,
        ),
        (
            7,
            include_str!("fixtures/upstream/metadata-v7.yaml"),
            "mcap",
            7,
            2,
        ),
        (
            8,
            include_str!("fixtures/upstream/metadata-v8.yaml"),
            "sqlite3",
            10,
            5,
        ),
        (
            9,
            include_str!("fixtures/upstream/metadata-v9.yaml"),
            "sqlite3",
            187,
            17,
        ),
    ] {
        let metadata =
            from_str(yaml).unwrap_or_else(|error| panic!("official v{version}: {error}"));
        assert_eq!(from_reader(yaml.as_bytes()).unwrap(), metadata);
        assert_eq!(metadata.version, version);
        assert_eq!(metadata.storage_identifier, storage);
        assert_eq!(metadata.message_count, message_count);
        assert_eq!(metadata.topics_with_message_count.len(), topic_count);
        assert_eq!(
            metadata
                .topics_with_message_count
                .iter()
                .map(|topic| topic.message_count)
                .sum::<u64>(),
            message_count
        );
        assert_eq!(metadata.relative_file_paths.len(), 1);
        assert!(metadata.duration.nanoseconds > 0);
        assert!(metadata.custom_data.is_empty());
        if version >= 5 {
            assert_eq!(metadata.files.len(), 1);
            let file = &metadata.files[0];
            assert_eq!(file.path, metadata.relative_file_paths[0]);
            assert_eq!(file.message_count, message_count);
            assert_eq!(file.starting_time, metadata.starting_time);
            assert_eq!(file.duration, metadata.duration);
        } else {
            assert!(metadata.files.is_empty());
        }
        for topic in &metadata.topics_with_message_count {
            assert_eq!(topic.topic_metadata.serialization_format, "cdr");
            match &topic.topic_metadata.offered_qos_profiles {
                None => assert!(version < 4),
                Some(OfferedQosProfiles::Legacy(text)) => {
                    assert!((4..=8).contains(&version));
                    assert!(text.contains("reliability: 1"));
                }
                Some(OfferedQosProfiles::Profiles(profiles)) => {
                    assert_eq!(version, 9);
                    assert!(!profiles.is_empty());
                    assert!(profiles[0]["reliability"].as_str().is_some());
                }
            }
        }
        if version < 4 {
            assert_eq!(
                metadata.topics_with_message_count[0]
                    .topic_metadata
                    .topic_type,
                "test_msgs/BasicTypes"
            );
        }
        if version == 9 {
            assert_eq!(metadata.ros_distro, "rolling");
            assert!(
                metadata.topics_with_message_count[0]
                    .topic_metadata
                    .type_description_hash
                    .starts_with("RIHS01_")
            );
        }
    }
}
