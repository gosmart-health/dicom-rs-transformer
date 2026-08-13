//! Error types for DICOM transformation and parsing operations.

use thiserror::Error;

/// Represents errors that can occur during DICOM dataset transformation,
/// script parsing, or DSL deserialization.
#[derive(Error, Debug)]
pub enum TransformError {
    /// Error when reading or loading a DICOM file/object.
    #[error("DICOM read error: {0}")]
    Read(#[from] Box<dicom_object::ReadError>),

    /// Error when writing or saving a DICOM file/object.
    #[error("DICOM write error: {0}")]
    Write(#[from] Box<dicom_object::WriteError>),

    /// Error accessing an element within a DICOM dataset.
    #[error("DICOM element access error: {0}")]
    Access(#[from] dicom_object::AccessError),

    /// Error accessing an element by name within a DICOM dataset.
    #[error("DICOM access by name error: {0}")]
    AccessByName(#[from] dicom_object::AccessByNameError),

    /// Error converting DICOM element values.
    #[error("DICOM convert value error: {0}")]
    ConvertValue(#[from] dicom_core::value::ConvertValueError),

    /// Error encountered when a DICOM tag keyword or tag string cannot be resolved.
    #[error("Unknown DICOM tag selector: {0}")]
    UnknownTag(String),

    /// Error parsing JSON-encoded DSL specification.
    #[error("DSL JSON parse error: {0}")]
    DslParse(#[from] serde_json::Error),

    /// Error parsing line-by-line script command.
    #[error("Script parse error at line {line}: {message}")]
    ScriptParse {
        /// Line number (1-indexed) where the syntax error occurred.
        line: usize,
        /// Description of the script syntax error.
        message: String,
    },

    /// Standard I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid transformation operation or invalid value format.
    #[error("Invalid transformation operation: {0}")]
    InvalidOperation(String),

    /// Operation or feature requires the PRO edition of dicom-rs-transformer.
    #[error("PRO Feature Required: {0}")]
    ProFeatureRequired(String),
}

impl From<dicom_object::ReadError> for TransformError {
    fn from(err: dicom_object::ReadError) -> Self {
        TransformError::Read(Box::new(err))
    }
}

impl From<dicom_object::WriteError> for TransformError {
    fn from(err: dicom_object::WriteError) -> Self {
        TransformError::Write(Box::new(err))
    }
}
