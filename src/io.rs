//! Local filesystem I/O module for DICOM datasets (Community Edition).
//!
//! Supports local Unix/Windows file paths and `file://` URIs.
//! Cloud and network storage URIs (`s3://`, `gs://`, `az://`, `dicom://`, `dicoms://`)
//! require the PRO edition of dicom-rs-transformer.

use dicom_object::from_reader;
use dicom_object::{FileDicomObject, InMemDicomObject};
use std::io::Cursor;
use std::path::Path as LocalPath;

use crate::error::TransformError;

/// Reads raw bytes asynchronously from a local file path.
pub async fn read_bytes_async(location: &str) -> Result<Vec<u8>, TransformError> {
    check_cloud_uri(location)?;
    let clean_path = strip_file_prefix(location);
    let bytes = std::fs::read(clean_path)?;
    Ok(bytes)
}

/// Writes raw bytes asynchronously to a local file path.
pub async fn write_bytes_async(location: &str, data: Vec<u8>) -> Result<(), TransformError> {
    check_cloud_uri(location)?;
    let clean_path = strip_file_prefix(location);
    if let Some(parent) = LocalPath::new(clean_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(clean_path, data)?;
    Ok(())
}

/// Reads raw bytes synchronously from a local file path.
pub fn read_bytes(location: &str) -> Result<Vec<u8>, TransformError> {
    check_cloud_uri(location)?;
    let clean_path = strip_file_prefix(location);
    let bytes = std::fs::read(clean_path)?;
    Ok(bytes)
}

/// Writes raw bytes synchronously to a local file path.
pub fn write_bytes(location: &str, data: &[u8]) -> Result<(), TransformError> {
    check_cloud_uri(location)?;
    let clean_path = strip_file_prefix(location);
    if let Some(parent) = LocalPath::new(clean_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(clean_path, data)?;
    Ok(())
}

/// Reads and parses a DICOM dataset object from a local file path.
pub fn load_dicom_object(
    location: &str,
) -> Result<FileDicomObject<InMemDicomObject>, TransformError> {
    let bytes = read_bytes(location)?;
    let cursor = Cursor::new(bytes);
    let obj = from_reader(cursor)?;
    Ok(obj)
}

/// Serializes and writes a DICOM dataset object to a local file path.
pub fn save_dicom_object(
    location: &str,
    obj: &FileDicomObject<InMemDicomObject>,
) -> Result<(), TransformError> {
    let mut buffer = Vec::new();
    obj.write_all(&mut buffer)?;
    write_bytes(location, &buffer)?;
    Ok(())
}

fn check_cloud_uri(location: &str) -> Result<(), TransformError> {
    let cloud_schemes = [
        "s3://", "gs://", "gcs://", "az://", "abfs://",
        "http://", "https://", "dicom://", "dicoms://",
    ];

    for scheme in cloud_schemes {
        if location.starts_with(scheme) {
            return Err(TransformError::ProFeatureRequired(format!(
                "Cloud storage and network protocol URIs ('{}') are PRO features (s3://, gs://, az://, dicom://, dicoms://). Community edition supports local filesystem paths only ('file://' or standard file paths). Please upgrade to dicom-rs-transformer-pro for cloud I/O and network transfer.",
                location
            )));
        }
    }

    Ok(())
}

fn strip_file_prefix(location: &str) -> &str {
    if let Some(stripped) = location.strip_prefix("file://") {
        stripped
    } else {
        location
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_output.dcm");
        let path_str = file_path.to_str().unwrap();

        let data = b"HEADER_MOCK_DICOM_BYTES";
        write_bytes(path_str, data).unwrap();

        let read_back = read_bytes(path_str).unwrap();
        assert_eq!(read_back, data);
    }

    #[test]
    fn test_cloud_uri_rejection() {
        let test_uris = [
            "s3://my-bucket/test.dcm",
            "gs://my-bucket/test.dcm",
            "az://my-container/test.dcm",
            "dicom://pacs.hospital.org:104/STUDY",
            "dicoms://securepacs.hospital.org:2762/STUDY",
        ];

        for uri in test_uris {
            let err = read_bytes(uri).unwrap_err();
            match err {
                TransformError::ProFeatureRequired(msg) => {
                    assert!(msg.contains("PRO features"));
                    assert!(msg.contains("dicom-rs-transformer-pro"));
                }
                _ => panic!("Expected ProFeatureRequired error for URI: {}", uri),
            }
        }
    }
}
