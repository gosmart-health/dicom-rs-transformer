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

#[test]
fn test_dicom_test_files_integration() {
    use tempfile::tempdir;

    const PYDICOM_FILES: &[&str] = &[
        "pydicom/693_J2KI.dcm",
        "pydicom/CT_small.dcm",
        "pydicom/ExplVR_LitEndNoMeta.dcm",
        "pydicom/JPEG-LL.dcm",
        "pydicom/JPEG-lossy.dcm",
        "pydicom/JPEG2000.dcm",
        "pydicom/JPEG2000_UNC.dcm",
        "pydicom/JPGLosslessP14SV1_1s_1f_8b.dcm",
        "pydicom/MR_small.dcm",
        "pydicom/SC_rgb.dcm",
        "pydicom/SC_rgb_16bit.dcm",
        "pydicom/SC_rgb_2frame.dcm",
        "pydicom/SC_rgb_jpeg_dcmtk.dcm",
        "pydicom/SC_rgb_jpeg_gdcm.dcm",
        "pydicom/SC_rgb_jpeg_lossy_gdcm.dcm",
        "pydicom/SC_rgb_rle.dcm",
        "pydicom/SC_rgb_rle_16bit.dcm",
        "pydicom/SC_rgb_rle_2frame.dcm",
        "pydicom/color-px.dcm",
        "pydicom/color3d_jpeg_baseline.dcm",
        "pydicom/image_dfl.dcm",
        "pydicom/liver.dcm",
    ];

    let input_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    let mut expected_count = 0;
    for file_name in PYDICOM_FILES {
        let test_file_path = dicom_test_files::path(file_name)
            .unwrap_or_else(|e| panic!("dicom-test-files should download or locate {}: {:?}", file_name, e));

        let dest_filename = file_name.replace('/', "_");
        let dest_path = input_dir.path().join(&dest_filename);
        std::fs::copy(&test_file_path, &dest_path).unwrap();
        expected_count += 1;
    }

    let input_dir_str = input_dir.path().to_string_lossy().to_string();
    let output_dir_str = output_dir.path().to_string_lossy().to_string();

    let scanned_files = dicom_rs_transformer::scan_dicom_directory(&input_dir_str).unwrap();
    assert_eq!(scanned_files.len(), expected_count);

    let mut spec = TransformSpec::new();
    spec.add_action(Action::AnonymizePatient {
        patient_name: Some("ANON^PATIENT".to_string()),
        patient_id: Some("ANON-ID-1234".to_string()),
    });
    spec.add_action(Action::SaveDataset {
        location: output_dir_str.clone(),
    });

    for file_path in &scanned_files {
        let file_path_str = file_path.to_string_lossy().to_string();

        let mut file_spec = TransformSpec::new();
        file_spec.add_action(Action::LoadDataset {
            location: file_path_str,
        });
        file_spec.add_action(Action::AnonymizePatient {
            patient_name: Some("ANON^PATIENT".to_string()),
            patient_id: Some("ANON-ID-1234".to_string()),
        });
        file_spec.add_action(Action::SaveDataset {
            location: output_dir_str.clone(),
        });

        let file_transformer = DicomTransformer::new(file_spec);
        let mut obj = InMemDicomObject::new_empty();

        let report = file_transformer.transform_dataset(&mut obj).unwrap();
        assert_eq!(report.actions_executed, 3);
    }

    let output_files = dicom_rs_transformer::scan_dicom_directory(&output_dir_str).unwrap();
    assert_eq!(output_files.len(), scanned_files.len());

    for out_file in &output_files {
        let out_obj = dicom_rs_transformer::io::load_dicom_object(&out_file.to_string_lossy()).unwrap();

        let name_elem = out_obj.element(Tag(0x0010, 0x0010)).unwrap();
        assert_eq!(name_elem.to_str().unwrap(), "ANON^PATIENT");

        let id_elem = out_obj.element(Tag(0x0010, 0x0020)).unwrap();
        assert_eq!(id_elem.to_str().unwrap(), "ANON-ID-1234");
    }
}



