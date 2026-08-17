//! Sample program demonstrating how external developers use `dicom-rs-transformer`.

use dicom_core::Tag;
use dicom_object::InMemDicomObject;
use dicom_rs_transformer::{DicomTransformer, ScriptParser};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== dicom-rs-transformer API Usage Example ===");

    // 1. Parse a script
    let script = r#"
SET PatientName "SMITH^JOHN"
SET PatientID "ID-9001"
REPLACE PatientName "JOHN" WITH "ALEX"
"#;
    let parser = ScriptParser::new();
    let spec = parser.parse_script(Cursor::new(script))?;

    println!(
        "Parsed script into {} transformation actions.",
        spec.actions.len()
    );

    // 2. Create in-memory DICOM dataset
    let mut dataset = InMemDicomObject::new_empty();

    // 3. Execute transformer
    let transformer = DicomTransformer::new(spec);
    let report = transformer.transform_dataset(&mut dataset)?;

    println!(
        "Report: Executed {} actions ({} tags modified) in {}ms",
        report.actions_executed, report.tags_modified, report.duration_ms
    );

    let name = dataset.element(Tag(0x0010, 0x0010))?.to_str()?;
    println!("Final PatientName: {}", name);

    Ok(())
}
