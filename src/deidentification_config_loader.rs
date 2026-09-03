//! Loader module for loading DICOM PS3.15 Annex E de-identification rules and profiles.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::TransformError;
use crate::models::deidentification_config::{
    DeidentificationConfig, DeidentificationProfile, TableE11Rule,
};

/// Default file path for the de-identification profile JSON.
pub const DEFAULT_PROFILE_PATH: &str = "configs/anonymization_profile.current.json";

/// Compile-time embedded default profile JSON string.
pub const DEFAULT_PROFILE_JSON: &str = include_str!("../configs/anonymization_profile.current.json");

/// Loads Table E.1-1 de-identification rules from a JSON file path.
///
/// If `path` is `None`, defaults to loading from [`DEFAULT_PROFILE_PATH`].
/// If the default file is not found on disk at runtime, it falls back to the embedded default profile JSON.
pub fn load_deidentification_rules<P: AsRef<Path>>(
    path: Option<P>,
) -> Result<Vec<TableE11Rule>, TransformError> {
    match path {
        Some(ref p) => {
            let path_ref = p.as_ref();
            if !path_ref.exists() && path_ref.to_string_lossy() == DEFAULT_PROFILE_PATH {
                parse_deidentification_rules_json(DEFAULT_PROFILE_JSON)
            } else {
                let file = File::open(path_ref)?;
                let reader = BufReader::new(file);
                let rules: Vec<TableE11Rule> = serde_json::from_reader(reader)?;
                Ok(rules)
            }
        }
        None => {
            let path_ref = Path::new(DEFAULT_PROFILE_PATH);
            if path_ref.exists() {
                let file = File::open(path_ref)?;
                let reader = BufReader::new(file);
                let rules: Vec<TableE11Rule> = serde_json::from_reader(reader)?;
                Ok(rules)
            } else {
                parse_deidentification_rules_json(DEFAULT_PROFILE_JSON)
            }
        }
    }
}

/// Parses Table E.1-1 de-identification rules from a JSON string.
pub fn parse_deidentification_rules_json(json_str: &str) -> Result<Vec<TableE11Rule>, TransformError> {
    let rules: Vec<TableE11Rule> = serde_json::from_str(json_str)?;
    Ok(rules)
}

/// Loads a full [`DeidentificationProfile`] from a JSON file path.
///
/// If `path` is `None`, defaults to [`DEFAULT_PROFILE_PATH`].
/// Optional `config` allows overriding runtime flags.
pub fn load_deidentification_profile<P: AsRef<Path>>(
    path: Option<P>,
    config: Option<DeidentificationConfig>,
) -> Result<DeidentificationProfile, TransformError> {
    let rules = load_deidentification_rules(path)?;
    Ok(DeidentificationProfile {
        rules,
        config: config.unwrap_or_default(),
    })
}

/// Parses a full [`DeidentificationProfile`] from a JSON string.
///
/// Supports parsing both a JSON array of `TableE11Rule` items or a JSON object containing `{ "rules": [...], "config": {...} }`.
pub fn parse_deidentification_profile_json(
    json_str: &str,
    config: Option<DeidentificationConfig>,
) -> Result<DeidentificationProfile, TransformError> {
    if let Ok(profile) = serde_json::from_str::<DeidentificationProfile>(json_str) {
        let final_config = config.unwrap_or(profile.config);
        Ok(DeidentificationProfile {
            rules: profile.rules,
            config: final_config,
        })
    } else {
        let rules: Vec<TableE11Rule> = serde_json::from_str(json_str)?;
        Ok(DeidentificationProfile {
            rules,
            config: config.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_default_deidentification_rules() {
        let rules = load_deidentification_rules(None::<&str>).expect("Failed to load default rules");
        assert!(!rules.is_empty(), "Rules list should not be empty");
        // Verify Accession Number tag (0008,0050)
        let accession_rule = rules.iter().find(|r| r.tag == "(0008,0050)");
        assert!(accession_rule.is_some());
        assert_eq!(accession_rule.unwrap().attribute_name, "Accession Number");
    }

    #[test]
    fn test_load_custom_json_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        let sample_json = r#"[
            {
                "attribute_name": "Patient's Name",
                "tag": "(0010,0010)",
                "retired": false,
                "in_std_comp_iod": true,
                "basic_profile": "Z",
                "options": {}
            }
        ]"#;
        temp_file.write_all(sample_json.as_bytes()).unwrap();

        let rules = load_deidentification_rules(Some(temp_file.path())).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].attribute_name, "Patient's Name");
    }

    #[test]
    fn test_parse_deidentification_profile_json() {
        let profile = parse_deidentification_profile_json(DEFAULT_PROFILE_JSON, None)
            .expect("Failed to parse embedded profile JSON");
        assert!(!profile.rules.is_empty());
    }
}

