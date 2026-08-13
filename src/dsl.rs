//! Data structures for the JSON-encoded DICOM transformation DSL.

use dicom_core::dictionary::DataDictionary;
use dicom_core::Tag;
use dicom_dictionary_std::StandardDataDictionary;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::TransformError;

/// Tag selector used to target specific DICOM elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagSelector {
    /// Target by DICOM keyword (e.g., `"PatientName"`, `"PatientID"`) or hex string (`"0010,0010"`).
    Keyword(String),
    /// Target by explicit group and element values.
    Tuple {
        /// DICOM Tag Group number (e.g. `0x0010`).
        group: u16,
        /// DICOM Tag Element number (e.g. `0x0010`).
        element: u16,
    },
}

mod tag_serde {
    use dicom_core::Tag;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(tag: &Tag, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (tag.0, tag.1).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Tag, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (group, element) = <(u16, u16)>::deserialize(deserializer)?;
        Ok(Tag(group, element))
    }
}

/// A single segment in a DICOM tag path expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagPathSegment {
    /// Target DICOM tag.
    #[serde(with = "tag_serde")]
    pub tag: Tag,
    /// Optional sequence item index (e.g. `Some(0)` for `[0]`).
    /// `None` indicates all items (wildcard / missing index).
    pub item_index: Option<usize>,
}

impl fmt::Display for TagPathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:04X},{:04X})", self.tag.0, self.tag.1)?;
        if let Some(idx) = self.item_index {
            write!(f, "[{}]", idx)?;
        }
        Ok(())
    }
}

/// A path expression representing a sequence hierarchy of DICOM elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagPath {
    /// Ordered segments from top-level sequence down to leaf tag.
    pub segments: Vec<TagPathSegment>,
}

impl TagPath {
    /// Creates a single-tag `TagPath` (top-level element).
    pub fn single(tag: Tag) -> Self {
        Self {
            segments: vec![TagPathSegment {
                tag,
                item_index: None,
            }],
        }
    }

    /// Parses a path string into a `TagPath`.
    /// In Community Edition, nested sequence paths (containing '/' or '[...]') are rejected.
    pub fn parse(raw: &str) -> Result<Self, TransformError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(TransformError::InvalidOperation(
                "Tag path cannot be empty".to_string(),
            ));
        }

        if raw.contains('/') || raw.contains('[') || raw.contains(']') {
            return Err(TransformError::ProFeatureRequired(format!(
                "Nested DICOM Sequence path '{}' is a PRO feature. Community edition supports top-level tags only. Please upgrade to dicom-rs-transformer-pro for DICOM Sequence path evaluation.",
                raw
            )));
        }

        let (tag_str, _) = parse_segment_str(raw)?;
        let tag = parse_tag_string(tag_str)?;
        let segment = TagPathSegment {
            tag,
            item_index: None,
        };

        Ok(TagPath {
            segments: vec![segment],
        })
    }

    /// Returns the leaf tag (the tag of the final segment).
    pub fn leaf_tag(&self) -> Tag {
        self.segments.last().unwrap().tag
    }

    /// Returns `true` if this path refers to a top-level tag without sequence steps.
    pub fn is_single(&self) -> bool {
        self.segments.len() == 1 && self.segments[0].item_index.is_none()
    }
}

impl fmt::Display for TagPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seg_strs: Vec<String> = self.segments.iter().map(|s| s.to_string()).collect();
        write!(f, "{}", seg_strs.join("/"))
    }
}

fn parse_segment_str(s: &str) -> Result<(&str, Option<usize>), TransformError> {
    let s = s.trim();
    if let Some(bracket_idx) = s.rfind('[') {
        if s.ends_with(']') {
            let tag_part = s[..bracket_idx].trim();
            let idx_part = s[bracket_idx + 1..s.len() - 1].trim();
            if idx_part.is_empty() || idx_part == "*" {
                return Ok((tag_part, None));
            }
            let idx = idx_part.parse::<usize>().map_err(|_| {
                TransformError::InvalidOperation(format!(
                    "Invalid sequence item index '[{}]'",
                    idx_part
                ))
            })?;
            return Ok((tag_part, Some(idx)));
        }
    }
    Ok((s, None))
}

impl TagSelector {
    /// Resolves the `TagSelector` into a concrete `dicom_core::Tag`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError::UnknownTag` if the selector cannot be parsed or looked up.
    pub fn resolve(&self) -> Result<Tag, TransformError> {
        let path = self.resolve_path()?;
        Ok(path.leaf_tag())
    }

    /// Resolves the `TagSelector` into a structured `TagPath`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError::UnknownTag` if any tag in the path cannot be resolved.
    pub fn resolve_path(&self) -> Result<TagPath, TransformError> {
        match self {
            TagSelector::Tuple { group, element } => Ok(TagPath::single(Tag(*group, *element))),
            TagSelector::Keyword(raw) => TagPath::parse(raw),
        }
    }
}

impl fmt::Display for TagSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagSelector::Keyword(kw) => write!(f, "{}", kw),
            TagSelector::Tuple { group, element } => write!(f, "({:04X},{:04X})", group, element),
        }
    }
}

impl FromStr for TagSelector {
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(TransformError::InvalidOperation(
                "Tag selector string cannot be empty".to_string(),
            ));
        }
        Ok(TagSelector::Keyword(trimmed.to_string()))
    }
}

/// Represents an individual DICOM dataset transformation operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Action {
    /// Load a DICOM dataset from a local file path or cloud URI (s3://, gs://, az://).
    LoadDataset {
        /// Source location URI or file path.
        location: String,
    },
    /// Save the current DICOM dataset to a local file path or cloud URI (s3://, gs://, az://).
    SaveDataset {
        /// Destination location URI or file path.
        location: String,
    },
    /// Export the anonymization audit map JSON to a local file path or cloud URI (s3://, gs://, az://).
    SaveMap {
        /// Destination location URI or file path for JSON audit map.
        location: String,
    },
    /// Extract DICOM dataset image frames to JPEG, PNG, or RAW files in target directory or URI (s3://, gs://, az://).
    ExtractPixels {
        /// Destination folder URI or local directory path.
        destination: String,
        /// Image export format ("jpeg", "png", "raw"). Defaults to "jpeg".
        #[serde(default = "default_pixel_format")]
        format: String,
    },
    /// Set or update the value of a specific DICOM tag.
    SetTag {
        /// Target tag selector.
        selector: TagSelector,
        /// New string value to set for the tag.
        value: String,
    },
    /// Remove a DICOM tag from the dataset.
    RemoveTag {
        /// Target tag selector.
        selector: TagSelector,
    },
    /// Perform substring replacement on a string-based DICOM tag's value.
    ReplaceValue {
        /// Target tag selector.
        selector: TagSelector,
        /// Substring pattern to match.
        pattern: String,
        /// Replacement text.
        replacement: String,
    },
    /// Anonymize standard DICOM patient identification fields.
    AnonymizePatient {
        /// Replacement patient name (defaults to `"ANONYMOUS"` if None).
        patient_name: Option<String>,
        /// Replacement patient ID (defaults to `"ANON-ID"` if None).
        patient_id: Option<String>,
    },
}

/// Specification holding metadata and an ordered sequence of transformation actions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformSpec {
    /// Specification format version string (e.g., `"1.0"`).
    #[serde(default = "default_version")]
    pub version: String,

    /// Optional descriptive name for this transformation rule set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional detailed description of what the transformation accomplishes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// List of actions to execute in order.
    pub actions: Vec<Action>,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_pixel_format() -> String {
    "jpeg".to_string()
}

impl TransformSpec {
    /// Create a new empty `TransformSpec`.
    pub fn new() -> Self {
        Self {
            version: default_version(),
            name: None,
            description: None,
            actions: Vec::new(),
        }
    }

    /// Add an action to the specification.
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Deserialize a `TransformSpec` from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns `TransformError::DslParse` if JSON deserialization fails.
    pub fn from_json(json_str: &str) -> Result<Self, TransformError> {
        let spec: TransformSpec = serde_json::from_str(json_str)?;
        Ok(spec)
    }

    /// Serialize the `TransformSpec` into a formatted JSON string.
    ///
    /// # Errors
    ///
    /// Returns `TransformError::DslParse` if serialization fails.
    pub fn to_json(&self) -> Result<String, TransformError> {
        let json_str = serde_json::to_string_pretty(self)?;
        Ok(json_str)
    }

    /// Decompile/format the `TransformSpec` into a human-readable line-by-line text script string.
    pub fn to_script(&self) -> String {
        let mut lines = Vec::new();

        if let Some(ref name) = self.name {
            lines.push(format!("# {}", name));
        }
        if let Some(ref desc) = self.description {
            lines.push(format!("# {}", desc));
        }
        if self.name.is_some() || self.description.is_some() {
            lines.push(String::new());
        }

        for action in &self.actions {
            match action {
                Action::LoadDataset { location } => {
                    lines.push(format!("LOAD \"{}\"", location));
                }
                Action::SaveDataset { location } => {
                    lines.push(format!("SAVE \"{}\"", location));
                }
                Action::SaveMap { location } => {
                    lines.push(format!("SAVE_MAP \"{}\"", location));
                }
                Action::ExtractPixels {
                    destination,
                    format,
                } => {
                    lines.push(format!(
                        "EXTRACT_PIXELS \"{}\" FORMAT=\"{}\"",
                        destination, format
                    ));
                }
                Action::SetTag { selector, value } => {
                    lines.push(format!("SET {} \"{}\"", selector, value));
                }
                Action::RemoveTag { selector } => {
                    lines.push(format!("DELETE {}", selector));
                }
                Action::ReplaceValue {
                    selector,
                    pattern,
                    replacement,
                } => {
                    lines.push(format!(
                        "REPLACE {} \"{}\" WITH \"{}\"",
                        selector, pattern, replacement
                    ));
                }
                Action::AnonymizePatient {
                    patient_name,
                    patient_id,
                } => {
                    let mut opts = Vec::new();
                    if let Some(ref name) = patient_name {
                        opts.push(format!("NAME=\"{}\"", name));
                    }
                    if let Some(ref id) = patient_id {
                        opts.push(format!("ID=\"{}\"", id));
                    }
                    if opts.is_empty() {
                        lines.push("ANONYMIZE".to_string());
                    } else {
                        lines.push(format!("ANONYMIZE {}", opts.join(" ")));
                    }
                }
            }
        }

        lines.join("\n")
    }
}

/// Parses a raw tag string into a `dicom_core::Tag`.
///
/// Accepts formats:
/// - DICOM Keyword: `"PatientName"`
/// - Hex string: `"0010,0010"`, `"(0010,0010)"`, or `"00100010"`
fn parse_tag_string(raw: &str) -> Result<Tag, TransformError> {
    let clean = raw.trim().trim_matches(|c| c == '(' || c == ')');

    // Check if it's formatted like 0010,0010
    if let Some((g_str, e_str)) = clean.split_once(',') {
        let group = u16::from_str_radix(g_str.trim(), 16)
            .map_err(|_| TransformError::UnknownTag(raw.to_string()))?;
        let element = u16::from_str_radix(e_str.trim(), 16)
            .map_err(|_| TransformError::UnknownTag(raw.to_string()))?;
        return Ok(Tag(group, element));
    }

    // Check if it's 8 hexadecimal digits like 00100010
    if clean.len() == 8 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
        let group = u16::from_str_radix(&clean[0..4], 16)
            .map_err(|_| TransformError::UnknownTag(raw.to_string()))?;
        let element = u16::from_str_radix(&clean[4..8], 16)
            .map_err(|_| TransformError::UnknownTag(raw.to_string()))?;
        return Ok(Tag(group, element));
    }

    // Try Standard Dictionary lookup by keyword
    let dict = StandardDataDictionary;
    dict.by_name(clean)
        .map(|entry| entry.tag.inner())
        .ok_or_else(|| TransformError::UnknownTag(raw.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_selector_resolution() {
        let sel_keyword = TagSelector::Keyword("PatientName".to_string());
        assert_eq!(sel_keyword.resolve().unwrap(), Tag(0x0010, 0x0010));

        let sel_hex = TagSelector::Keyword("(0010,0020)".to_string());
        assert_eq!(sel_hex.resolve().unwrap(), Tag(0x0010, 0x0020));

        let sel_tuple = TagSelector::Tuple {
            group: 0x0010,
            element: 0x0010,
        };
        assert_eq!(sel_tuple.resolve().unwrap(), Tag(0x0010, 0x0010));
    }

    #[test]
    fn test_spec_json_roundtrip() {
        let mut spec = TransformSpec::new();
        spec.name = Some("Anonymize Test".to_string());
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword("PatientName".to_string()),
            value: "ANON^JOHN".to_string(),
        });
        spec.add_action(Action::RemoveTag {
            selector: TagSelector::Keyword("PatientAddress".to_string()),
        });

        let json = spec.to_json().expect("Serialization failed");
        let restored = TransformSpec::from_json(&json).expect("Deserialization failed");
        assert_eq!(spec, restored);
    }

    #[test]
    fn test_tag_path_parsing() {
        // Single keyword tag
        let path1 = TagPath::parse("PatientName").unwrap();
        assert_eq!(path1.segments.len(), 1);
        assert_eq!(path1.segments[0].tag, Tag(0x0010, 0x0010));
        assert_eq!(path1.segments[0].item_index, None);

        // Sequence paths rejected in Community Edition
        let seq_paths = [
            "RequestAttributesSequence[0]/ScheduledProcedureStepID",
            "RequestAttributesSequence/ScheduledProcedureStepID",
            "(0040,0275)[*]/(0040,0009)",
        ];

        for seq in seq_paths {
            let err = TagPath::parse(seq).unwrap_err();
            match err {
                TransformError::ProFeatureRequired(msg) => {
                    assert!(msg.contains("Nested DICOM Sequence path"));
                    assert!(msg.contains("dicom-rs-transformer-pro"));
                }
                _ => panic!("Expected ProFeatureRequired error for sequence path: {}", seq),
            }
        }
    }

    #[test]
    fn test_to_script_generation() {
        let mut spec = TransformSpec::new();
        spec.name = Some("Test Script".to_string());
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword("PatientName".to_string()),
            value: "DOE^JOHN".to_string(),
        });
        spec.add_action(Action::RemoveTag {
            selector: TagSelector::Keyword("PatientAddress".to_string()),
        });
        spec.add_action(Action::ReplaceValue {
            selector: TagSelector::Keyword("StudyDescription".to_string()),
            pattern: "HOSP".to_string(),
            replacement: "CLINIC".to_string(),
        });

        let script = spec.to_script();
        assert!(script.contains("# Test Script"));
        assert!(script.contains("SET PatientName \"DOE^JOHN\""));
        assert!(script.contains("DELETE PatientAddress"));
        assert!(script.contains("REPLACE StudyDescription \"HOSP\" WITH \"CLINIC\""));
    }
}
