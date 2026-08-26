//! DICOM dataset assembly module.
//!
//! Reconstructs standard in-memory DICOM objects (`FileDicomObject<InMemDicomObject>`) from
//! exported DICOM JSON metadata (PS 3.18) and optional companion raw pixel data files (`.raw`).

use dicom_core::value::{PrimitiveValue, Value};
use dicom_core::{DataElement, Tag, VR};
use dicom_dictionary_std::StandardDataDictionary;
use dicom_object::{FileDicomObject, InMemDicomObject};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::error::TransformError;

/// Result of an assembly operation containing assembled datasets and statistics.
#[derive(Debug, Clone)]
pub struct AssemblyResult {
    /// Number of JSON files parsed and assembled.
    pub total_assembled: usize,
    /// Number of datasets that had raw pixel data attached.
    pub with_pixel_data: usize,
    /// Assembled in-memory DICOM file objects.
    pub objects: Vec<FileDicomObject<InMemDicomObject>>,
}

/// Assembler engine that reads DICOM JSON files and companion raw pixel data,
/// reconstructing complete in-memory DICOM objects.
pub struct DicomAssembler;

impl DicomAssembler {
    /// Assemble a single JSON file (and optional companion raw pixel file) into a `FileDicomObject`.
    pub fn assemble_file(
        json_path: &Path,
        raw_path: Option<&Path>,
    ) -> Result<FileDicomObject<InMemDicomObject>, TransformError> {
        if !json_path.exists() {
            return Err(TransformError::InvalidOperation(format!(
                "JSON input file does not exist: {}",
                json_path.display()
            )));
        }

        let file = File::open(json_path)?;
        let reader = BufReader::new(file);
        let json_value: serde_json::Value = serde_json::from_reader(reader)?;

        let mut inmem_obj: InMemDicomObject = dicom_json::from_value(json_value)
            .map_err(|e| TransformError::InvalidOperation(format!("Failed to parse DICOM JSON: {}", e)))?;

        // Locate and attach raw pixel data if available
        let resolved_raw = match raw_path {
            Some(p) if p.exists() => Some(p.to_path_buf()),
            _ => find_companion_raw_path(json_path),
        };

        if let Some(ref raw_p) = resolved_raw {
            if let Some(pixel_bytes) = read_raw_pixel_bytes(raw_p)? {
                let vr = determine_pixel_vr(&inmem_obj);
                inmem_obj.put(DataElement::new(
                    dicom_dictionary_std::tags::PIXEL_DATA,
                    vr,
                    Value::from(PrimitiveValue::U8(pixel_bytes.into())),
                ));
            }
        }

        // Build FileMetaTable
        let file_obj = create_file_dicom_object(inmem_obj)?;
        Ok(file_obj)
    }

    /// Assemble all DICOM JSON files found in a directory (and companion raw files).
    pub fn assemble_directory(
        dir_path: &Path,
        raw_dir: Option<&Path>,
    ) -> Result<AssemblyResult, TransformError> {
        if !dir_path.is_dir() {
            return Err(TransformError::InvalidOperation(format!(
                "Path is not a directory: {}",
                dir_path.display()
            )));
        }

        let mut json_files = Vec::new();
        for entry in std::fs::read_dir(dir_path)?.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("json") {
                        json_files.push(path);
                    }
                }
            }
        }
        json_files.sort();

        let mut objects = Vec::new();
        let mut with_pixel_data = 0;

        for json_file in &json_files {
            let specific_raw = raw_dir.and_then(|rd| {
                let stem = json_file.file_stem()?;
                let candidate = rd.join(format!("{}.raw", stem.to_string_lossy()));
                if candidate.exists() {
                    Some(candidate)
                } else {
                    let dir_candidate = rd.join(stem);
                    if dir_candidate.is_dir() {
                        Some(dir_candidate)
                    } else {
                        None
                    }
                }
            });

            let file_obj = Self::assemble_file(json_file, specific_raw.as_deref())?;
            if file_obj.element(dicom_dictionary_std::tags::PIXEL_DATA).is_ok() {
                with_pixel_data += 1;
            }
            objects.push(file_obj);
        }

        Ok(AssemblyResult {
            total_assembled: json_files.len(),
            with_pixel_data,
            objects,
        })
    }
}

/// Helper function to locate companion raw pixel file or directory for a JSON file.
fn find_companion_raw_path(json_path: &Path) -> Option<PathBuf> {
    // 1. Same stem with .raw (e.g. image.json -> image.raw)
    let raw_same_stem = json_path.with_extension("raw");
    if raw_same_stem.exists() {
        return Some(raw_same_stem);
    }

    // 2. Same stem directory (e.g. image.json -> image/0.raw)
    if let Some(parent) = json_path.parent() {
        if let Some(stem) = json_path.file_stem() {
            let dir_candidate = parent.join(stem);
            if dir_candidate.is_dir() {
                return Some(dir_candidate);
            }
        }
        // 3. Look for 0.raw in the same parent directory if parent has single JSON
        let zero_raw = parent.join("0.raw");
        if zero_raw.exists() {
            return Some(zero_raw);
        }
    }

    None
}

/// Reads raw pixel bytes from either a single `.raw` file or a directory of numbered frame files (`0.raw`, `1.raw`, ...).
fn read_raw_pixel_bytes(path: &Path) -> Result<Option<Vec<u8>>, TransformError> {
    if path.is_file() {
        let bytes = std::fs::read(path)?;
        Ok(Some(bytes))
    } else if path.is_dir() {
        let mut frame_files = Vec::new();
        for entry in std::fs::read_dir(path)?.flatten() {
            let f_path = entry.path();
            if f_path.is_file() {
                if let Some(ext) = f_path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("raw") || ext.eq_ignore_ascii_case("bin") {
                        let idx = f_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);
                        frame_files.push((idx, f_path));
                    }
                }
            }
        }
        frame_files.sort_by_key(|(idx, _)| *idx);

        if frame_files.is_empty() {
            return Ok(None);
        }

        let mut combined_bytes = Vec::new();
        for (_, f_path) in frame_files {
            let mut f_bytes = std::fs::read(&f_path)?;
            combined_bytes.append(&mut f_bytes);
        }
        Ok(Some(combined_bytes))
    } else {
        Ok(None)
    }
}

/// Determines whether PixelData VR should be OB or OW based on BitsAllocated.
fn determine_pixel_vr(obj: &InMemDicomObject) -> VR {
    if let Ok(elem) = obj.element(dicom_dictionary_std::tags::BITS_ALLOCATED) {
        if let Ok(bits) = elem.to_int::<u16>() {
            if bits > 8 {
                return VR::OW;
            } else {
                return VR::OB;
            }
        }
    }
    VR::OB
}

/// Creates a `FileDicomObject` from an `InMemDicomObject`, reconstructing proper FileMetaInformation.
pub fn create_file_dicom_object(
    inmem_obj: InMemDicomObject,
) -> Result<FileDicomObject<InMemDicomObject>, TransformError> {
    let sop_class_uid = inmem_obj
        .element(dicom_dictionary_std::tags::SOP_CLASS_UID)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1.2.840.10008.5.1.4.1.1.7".to_string()); // Secondary Capture Image Storage

    let sop_instance_uid = inmem_obj
        .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| format!("2.25.{}", uuid::Uuid::new_v4().as_u128()));

    let transfer_syntax_uid = inmem_obj
        .element(Tag(0x0002, 0x0010))
        .ok()
        .and_then(|e| e.to_str().ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN
                .uid()
                .to_string()
        });

    let meta = dicom_object::FileMetaTableBuilder::new()
        .media_storage_sop_class_uid(sop_class_uid)
        .media_storage_sop_instance_uid(sop_instance_uid)
        .transfer_syntax(transfer_syntax_uid)
        .build()
        .map_err(|e| TransformError::InvalidOperation(format!("Failed to build FileMetaTable: {}", e)))?;

    let mut file_obj =
        FileDicomObject::new_empty_with_dict_and_meta(StandardDataDictionary, meta);
    *file_obj = inmem_obj;
    Ok(file_obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Action;
    use crate::engine::DicomTransformer;
    use crate::dsl::TransformSpec;
    use dicom_core::value::PrimitiveValue;

    #[test]
    fn test_assemble_file_with_raw_pixels() {
        let temp_dir = tempfile::tempdir().unwrap();
        let json_path = temp_dir.path().join("test_export.json");
        let raw_path = temp_dir.path().join("test_export.raw");

        // 1. Create a source dataset with tags and pixel data
        let mut dataset = InMemDicomObject::new_empty();
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::PATIENT_NAME,
            VR::PN,
            Value::from("DOE^JANE"),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::PATIENT_ID,
            VR::LO,
            Value::from("PID-9999"),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::SOP_CLASS_UID,
            VR::UI,
            Value::from("1.2.840.10008.5.1.4.1.1.7"),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::SOP_INSTANCE_UID,
            VR::UI,
            Value::from("1.2.3.4.5.6.7.8.9"),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::ROWS,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![2].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::COLUMNS,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![2].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::BITS_ALLOCATED,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![8].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::BITS_STORED,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![8].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::HIGH_BIT,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![7].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::PIXEL_REPRESENTATION,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![0].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::SAMPLES_PER_PIXEL,
            VR::US,
            Value::from(PrimitiveValue::U16(vec![1].into())),
        ));
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::PHOTOMETRIC_INTERPRETATION,
            VR::CS,
            Value::from("MONOCHROME2"),
        ));

        let pixel_bytes = vec![11u8, 22u8, 33u8, 44u8];
        dataset.put(DataElement::new(
            dicom_dictionary_std::tags::PIXEL_DATA,
            VR::OB,
            Value::from(PrimitiveValue::U8(pixel_bytes.clone().into())),
        ));

        // 2. Export using SaveJson
        let mut spec = TransformSpec::new();
        spec.add_action(Action::SaveJson {
            json_location: json_path.to_string_lossy().to_string(),
            raw_pixel_location: Some(raw_path.to_string_lossy().to_string()),
        });
        let transformer = DicomTransformer::new(spec);
        transformer.transform_dataset(&mut dataset).unwrap();

        assert!(json_path.exists());
        assert!(raw_path.exists());

        // 3. Assemble back to DICOM object
        let assembled = DicomAssembler::assemble_file(&json_path, Some(&raw_path)).unwrap();

        assert_eq!(
            assembled.element(dicom_dictionary_std::tags::PATIENT_NAME).unwrap().to_str().unwrap(),
            "DOE^JANE"
        );
        assert_eq!(
            assembled.element(dicom_dictionary_std::tags::PATIENT_ID).unwrap().to_str().unwrap(),
            "PID-9999"
        );
        assert_eq!(
            assembled.element(dicom_dictionary_std::tags::SOP_INSTANCE_UID).unwrap().to_str().unwrap(),
            "1.2.3.4.5.6.7.8.9"
        );

        let assembled_pixel_elem = assembled.element(dicom_dictionary_std::tags::PIXEL_DATA).unwrap();
        let assembled_bytes = assembled_pixel_elem.to_bytes().unwrap();
        assert_eq!(&assembled_bytes[..], &pixel_bytes[..]);
    }

    #[test]
    fn test_assemble_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path();

        for i in 0..3 {
            let json_file = dir_path.join(format!("series_{}.json", i));
            let raw_file = dir_path.join(format!("series_{}.raw", i));

            let mut ds = InMemDicomObject::new_empty();
            ds.put(DataElement::new(
                dicom_dictionary_std::tags::PATIENT_NAME,
                VR::PN,
                Value::from(format!("PATIENT^{}", i)),
            ));
            ds.put(DataElement::new(
                dicom_dictionary_std::tags::SOP_INSTANCE_UID,
                VR::UI,
                Value::from(format!("1.2.3.4.5.{}", i)),
            ));

            let json_val = dicom_json::to_value(&ds).unwrap();
            std::fs::write(&json_file, serde_json::to_vec_pretty(&json_val).unwrap()).unwrap();
            std::fs::write(&raw_file, vec![i as u8; 4]).unwrap();
        }

        let result = DicomAssembler::assemble_directory(dir_path, None).unwrap();
        assert_eq!(result.total_assembled, 3);
        assert_eq!(result.with_pixel_data, 3);
        assert_eq!(result.objects.len(), 3);
    }
}

