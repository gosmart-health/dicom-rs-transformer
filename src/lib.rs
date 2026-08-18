//! # dicom-rs-transformer
//!
//! `dicom-rs-transformer` is a utility library for transforming DICOM dataset objects
//! using JSON-encoded DSL specifications or line-by-line text scripts.
//!
//! ## Overview
//!
//! This library provides programmatic APIs for DICOM dataset transformations such as tag editing,
//! tag removal, string value pattern replacement, and standard patient anonymization.
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use dicom_rs_transformer::{
//!     Action, DicomTransformer, TagSelector, TransformSpec, TransformError
//! };
//! use dicom_object::open_file;
//!
//! fn main() -> Result<(), TransformError> {
//!     // 1. Build a transformation specification
//!     let mut spec = TransformSpec::new();
//!     spec.add_action(Action::SetTag {
//!         selector: TagSelector::Keyword("PatientName".to_string()),
//!         value: "ANONYMOUS^PATIENT".to_string(),
//!     });
//!     spec.add_action(Action::RemoveTag {
//!         selector: TagSelector::Keyword("PatientAddress".to_string()),
//!     });
//!
//!     // 2. Instantiate the transformer
//!     let transformer = DicomTransformer::new(spec);
//!
//!     // 3. Open a DICOM file and apply transformation
//!     let mut obj = open_file("sample.dcm")?;
//!     let report = transformer.transform_file(&mut obj)?;

//!     println!("Transformed dataset: {:?}", report);
//!     obj.write_to_file("anonymized.dcm")?;
//!
//!     Ok(())
//! }
//! ```

#![deny(missing_docs)]

pub mod dsl;
pub mod engine;
pub mod error;
pub mod io;
pub mod macro_eval;
pub mod map;
pub mod pixels;
pub mod pro;
pub mod script;

pub use dsl::{Action, TagSelector, TransformSpec};
pub use engine::{DicomTransformer, TransformReport, TransformStatus};
pub use error::TransformError;
pub use map::{AnonymizationMap, MappingEntry};
pub use pixels::{extract_pixel_frames, PixelExportFormat};
pub use pro::{
    CloudStorageHandler, DefaultCloudStorageHandler, DefaultLogicStackEvaluator,
    DefaultSequencePathEvaluator, LogicStackEvaluator, SequencePathEvaluator,
};
pub use script::ScriptParser;
