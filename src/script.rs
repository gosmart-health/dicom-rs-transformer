//! Line-by-line text script parser for constructing DICOM transformation specs.

use std::io::BufRead;
use std::str::FromStr;

use crate::dsl::{Action, TagSelector, TransformSpec};
use crate::error::TransformError;

/// Parser for transforming line-by-line script commands into `TransformSpec` models.
#[derive(Debug, Default, Clone)]
pub struct ScriptParser;

impl ScriptParser {
    /// Creates a new `ScriptParser`.
    pub fn new() -> Self {
        Self
    }

    /// Parses a single line of script command into an optional `Action`.
    /// Returns `Ok(None)` for comments or blank lines.
    ///
    /// # Errors
    ///
    /// Returns `TransformError::ScriptParse` if line syntax is invalid.
    pub fn parse_line(
        &self,
        line_num: usize,
        line: &str,
    ) -> Result<Option<Action>, TransformError> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            return Ok(None);
        }

        let tokens = tokenize(trimmed);
        if tokens.is_empty() {
            return Ok(None);
        }

        let command = tokens[0].to_uppercase();
        match command.as_str() {
            "HELP" | "COMMANDS" => Err(TransformError::InvalidOperation(
                "Available commands: LOAD <path/uri>, SAVE <path/uri>, SAVE_MAP <path/uri>, SAVE_JSON <json_uri> [<raw_uri>], ASSEMBLE <input_uri> [RAW=\"<raw>\"] [OUT=\"<out>\"], DUMP <path/uri>, SET <tag> <value>, GENERATE_UID <tag> [FROM <source>], DELETE <tag>, REPLACE <tag> <pattern> WITH <replacement>, ANONYMIZE NAME=\"<name>\" ID=\"<id>\", EXECUTE".to_string()
            )),
            "EXECUTE" | "RUN_BATCH" | "APPLY" => {
                Ok(Some(Action::Execute))
            }
            "ASSEMBLE" | "REASSEMBLE" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "ASSEMBLE command requires input JSON location URI (e.g. ASSEMBLE \"input.json\" [RAW=\"input.raw\"] [OUT=\"output.dcm\"])".to_string(),
                    });
                }
                let input_location = tokens[1].clone();
                let mut raw_location = None;
                let mut output_location = None;
                let mut pacs_destination = None;

                for token in &tokens[2..] {
                    if let Some((k, v)) = token.split_once('=') {
                        match k.to_uppercase().as_str() {
                            "RAW" | "RAW_LOCATION" | "PIXELS" => raw_location = Some(v.trim_matches('"').to_string()),
                            "OUT" | "OUTPUT" | "DEST" | "DESTINATION" => output_location = Some(v.trim_matches('"').to_string()),
                            "PACS" | "PUSH" => pacs_destination = Some(v.trim_matches('"').to_string()),
                            _ => {}
                        }
                    } else if raw_location.is_none() {
                        raw_location = Some(token.clone());
                    } else if output_location.is_none() {
                        output_location = Some(token.clone());
                    }
                }

                Ok(Some(Action::Assemble {
                    input_location,
                    raw_location,
                    output_location,
                    pacs_destination,
                }))
            }
            "LOAD" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "LOAD command requires location URI (e.g. LOAD \"s3://bucket/input.dcm\")".to_string(),
                    });
                }
                let location = tokens[1].clone();
                Ok(Some(Action::LoadDataset { location }))
            }
            "SAVE" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "SAVE command requires location URI (e.g. SAVE \"gs://bucket/output.dcm\")".to_string(),
                    });
                }
                let location = tokens[1].clone();
                Ok(Some(Action::SaveDataset { location }))
            }
            "SAVE_MAP" | "EXPORT_MAP" | "MAP_SAVE" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "SAVE_MAP command requires location URI (e.g. SAVE_MAP \"s3://audit/map.json\")".to_string(),
                    });
                }
                let location = tokens[1].clone();
                Ok(Some(Action::SaveMap { location }))
            }
            "EXTRACT_PIXELS" | "EXTRACT_IMAGES" | "EXTRACT_FRAMES" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "EXTRACT_PIXELS command requires destination URI (e.g. EXTRACT_PIXELS \"s3://bucket/images/\" FORMAT=\"jpeg\")".to_string(),
                    });
                }
                let destination = tokens[1].clone();
                let mut format = "jpeg".to_string();
                for token in &tokens[2..] {
                    if let Some((k, v)) = token.split_once('=') {
                        if k.eq_ignore_ascii_case("FORMAT") || k.eq_ignore_ascii_case("TYPE") {
                            format = v.trim_matches('"').to_string();
                        }
                    } else if !token.contains('=') {
                        format = token.trim_matches('"').to_string();
                    }
                }
                Ok(Some(Action::ExtractPixels { destination, format }))
            }
            "SET" => {
                if tokens.len() < 3 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "SET command requires tag selector and value (e.g. SET PatientName \"ANONYMOUS\")".to_string(),
                    });
                }
                let selector = TagSelector::from_str(&tokens[1])?;
                // Skip optional "=" if present
                let value_idx = if tokens[2] == "=" {
                    if tokens.len() < 4 {
                        return Err(TransformError::ScriptParse {
                            line: line_num,
                            message: "SET command missing value after '='".to_string(),
                        });
                    }
                    3
                } else {
                    2
                };
                let value = tokens[value_idx].clone();
                Ok(Some(Action::SetTag { selector, value }))
            }
            "GENERATE_UID" | "NEW_UID" | "GEN_UID" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "GENERATE_UID command requires tag selector (e.g. GENERATE_UID StudyInstanceUID [FROM <source>])".to_string(),
                    });
                }
                let selector = TagSelector::from_str(&tokens[1])?;
                let source = if tokens.len() >= 4 && tokens[2].to_uppercase() == "FROM" {
                    Some(tokens[3].clone())
                } else if tokens.len() >= 3 && tokens[2] != "=" {
                    Some(tokens[2].clone())
                } else {
                    None
                };
                Ok(Some(Action::GenerateUid { selector, source }))
            }
            "REMOVE" | "DELETE" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "REMOVE/DELETE command requires tag selector (e.g. REMOVE PatientAddress)".to_string(),
                    });
                }
                let selector = TagSelector::from_str(&tokens[1])?;
                Ok(Some(Action::RemoveTag { selector }))
            }
            "REPLACE" => {
                if tokens.len() < 4 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "REPLACE command format: REPLACE <tag> \"<pattern>\" WITH \"<replacement>\"".to_string(),
                    });
                }
                let selector = TagSelector::from_str(&tokens[1])?;
                let pattern = tokens[2].clone();
                let replacement = if tokens.len() >= 5 && tokens[3].to_uppercase() == "WITH" {
                    tokens[4].clone()
                } else {
                    tokens[3].clone()
                };
                Ok(Some(Action::ReplaceValue {
                    selector,
                    pattern,
                    replacement,
                }))
            }
            "SAVE_JSON" | "EXPORT_JSON" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "SAVE_JSON command requires json_file location URI (e.g. SAVE_JSON \"output.json\" [\"output.raw\"])".to_string(),
                    });
                }
                let json_location = tokens[1].clone();
                let raw_pixel_location = if tokens.len() >= 3 {
                    Some(tokens[2].clone())
                } else {
                    None
                };
                Ok(Some(Action::SaveJson {
                    json_location,
                    raw_pixel_location,
                }))
            }
            "DUMP" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "DUMP command requires location URI (e.g. DUMP \"dump.txt\")".to_string(),
                    });
                }
                let location = tokens[1].clone();
                Ok(Some(Action::Dump { location }))
            }
            "ANONYMIZE" => {
                let mut patient_name = None;
                let mut patient_id = None;
                for token in &tokens[1..] {
                    if let Some((k, v)) = token.split_once('=') {
                        match k.to_uppercase().as_str() {
                            "NAME" | "PATIENTNAME" => patient_name = Some(v.trim_matches('"').to_string()),
                            "ID" | "PATIENTID" => patient_id = Some(v.trim_matches('"').to_string()),
                            _ => {}
                        }
                    }
                }
                Ok(Some(Action::AnonymizePatient {
                    patient_name,
                    patient_id,
                }))
            }
            "CHECK" => {
                if tokens.len() < 3 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: "CHECK command format: CHECK <tag> <op> [<arg1> <arg2>]".to_string(),
                    });
                }
                let selector = TagSelector::from_str(&tokens[1])?;
                let check_op = tokens[2].to_uppercase();
                let args = tokens[3..].to_vec();
                Ok(Some(Action::Check { selector, check_op, args }))
            }
            "AND" | "OR" | "XOR" | "NOT" | "DUP" | "DROP" | "CLEAR" => {
                Ok(Some(Action::LogicOp { logic_op: command }))
            }
            "IF_TRUE" | "IF_FALSE" => {
                if tokens.len() < 2 {
                    return Err(TransformError::ScriptParse {
                        line: line_num,
                        message: format!("{} command requires script location URI", command),
                    });
                }
                let condition = command == "IF_TRUE";
                let script_location = tokens[1].clone();
                Ok(Some(Action::IfBranch {
                    condition,
                    script_location,
                }))
            }
            _ => Err(TransformError::ScriptParse {
                line: line_num,
                message: format!("Unknown script command '{}'", command),
            }),
        }
    }

    /// Parses a complete script from any `BufRead` source into a `TransformSpec`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if any line fails syntax validation or I/O fails.
    pub fn parse_script<R: BufRead>(&self, reader: R) -> Result<TransformSpec, TransformError> {
        let mut spec = TransformSpec::new();
        spec.name = Some("Script Transformation".to_string());

        for (idx, line_res) in reader.lines().enumerate() {
            let line = line_res?;
            if let Some(action) = self.parse_line(idx + 1, &line)? {
                spec.add_action(action);
            }
        }

        Ok(spec)
    }
}

/// Simple whitespace and quoted string tokenizer.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            '\\' if in_quotes => {
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_script_lines() {
        let parser = ScriptParser::new();

        // SET line
        let action1 = parser
            .parse_line(1, "SET PatientName = \"ANON^DOE\"")
            .unwrap()
            .unwrap();
        assert_eq!(
            action1,
            Action::SetTag {
                selector: TagSelector::Keyword("PatientName".to_string()),
                value: "ANON^DOE".to_string(),
            }
        );

        // REMOVE line
        let action2 = parser
            .parse_line(2, "DELETE PatientAddress")
            .unwrap()
            .unwrap();
        assert_eq!(
            action2,
            Action::RemoveTag {
                selector: TagSelector::Keyword("PatientAddress".to_string()),
            }
        );

        // REPLACE line
        let action3 = parser
            .parse_line(3, "REPLACE StudyDescription \"HOSPITAL_A\" WITH \"SITE_1\"")
            .unwrap()
            .unwrap();
        assert_eq!(
            action3,
            Action::ReplaceValue {
                selector: TagSelector::Keyword("StudyDescription".to_string()),
                pattern: "HOSPITAL_A".to_string(),
                replacement: "SITE_1".to_string(),
            }
        );

        // SAVE_JSON lines
        let action4 = parser
            .parse_line(4, "SAVE_JSON \"output.json\"")
            .unwrap()
            .unwrap();
        assert_eq!(
            action4,
            Action::SaveJson {
                json_location: "output.json".to_string(),
                raw_pixel_location: None,
            }
        );

        let action5 = parser
            .parse_line(5, "SAVE_JSON \"output.json\" \"output.raw\"")
            .unwrap()
            .unwrap();
        assert_eq!(
            action5,
            Action::SaveJson {
                json_location: "output.json".to_string(),
                raw_pixel_location: Some("output.raw".to_string()),
            }
        );

        // DUMP line
        let action6 = parser
            .parse_line(6, "DUMP \"dump.txt\"")
            .unwrap()
            .unwrap();
        assert_eq!(
            action6,
            Action::Dump {
                location: "dump.txt".to_string(),
            }
        );
    }
}
