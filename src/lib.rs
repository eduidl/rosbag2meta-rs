//! Parse rosbag2 `metadata.yaml` without a ROS installation.
//!
//! Fields and QoS representations are checked against metadata versions 1–9.
//! Fields introduced after the declared version are ignored, like rosbag2.

#![doc = include_str!("../README.md")]

use std::{
    collections::BTreeMap,
    fmt,
    io::Read,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use serde::{Deserialize, de::DeserializeOwned};

pub use serde_yaml_ng::Value;

const SUPPORTED_VERSIONS: RangeInclusive<u32> = 1..=9;

/// Failures from the YAML parsing entry points.
#[derive(Debug)]
pub enum Error {
    /// The document declares a version this parser cannot interpret.
    UnsupportedVersion { version: u32 },
    /// Invalid YAML or metadata fields.
    Yaml(serde_yaml_ng::Error),
    /// Reading the input failed.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion { version } => write!(
                formatter,
                "unsupported rosbag2 metadata version {version}; expected {SUPPORTED_VERSIONS:?}"
            ),
            Self::Yaml(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedVersion { .. } => None,
            Self::Yaml(error) => Some(error),
            Self::Io(error) => Some(error),
        }
    }
}

impl From<serde_yaml_ng::Error> for Error {
    fn from(error: serde_yaml_ng::Error) -> Self {
        Self::Yaml(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

fn check_version(version: u32) -> Result<(), Error> {
    if SUPPORTED_VERSIONS.contains(&version) {
        Ok(())
    } else {
        Err(Error::UnsupportedVersion { version })
    }
}

/// Parse a complete `metadata.yaml`, including its root wrapper.
pub fn from_str(yaml: &str) -> Result<BagMetadata, Error> {
    from_slice(yaml.as_bytes())
}

/// Read a complete `metadata.yaml` from a file or other byte stream.
pub fn from_reader(mut reader: impl Read) -> Result<BagMetadata, Error> {
    let mut yaml = Vec::new();
    reader.read_to_end(&mut yaml)?;
    from_slice(&yaml)
}

#[derive(Deserialize)]
struct MetadataDocument {
    rosbag2_bagfile_information: Value,
}

#[derive(Deserialize)]
struct CustomDataDocument {
    rosbag2_bagfile_information: CustomDataInformation,
}

#[derive(Deserialize)]
struct CustomDataInformation {
    custom_data: Option<BTreeMap<String, String>>,
}

fn from_slice(yaml: &[u8]) -> Result<BagMetadata, Error> {
    let document: MetadataDocument = serde_yaml_ng::from_slice(yaml)?;
    let mut information = document.rosbag2_bagfile_information;
    let version = information
        .get("version")
        .map(|value| serde_yaml_ng::from_value::<u32>(value.clone()))
        .transpose()?;
    if let Some(version) = version {
        check_version(version)?;
    }
    if version.is_some_and(|version| version >= 6) && information.get("custom_data").is_some() {
        // Read scalars directly as strings; Value would lose spellings such as TRUE and 0x10.
        let custom: CustomDataDocument = serde_yaml_ng::from_slice(yaml)?;
        information["custom_data"] =
            serde_yaml_ng::to_value(custom.rosbag2_bagfile_information.custom_data)?;
    }
    Ok(serde_yaml_ng::from_value(information)?)
}

/// Metadata shared by SQLite3 and MCAP bags; paths are kept as written.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(try_from = "BTreeMap<String, Value>")]
pub struct BagMetadata {
    pub version: u32,
    pub storage_identifier: String,
    pub relative_file_paths: Vec<String>,
    pub duration: Duration,
    pub starting_time: StartingTime,
    pub message_count: u64,
    pub topics_with_message_count: Vec<TopicInformation>,
    /// Added in version 3.
    pub compression_format: String,
    /// Added in version 3.
    pub compression_mode: String,
    /// Added in version 5.
    pub files: Vec<FileInformation>,
    /// Added in version 6. rosbag2 can emit null for an empty map.
    pub custom_data: BTreeMap<String, String>,
    /// Added in version 8.
    pub ros_distro: String,
}

impl BagMetadata {
    /// Resolve storage paths using the version's base directory, without I/O.
    /// `bag_directory` is the directory containing `metadata.yaml`.
    /// Absolute paths are preserved; this does not enforce directory containment.
    pub fn resolved_file_paths(&self, bag_directory: impl AsRef<Path>) -> Vec<PathBuf> {
        let bag_directory = bag_directory.as_ref();
        let base = if self.version < 4 {
            match bag_directory.file_name() {
                Some(_) => bag_directory
                    .parent()
                    .unwrap_or(bag_directory)
                    .to_path_buf(),
                // parent() removes a component; "." and ".." require traversal instead.
                None => bag_directory.join(".."),
            }
        } else {
            bag_directory.to_path_buf()
        };
        self.relative_file_paths
            .iter()
            .map(|path| base.join(path))
            .collect()
    }
}

impl TryFrom<BTreeMap<String, Value>> for BagMetadata {
    type Error = String;

    fn try_from(mut fields: BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let version: u32 = field(&mut fields, "version")?;
        check_version(version).map_err(|error| error.to_string())?;
        let topics: Vec<BTreeMap<String, Value>> = field(&mut fields, "topics_with_message_count")?;
        let topics_with_message_count = topics
            .into_iter()
            .enumerate()
            .map(|(index, mut topic)| {
                let mut parse = || -> Result<TopicInformation, String> {
                    let mut metadata = field(&mut topic, "topic_metadata")?;
                    let offered_qos_profiles = if version < 4 {
                        None
                    } else {
                        let profiles = if version < 9 {
                            OfferedQosProfiles::Legacy(field(
                                &mut metadata,
                                "offered_qos_profiles",
                            )?)
                        } else {
                            OfferedQosProfiles::Profiles(field(
                                &mut metadata,
                                "offered_qos_profiles",
                            )?)
                        };
                        validate_qos(&profiles)?;
                        Some(profiles)
                    };
                    Ok(TopicInformation {
                        topic_metadata: TopicMetadata {
                            name: field(&mut metadata, "name")?,
                            topic_type: field(&mut metadata, "type")?,
                            serialization_format: field(&mut metadata, "serialization_format")?,
                            offered_qos_profiles,
                            type_description_hash: versioned_field(
                                &mut metadata,
                                "type_description_hash",
                                version,
                                7,
                            )?,
                        },
                        message_count: field(&mut topic, "message_count")?,
                    })
                };
                parse().map_err(|error| format!("topics_with_message_count[{index}]: {error}"))
            })
            .collect::<Result<_, _>>()?;
        let custom_data: Option<BTreeMap<String, String>> =
            versioned_field(&mut fields, "custom_data", version, 6)?;
        Ok(Self {
            version,
            storage_identifier: field(&mut fields, "storage_identifier")?,
            relative_file_paths: field(&mut fields, "relative_file_paths")?,
            duration: field(&mut fields, "duration")?,
            starting_time: field(&mut fields, "starting_time")?,
            message_count: field(&mut fields, "message_count")?,
            topics_with_message_count,
            compression_format: versioned_field(&mut fields, "compression_format", version, 3)?,
            compression_mode: versioned_field(&mut fields, "compression_mode", version, 3)?,
            files: versioned_field(&mut fields, "files", version, 5)?,
            custom_data: custom_data.unwrap_or_default(),
            ros_distro: versioned_field(&mut fields, "ros_distro", version, 8)?,
        })
    }
}

/// Elapsed time, stored in nanoseconds without floating point conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Duration {
    pub nanoseconds: u64,
}

/// Recording start time relative to the Unix epoch, in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct StartingTime {
    pub nanoseconds_since_epoch: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopicInformation {
    pub topic_metadata: TopicMetadata,
    pub message_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopicMetadata {
    pub name: String,
    pub topic_type: String,
    pub serialization_format: String,
    /// Required from version 4; None before then. Original representation retained.
    pub offered_qos_profiles: Option<OfferedQosProfiles>,
    /// Added in version 7.
    pub type_description_hash: String,
}

/// QoS was a YAML-encoded string before version 9 and a sequence thereafter.
#[derive(Debug, Clone, PartialEq)]
pub enum OfferedQosProfiles {
    Legacy(String),
    Profiles(Vec<BTreeMap<String, Value>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FileInformation {
    pub path: String,
    pub starting_time: StartingTime,
    pub duration: Duration,
    pub message_count: u64,
}

fn field<T: DeserializeOwned>(
    fields: &mut BTreeMap<String, Value>,
    name: &str,
) -> Result<T, String> {
    let value = fields
        .remove(name)
        .ok_or_else(|| format!("missing field `{name}`"))?;
    serde_yaml_ng::from_value(value).map_err(|error| format!("{name}: {error}"))
}

fn versioned_field<T: DeserializeOwned + Default>(
    fields: &mut BTreeMap<String, Value>,
    name: &str,
    version: u32,
    introduced: u32,
) -> Result<T, String> {
    if version < introduced {
        Ok(T::default())
    } else {
        field(fields, name)
    }
}

fn validate_qos(qos: &OfferedQosProfiles) -> Result<(), String> {
    // shortcut: policy values stay opaque, add enum validation when QoS is interpreted.
    let legacy;
    let (profiles, numeric) = match qos {
        OfferedQosProfiles::Legacy(text) => {
            legacy = if text.trim().is_empty() {
                Vec::new()
            } else {
                serde_yaml_ng::from_str::<Vec<BTreeMap<String, Value>>>(text)
                    .map_err(|error| format!("offered_qos_profiles: {error}"))?
            };
            (&legacy, true)
        }
        OfferedQosProfiles::Profiles(profiles) => (profiles, false),
    };
    for (index, profile) in profiles.iter().enumerate() {
        for name in ["history", "reliability", "durability", "liveliness"] {
            if let Some(value) = profile.get(name) {
                let valid = if numeric {
                    value.as_u64().is_some()
                } else {
                    value.as_str().is_some()
                };
                if !valid {
                    let expected = if numeric {
                        "integer policy (version 4–8)"
                    } else {
                        "string policy (version 9)"
                    };
                    return Err(format!(
                        "offered_qos_profiles[{index}].{name}: expected {expected}"
                    ));
                }
            }
        }
    }
    Ok(())
}
