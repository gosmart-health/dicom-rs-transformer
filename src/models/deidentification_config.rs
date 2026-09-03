//! De-identification configuration data models corresponding to DICOM PS3.15 Annex E standards.
//!
//! Equivalent to `docs/designs/di_struct.py` Python Pydantic definitions.

use serde::{Deserialize, Serialize};

/// PS3.15 Annex E Action Codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionCode {
    /// Delete / replace with zero-length value
    #[serde(rename = "D")]
    D,
    /// Zero-length value
    #[serde(rename = "Z")]
    Z,
    /// Remove tag entirely
    #[serde(rename = "X")]
    X,
    /// Keep tag unchanged
    #[serde(rename = "K")]
    K,
    /// Clean / replace with dummy or anonymized value
    #[serde(rename = "C")]
    C,
    /// Replace with unique UID
    #[serde(rename = "U")]
    U,
    /// Z unless option overrides to D
    #[serde(rename = "Z/D")]
    ZD,
    /// X unless option overrides to Z
    #[serde(rename = "X/Z")]
    XZ,
    /// X unless option overrides to D
    #[serde(rename = "X/D")]
    XD,
    /// X unless option overrides to Z or D
    #[serde(rename = "X/Z/D")]
    XZD,
    /// X unless option overrides to Z, U, or *
    #[serde(rename = "X/Z/U*")]
    XZUStar,
}

impl ActionCode {
    /// Returns string representation of action code.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::D => "D",
            Self::Z => "Z",
            Self::X => "X",
            Self::K => "K",
            Self::C => "C",
            Self::U => "U",
            Self::ZD => "Z/D",
            Self::XZ => "X/Z",
            Self::XD => "X/D",
            Self::XZD => "X/Z/D",
            Self::XZUStar => "X/Z/U*",
        }
    }
}

impl std::fmt::Display for ActionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Profile option overrides for DICOM PS3.15 Annex E attributes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileOptions {
    /// Retain Safe Private Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_safe_private: Option<ActionCode>,
    /// Retain UIDs Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_uids: Option<ActionCode>,
    /// Retain Device Identity Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_dev_id: Option<ActionCode>,
    /// Retain Institution Identity Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_inst_id: Option<ActionCode>,
    /// Retain Patient Characteristics Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_pat_chars: Option<ActionCode>,
    /// Retain Longitudinal Temporal Information with Full Dates Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_long_full_dates: Option<ActionCode>,
    /// Retain Longitudinal Temporal Information with Modified Dates Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retain_long_mod_dates: Option<ActionCode>,
    /// Clean Descriptors Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_desc: Option<ActionCode>,
    /// Clean Structured Content Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_struct_cont: Option<ActionCode>,
    /// Clean Graphics Option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_graph: Option<ActionCode>,
}

fn default_true() -> bool {
    true
}

/// Represents a single rule entry from PS3.15 Annex E Table E.1-1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableE11Rule {
    /// DICOM tag formatted string, e.g. "(0010,0010)"
    pub tag: String,
    /// Attribute name as specified in DICOM standard
    pub attribute_name: String,
    /// Indicates whether the attribute is retired in DICOM standard
    #[serde(default)]
    pub retired: bool,
    /// Indicates whether the attribute is in standard composite IOD
    #[serde(default = "default_true")]
    pub in_std_comp_iod: bool,
    /// Action code for Basic Application Level Confidentiality Profile
    pub basic_profile: ActionCode,
    /// Additional profile options overriding basic profile action
    #[serde(default)]
    pub options: ProfileOptions,
}

impl TableE11Rule {
    /// Normalizes a DICOM tag string into standard "(GGGG,EEEE)" uppercase format.
    pub fn normalize_tag(v: &str) -> Result<String, String> {
        let trimmed = v.trim().trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = trimmed
            .split(|c| c == ',' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
        
        if parts.len() == 2 && parts[0].len() == 4 && parts[1].len() == 4 {
            let g = u16::from_str_radix(parts[0], 16)
                .map_err(|_| format!("Invalid hex group in tag: {}", v))?;
            let e = u16::from_str_radix(parts[1], 16)
                .map_err(|_| format!("Invalid hex element in tag: {}", v))?;
            Ok(format!("({:04X},{:04X})", g, e))
        } else if trimmed.len() == 8 && parts.len() == 1 {
            let (g_str, e_str) = trimmed.split_at(4);
            let g = u16::from_str_radix(g_str, 16)
                .map_err(|_| format!("Invalid hex group in tag: {}", v))?;
            let e = u16::from_str_radix(e_str, 16)
                .map_err(|_| format!("Invalid hex element in tag: {}", v))?;
            Ok(format!("({:04X},{:04X})", g, e))
        } else {
            Err(format!("Invalid DICOM tag format: {}", v))
        }
    }

    /// Converts the formatted tag string into an integer tuple `(group, element)`.
    pub fn int_tuple(&self) -> Result<(u16, u16), String> {
        let normalized = Self::normalize_tag(&self.tag)?;
        let clean = normalized.trim_matches(|c| c == '(' || c == ')');
        let mut parts = clean.split(',');
        let g_str = parts.next().ok_or_else(|| "Missing group".to_string())?;
        let e_str = parts.next().ok_or_else(|| "Missing element".to_string())?;
        let g = u16::from_str_radix(g_str, 16).map_err(|e| e.to_string())?;
        let e = u16::from_str_radix(e_str, 16).map_err(|e| e.to_string())?;
        Ok((g, e))
    }
}

/// Runtime flags determining which PS3.15 Annex E options are enabled.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeidentificationConfig {
    /// Retain Safe Private Option flag
    #[serde(default)]
    pub retain_safe_private: bool,
    /// Retain UIDs Option flag
    #[serde(default)]
    pub retain_uids: bool,
    /// Retain Device Identity Option flag
    #[serde(default)]
    pub retain_dev_id: bool,
    /// Retain Institution Identity Option flag
    #[serde(default)]
    pub retain_inst_id: bool,
    /// Retain Patient Characteristics Option flag
    #[serde(default)]
    pub retain_pat_chars: bool,
    /// Retain Longitudinal Temporal Information with Full Dates Option flag
    #[serde(default)]
    pub retain_long_full_dates: bool,
    /// Retain Longitudinal Temporal Information with Modified Dates Option flag
    #[serde(default)]
    pub retain_long_mod_dates: bool,
    /// Clean Descriptors Option flag
    #[serde(default)]
    pub clean_desc: bool,
    /// Clean Structured Content Option flag
    #[serde(default)]
    pub clean_struct_cont: bool,
    /// Clean Graphics Option flag
    #[serde(default)]
    pub clean_graph: bool,
}

/// Top-level SHADE de-identification profile containing Table E.1-1 rules and configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadeDeidentificationProfile {
    /// Rules defining tag-level actions according to PS3.15 Annex E Table E.1-1
    #[serde(default)]
    pub rules: Vec<TableE11Rule>,
    /// Runtime configuration flags determining enabled profile options
    #[serde(default)]
    pub config: DeidentificationConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_code_serde() {
        let code = ActionCode::XZUStar;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"X/Z/U*\"");
        let deserialized: ActionCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ActionCode::XZUStar);
    }

    #[test]
    fn test_tag_normalization_and_int_tuple() {
        let rule = TableE11Rule {
            tag: "(0010,0010)".to_string(),
            attribute_name: "Patient's Name".to_string(),
            retired: false,
            in_std_comp_iod: true,
            basic_profile: ActionCode::Z,
            options: ProfileOptions::default(),
        };

        assert_eq!(rule.int_tuple().unwrap(), (0x0010, 0x0010));
        assert_eq!(TableE11Rule::normalize_tag("0010,0010").unwrap(), "(0010,0010)");
        assert_eq!(TableE11Rule::normalize_tag("0010 0010").unwrap(), "(0010,0010)");
        assert_eq!(TableE11Rule::normalize_tag("00100010").unwrap(), "(0010,0010)");
    }

    #[test]
    fn test_deidentification_config_serde_defaults() {
        let json = "{}";
        let config: DeidentificationConfig = serde_json::from_str(json).unwrap();
        assert!(!config.retain_uids);
        assert!(!config.clean_desc);
    }
}

