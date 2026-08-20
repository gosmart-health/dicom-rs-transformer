//! Pixel data extraction module for converting DICOM frames into JPEG, PNG, or RAW binary files.

use dicom_dictionary_std::StandardDataDictionary;
use dicom_object::{FileDicomObject, InMemDicomObject};
use dicom_pixeldata::PixelDecoder;
use image::ImageFormat;
use std::io::Cursor;
use std::str::FromStr;

use crate::error::TransformError;

/// Supported image export formats for extracted DICOM frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelExportFormat {
    /// JPEG compressed image (.jpg)
    Jpeg,
    /// PNG lossless compressed image (.png)
    Png,
    /// Uncompressed raw binary pixel data (.raw)
    Raw,
}

impl PixelExportFormat {
    /// Returns the standard file extension for the format.
    pub fn extension(&self) -> &'static str {
        match self {
            PixelExportFormat::Jpeg => "jpg",
            PixelExportFormat::Png => "png",
            PixelExportFormat::Raw => "raw",
        }
    }
}

impl FromStr for PixelExportFormat {
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(PixelExportFormat::Jpeg),
            "png" => Ok(PixelExportFormat::Png),
            "raw" | "bin" | "bytes" => Ok(PixelExportFormat::Raw),
            _ => Err(TransformError::InvalidOperation(format!(
                "Unsupported pixel export format '{}'. Expected 'jpeg', 'png', or 'raw'",
                s
            ))),
        }
    }
}

/// Extracts pixel frames from a DICOM object and saves them into the destination directory/URI
/// using numbered filename convention (`0.jpg`, `1.jpg`, `2.jpg`, ...).
///
/// Returns the number of frames extracted and saved.
pub fn extract_pixel_frames(
    obj: &InMemDicomObject,
    destination_uri: &str,
    format: PixelExportFormat,
) -> Result<usize, TransformError> {
    if obj.element(dicom_dictionary_std::tags::PIXEL_DATA).is_err() {
        return Ok(0);
    }
    let meta = dicom_object::FileMetaTableBuilder::new()
        .transfer_syntax("1.2.840.10008.1.2.1")
        .media_storage_sop_class_uid("1.2.840.10008.5.1.4.1.1.7")
        .media_storage_sop_instance_uid("1.2.840.10008.5.1.4.1.1.7.1")
        .build()
        .map_err(|e| TransformError::InvalidOperation(e.to_string()))?;
    let empty_file_obj =
        FileDicomObject::new_empty_with_dict_and_meta(StandardDataDictionary, meta);

    let mut buf = Vec::new();
    empty_file_obj.write_all(&mut buf)?;
    obj.write_dataset_with_ts(
        &mut buf,
        &dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.erased(),
    )?;

    let file_obj = FileDicomObject::from_reader(Cursor::new(&buf))
        .map_err(|e| TransformError::InvalidOperation(e.to_string()))?;

    let decoded = file_obj.decode_pixel_data().map_err(|e| {
        TransformError::InvalidOperation(format!("Failed to decode DICOM pixel data: {}", e))
    })?;

    let number_of_frames = decoded.number_of_frames() as usize;

    if number_of_frames == 0 {
        return Ok(0);
    }

    let dest = destination_uri.trim_end_matches('/');

    for frame_idx in 0..number_of_frames {
        let frame_location = format!("{}/{}.{}", dest, frame_idx, format.extension());

        match format {
            PixelExportFormat::Jpeg | PixelExportFormat::Png => {
                let options = dicom_pixeldata::ConvertOptions::new();
                let dynamic_img = match decoded.to_dynamic_image_with_options(frame_idx as u32, &options) {
                    Ok(img) => img,
                    Err(_) => continue,
                };

                let img_format = match format {
                    PixelExportFormat::Jpeg => ImageFormat::Jpeg,
                    PixelExportFormat::Png => ImageFormat::Png,
                    _ => unreachable!(),
                };

                let mut img_buf = Vec::new();
                dynamic_img
                    .write_to(&mut Cursor::new(&mut img_buf), img_format)
                    .map_err(|e| {
                        TransformError::InvalidOperation(format!(
                            "Failed to encode frame {} as {}: {}",
                            frame_idx,
                            format.extension(),
                            e
                        ))
                    })?;

                crate::io::write_bytes(&frame_location, &img_buf)?;
            }
            PixelExportFormat::Raw => {
                let raw_bytes = decoded.data();
                let frame_size = raw_bytes.len() / number_of_frames;
                let start = frame_idx * frame_size;
                let end = start + frame_size;

                if end <= raw_bytes.len() {
                    let frame_data = &raw_bytes[start..end];
                    crate::io::write_bytes(&frame_location, frame_data)?;
                } else {
                    crate::io::write_bytes(&frame_location, raw_bytes)?;
                }
            }
        }
    }

    Ok(number_of_frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dicom_core::value::Value;
    use dicom_core::{DataElement, Tag, VR};

    #[test]
    fn test_extract_pixel_frames_raw() {
        use dicom_core::value::PrimitiveValue;

        let mut dataset = InMemDicomObject::new_empty();
        dataset.put(DataElement::new(
            Tag(0x0002, 0x0010),
            VR::UI,
            Value::from("1.2.840.10008.1.2.1"),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0010),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![2].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0011),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![2].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0100),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![8].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0101),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![8].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0102),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![7].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0103),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![0].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0002),
            VR::US,
            Value::from(PrimitiveValue::U16(vec![1].into())),
        ));
        dataset.put(DataElement::new(
            Tag(0x0028, 0x0004),
            VR::CS,
            Value::from("MONOCHROME2"),
        ));

        let pixel_bytes: Vec<u8> = vec![10, 20, 30, 40];
        dataset.put(DataElement::new(
            Tag(0x7FE0, 0x0010),
            VR::OB,
            Value::from(PrimitiveValue::U8(pixel_bytes.into())),
        ));

        let temp_dir = std::env::temp_dir().join("dicom_pixel_test_raw");
        let count = extract_pixel_frames(
            &dataset,
            &temp_dir.to_string_lossy(),
            PixelExportFormat::Raw,
        )
        .unwrap();
        assert_eq!(count, 1);
        let out_file = temp_dir.join("0.raw");
        assert!(out_file.exists());
        let read_bytes = std::fs::read(out_file).unwrap();
        assert_eq!(read_bytes, vec![10, 20, 30, 40]);
    }
}
