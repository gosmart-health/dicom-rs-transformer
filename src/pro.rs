//! Developer extension traits and hooks for PRO feature integration.
//!
//! Community Edition provides default trait implementations that return friendly,
//! explicit error messages pointing developers to `dicom-rs-transformer-pro` or allowing
//! custom extensions.

use crate::dsl::TagPath;
use crate::error::TransformError;
use dicom_object::{FileDicomObject, InMemDicomObject};

/// Extension trait for cloud and network storage I/O operations (`s3://`, `gs://`, `az://`, `dicom://`, `dicoms://`).
pub trait CloudStorageHandler {
    /// Reads dataset bytes from a cloud URI synchronously.
    fn read_cloud_bytes(&self, uri: &str) -> Result<Vec<u8>, TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "Cloud storage and network protocol URIs ('{}') are PRO features (s3://, gs://, az://, dicom://, dicoms://). Community edition supports local filesystem paths only ('file://' or standard file paths). Please upgrade to dicom-rs-transformer-pro for cloud I/O and network transfer.",
            uri
        )))
    }

    /// Writes dataset bytes to a cloud URI synchronously.
    fn write_cloud_bytes(&self, uri: &str, _data: &[u8]) -> Result<(), TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "Cloud storage and network protocol URIs ('{}') are PRO features (s3://, gs://, az://, dicom://, dicoms://). Community edition supports local filesystem paths only ('file://' or standard file paths). Please upgrade to dicom-rs-transformer-pro for cloud I/O and network transfer.",
            uri
        )))
    }

    /// Loads a DICOM dataset from a cloud URI.
    fn load_cloud_object(
        &self,
        uri: &str,
    ) -> Result<FileDicomObject<InMemDicomObject>, TransformError> {
        let bytes = self.read_cloud_bytes(uri)?;
        let cursor = std::io::Cursor::new(bytes);
        let obj = dicom_object::from_reader(cursor)?;
        Ok(obj)
    }

    /// Saves a DICOM dataset to a cloud URI.
    fn save_cloud_object(
        &self,
        uri: &str,
        obj: &FileDicomObject<InMemDicomObject>,
    ) -> Result<(), TransformError> {
        let mut buffer = Vec::new();
        obj.write_all(&mut buffer)?;
        self.write_cloud_bytes(uri, &buffer)
    }
}

/// Extension trait for nested DICOM sequence hierarchy path evaluation.
pub trait SequencePathEvaluator {
    /// Evaluates and parses nested sequence tag paths containing `/` or `[...]`.
    fn evaluate_sequence_path(&self, raw_path: &str) -> Result<TagPath, TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "Nested DICOM Sequence path '{}' is a PRO feature. Community edition supports top-level tags only. Please upgrade to dicom-rs-transformer-pro for DICOM Sequence path evaluation.",
            raw_path
        )))
    }
}

/// Extension trait for conditional evaluation, RPN boolean logic, and sub-script branching.
pub trait LogicStackEvaluator {
    /// Evaluates conditional logic and sub-script execution.
    fn evaluate_logic_action(&self, action_name: &str) -> Result<(), TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "Conditional logic execution and RPN stack operation '{}' is a PRO feature. Community edition supports basic transformations only. Please upgrade to dicom-rs-transformer-pro for conditional logic and script caching.",
            action_name
        )))
    }
}

/// Extension trait for DICOM PACS C-STORE push and network DIMSE transfer (`dicom://`, `dicoms://`).
pub trait PacsPushHandler {
    /// Pushes an assembled DICOM object to a remote PACS / C-STORE destination.
    fn push_pacs(
        &self,
        pacs_uri: &str,
        _obj: &FileDicomObject<InMemDicomObject>,
    ) -> Result<(), TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "Direct PACS C-STORE network push ('{}') is a PRO feature. Community edition supports saving assembled DICOM files to local disk. Please upgrade to dicom-rs-transformer-pro for PACS networking and C-STORE transfer.",
            pacs_uri
        )))
    }
}

/// Extension trait for DICOM DIMSE query/retrieve (C-FIND, C-MOVE) and AE Title network push (C-STORE).
pub trait DimseHandler {
    /// Queries and retrieves DICOM datasets using combined C-FIND and C-MOVE from a source AE to a destination AE.
    fn fetch_datasets(
        &self,
        _filters: &std::collections::HashMap<String, String>,
        from_ae: &str,
        to_ae: &str,
    ) -> Result<Vec<FileDicomObject<InMemDicomObject>>, TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "DIMSE C-FIND and C-MOVE 'fetch' operation (from_ae: '{}', to_ae: '{}') is a PRO feature. Community edition supports local filesystem operations only. Please upgrade to dicom-rs-transformer-pro for PACS DIMSE query/retrieve and AE Title routing.",
            from_ae, to_ae
        )))
    }

    /// Pushes a DICOM dataset to a remote DIMSE Application Entity (AE) Title destination.
    fn push_dataset(
        &self,
        to_ae: &str,
        _obj: &FileDicomObject<InMemDicomObject>,
    ) -> Result<(), TransformError> {
        Err(TransformError::ProFeatureRequired(format!(
            "DIMSE C-STORE 'push_dataset' operation to AE Title '{}' is a PRO feature. Community edition supports local filesystem output. Please upgrade to dicom-rs-transformer-pro for DIMSE networking and AE Title push.",
            to_ae
        )))
    }
}

/// Default implementation for Community Edition CloudStorageHandler.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultCloudStorageHandler;

impl CloudStorageHandler for DefaultCloudStorageHandler {}

/// Default implementation for Community Edition SequencePathEvaluator.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSequencePathEvaluator;

impl SequencePathEvaluator for DefaultSequencePathEvaluator {}

/// Default implementation for Community Edition LogicStackEvaluator.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultLogicStackEvaluator;

impl LogicStackEvaluator for DefaultLogicStackEvaluator {}

/// Default implementation for Community Edition PacsPushHandler.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultPacsPushHandler;

impl PacsPushHandler for DefaultPacsPushHandler {}

/// Default implementation for Community Edition DimseHandler.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultDimseHandler;

impl DimseHandler for DefaultDimseHandler {}


