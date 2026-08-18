use dicom_core::Tag;
use dicom_object::{FileDicomObject, InMemDicomObject};
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

#[test]
fn test_generate_uid_script() {
    let script_text = r#"
# Generate random and deterministic UIDs
GENERATE_UID StudyInstanceUID
SET SOPInstanceUID "1.2.840.10008.1.2.3.4"
GENERATE_UID SeriesInstanceUID FROM SOPInstanceUID
"#;

    let parser = ScriptParser::new();
    let spec = parser.parse_script(Cursor::new(script_text)).unwrap();

    let mut dataset = InMemDicomObject::new_empty();
    let transformer = DicomTransformer::new(spec);
    let report = transformer.transform_dataset(&mut dataset).unwrap();

    assert_eq!(report.actions_executed, 3);

    let study_uid = dataset.element(Tag(0x0020, 0x000D)).unwrap().to_str().unwrap().to_string();
    assert!(study_uid.starts_with("2.25."));

    let series_uid_1 = dataset.element(Tag(0x0020, 0x000E)).unwrap().to_str().unwrap().to_string();
    assert!(series_uid_1.starts_with("2.25."));

    // Verify determinism: same input seed SOPInstanceUID yields same derived SeriesInstanceUID
    let spec2 = parser.parse_script(Cursor::new(script_text)).unwrap();
    let mut dataset2 = InMemDicomObject::new_empty();
    let transformer2 = DicomTransformer::new(spec2);
    let _ = transformer2.transform_dataset(&mut dataset2).unwrap();

    let series_uid_2 = dataset2.element(Tag(0x0020, 0x000E)).unwrap().to_str().unwrap().to_string();
    assert_eq!(series_uid_1, series_uid_2);
}

#[test]
fn test_directory_batch_scan_and_execute() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let file1_path = dir.path().join("test1.dcm");
    let file2_path = dir.path().join("test2.dcm");

    let meta = dicom_object::FileMetaTableBuilder::new()
        .media_storage_sop_instance_uid("2.25.1001")
        .transfer_syntax("1.2.840.10008.1.2.1")
        .build()
        .unwrap();

    let dcm1 = FileDicomObject::new_empty_with_dict_and_meta(
        dicom_dictionary_std::StandardDataDictionary,
        meta.clone(),
    );
    let dcm2 = FileDicomObject::new_empty_with_dict_and_meta(
        dicom_dictionary_std::StandardDataDictionary,
        meta,
    );

    dcm1.write_to_file(&file1_path).unwrap();
    dcm2.write_to_file(&file2_path).unwrap();

    let dicom_files = dicom_rs_transformer::scan_dicom_directory(&dir.path().to_string_lossy()).unwrap();
    assert_eq!(dicom_files.len(), 2);

    let script_text = "EXECUTE";
    let parser = ScriptParser::new();
    let spec = parser.parse_script(Cursor::new(script_text)).unwrap();

    assert_eq!(spec.actions.len(), 1);
    assert_eq!(spec.actions[0], Action::Execute);
}


