//! Anonymization mapping and audit trail engine.
//!
//! Generates structured JSON mapping files linking original DICOM tag values to their anonymized replacements,
//! suitable for audit trails, compliance verification, and re-identification databases.

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::error::TransformError;

/// Individual tag transformation audit record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingEntry {
    /// DICOM Tag string representation (e.g., `"(0010,0010)"`).
    pub tag: String,
    /// DICOM Element keyword if known (e.g., `"PatientName"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// Original value prior to transformation.
    pub original_value: String,
    /// New anonymized value after transformation.
    pub new_value: String,
}

/// Anonymization audit map holding all tag mapping records for a DICOM dataset execution.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymizationMap {
    /// Source DICOM dataset path or cloud URI if specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// ISO 8601 timestamp when the transformation map was generated.
    pub timestamp: String,
    /// List of modified tag mapping entries.
    pub entries: Vec<MappingEntry>,
}

impl AnonymizationMap {
    /// Creates a new empty `AnonymizationMap`.
    pub fn new(source: Option<String>) -> Self {
        Self {
            source,
            timestamp: Local::now().to_rfc3339(),
            entries: Vec::new(),
        }
    }

    /// Records a tag transformation entry into the audit map.
    pub fn add_entry(
        &mut self,
        tag: &str,
        keyword: Option<&str>,
        original_value: &str,
        new_value: &str,
    ) {
        self.entries.push(MappingEntry {
            tag: tag.to_string(),
            keyword: keyword.map(|s| s.to_string()),
            original_value: original_value.to_string(),
            new_value: new_value.to_string(),
        });
    }

    /// Serializes the audit map to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns `TransformError::DslParse` if serialization fails.
    pub fn to_json(&self) -> Result<String, TransformError> {
        let json_str = serde_json::to_string_pretty(self)?;
        Ok(json_str)
    }

    /// Saves the audit map JSON to a local path or cloud URI (`s3://`, `gs://`, `az://`).
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if file creation or upload fails.
    pub fn save(&self, location: &str) -> Result<(), TransformError> {
        let json_str = self.to_json()?;
        crate::io::write_bytes(location, json_str.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_json_roundtrip() {
        let mut map = AnonymizationMap::new(Some("s3://bucket/patient1.dcm".to_string()));
        map.add_entry("(0010,0010)", Some("PatientName"), "DOE^JOHN", "ANON^1234");
        map.add_entry("(0010,0020)", Some("PatientID"), "98765", "SUBJ-99");

        let json = map.to_json().unwrap();
        assert!(json.contains("DOE^JOHN"));
        assert!(json.contains("ANON^1234"));
    }
}
