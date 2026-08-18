use dicom_core::Tag;
use dicom_object::InMemDicomObject;
use dicom_rs_transformer::{Action, DicomTransformer, ScriptParser, TagSelector, TransformSpec, TransformError};
use std::io::Cursor;

#[test]
fn test_full_transformation_pipeline() {
    let mut dataset = InMemDicomObject::new_empty();

    let mut spec = TransformSpec::new();
    spec.add_action(Action::SetTag {
        selector: TagSelector::Keyword("PatientName".to_string()),
        value: "DOE^JANE".to_string(),
    });
    spec.add_action(Action::SetTag {
        selector: TagSelector::Keyword("PatientID".to_string()),
        value: "998877".to_string(),
    });
    spec.add_action(Action::ReplaceValue {
        selector: TagSelector::Keyword("PatientName".to_string()),
        pattern: "JANE".to_string(),
        replacement: "ALICE".to_string(),
    });
    spec.add_action(Action::RemoveTag {
        selector: TagSelector::Keyword("PatientID".to_string()),
    });

    let transformer = DicomTransformer::new(spec);
    let report = transformer.transform_dataset(&mut dataset).unwrap();

    assert_eq!(report.actions_executed, 4);
    assert_eq!(report.tags_modified, 3);
    assert_eq!(report.tags_removed, 1);

    let name_elem = dataset.element(Tag(0x0010, 0x0010)).unwrap();
    assert_eq!(name_elem.to_str().unwrap(), "DOE^ALICE");
    assert!(dataset.element(Tag(0x0010, 0x0020)).is_err());
}

#[test]
fn test_script_parser_and_execution() {
    let script_text = r#"
# Anonymization script test
SET PatientName "ANON^PERSON"
SET PatientID "ANON-001"
REPLACE PatientName "PERSON" WITH "USER"
REMOVE PatientAddress
"#;

    let parser = ScriptParser::new();
    let spec = parser.parse_script(Cursor::new(script_text)).unwrap();

    assert_eq!(spec.actions.len(), 4);

    let mut dataset = InMemDicomObject::new_empty();
    let transformer = DicomTransformer::new(spec);
    let report = transformer.transform_dataset(&mut dataset).unwrap();

    assert_eq!(report.actions_executed, 4);
    let name_elem = dataset.element(Tag(0x0010, 0x0010)).unwrap();
    assert_eq!(name_elem.to_str().unwrap(), "ANON^USER");
}

#[test]
fn test_sequence_transformation_script_and_json() {
    let script_text = r#"
# Sequence element handling test script
SET RequestAttributesSequence[0]/ScheduledProcedureStepID "PROC-9001"
"#;

    let parser = ScriptParser::new();
    let spec = parser.parse_script(Cursor::new(script_text)).unwrap();

    let mut dataset = InMemDicomObject::new_empty();
    let transformer = DicomTransformer::new(spec);
    let err = transformer.transform_dataset(&mut dataset).unwrap_err();

    match err {
        TransformError::ProFeatureRequired(msg) => {
            assert!(msg.contains("Nested DICOM Sequence path"));
            assert!(msg.contains("dicom-rs-transformer-pro"));
        }
        _ => panic!("Expected ProFeatureRequired error for sequence transformation script"),
    }
}
