//! Core DICOM transformation engine for executing actions on DICOM datasets.

use dicom_core::dictionary::DataDictionary;
use dicom_core::header::Header;
use dicom_core::value::{DataSetSequence, Value};
use dicom_core::{DataElement, Tag, VR};
use dicom_dictionary_std::StandardDataDictionary;
use dicom_object::{FileDicomObject, InMemDicomObject};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Instant;

use crate::dsl::{Action, TagPathSegment, TagSelector, TransformSpec};
use crate::error::TransformError;
use crate::macro_eval::evaluate_macros;
use crate::map::AnonymizationMap;

/// Overall outcome status of a transformation pipeline execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformStatus {
    /// Every action in the specification resulted in dataset changes.
    Success,
    /// At least one action resulted in dataset changes, but others were skipped or no-ops.
    Partial,
    /// No actions resulted in dataset changes (all actions were skipped or no-ops).
    None,
}

/// Execution report detailing the outcome of a transformation pipeline run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformReport {
    /// Overall transformation outcome status (`Success`, `Partial`, or `None`).
    pub status: TransformStatus,
    /// Total number of actions defined in the specification.
    pub total_actions: usize,
    /// Total number of actions evaluated during execution.
    pub actions_executed: usize,
    /// Total number of actions that resulted in dataset changes.
    pub actions_effective: usize,
    /// Count of DICOM tags modified or inserted.
    pub tags_modified: usize,
    /// Count of DICOM tags removed.
    pub tags_removed: usize,
    /// Time taken to complete the transformation in milliseconds.
    pub duration_ms: u128,
    /// Anonymization audit map generated during execution.
    pub map: AnonymizationMap,
}

impl TransformReport {
    /// Returns `true` if all actions succeeded in mutating the dataset.
    pub fn is_success(&self) -> bool {
        self.status == TransformStatus::Success
    }

    /// Returns `true` if some actions modified the dataset while others were skipped.
    pub fn is_partial(&self) -> bool {
        self.status == TransformStatus::Partial
    }

    /// Returns `true` if no actions modified the dataset.
    pub fn is_none(&self) -> bool {
        self.status == TransformStatus::None
    }
}

fn resolve_target_filename(
    dataset: &InMemDicomObject,
    source_path: Option<&str>,
    extension: &str,
) -> String {
    let base_name = source_path
        .and_then(|s| std::path::Path::new(s).file_stem())
        .map(|f| f.to_string_lossy().to_string())
        .or_else(|| {
            dataset
                .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
                .ok()
                .and_then(|e| e.to_str().ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| format!("dataset_{}", uuid::Uuid::new_v4()));

    format!("{}.{}", base_name, extension)
}

/// Executor that applies `TransformSpec` action sequences to DICOM objects.
#[derive(Debug, Clone)]
pub struct DicomTransformer {
    spec: TransformSpec,
}

impl DicomTransformer {
    /// Creates a new `DicomTransformer` with the specified transformation spec.
    pub fn new(spec: TransformSpec) -> Self {
        Self { spec }
    }

    /// Access the underlying `TransformSpec`.
    pub fn spec(&self) -> &TransformSpec {
        &self.spec
    }

    /// Applies the configured transformation pipeline to a `FileDicomObject`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if any action resolution or dataset mutation fails.
    pub fn transform_file(
        &self,
        obj: &mut FileDicomObject<InMemDicomObject>,
    ) -> Result<TransformReport, TransformError> {
        self.transform_dataset(obj)
    }

    /// Applies the configured transformation pipeline to an `InMemDicomObject`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if any action fails.
    pub fn transform_dataset(
        &self,
        dataset: &mut InMemDicomObject,
    ) -> Result<TransformReport, TransformError> {
        let start = Instant::now();
        let total_actions = self.spec.actions.len();
        let mut actions_executed = 0;
        let mut actions_effective = 0;
        let mut tags_modified = 0;
        let mut tags_removed = 0;

        let mut map = AnonymizationMap::new(None);

        for action in &self.spec.actions {
            match action {
                Action::LoadDataset { location } => {
                    let eval_loc = evaluate_macros(location)?;
                    let loaded = crate::io::load_dicom_object(&eval_loc)?;
                    *dataset = loaded.into_inner();
                    map.source = Some(eval_loc);
                    actions_effective += 1;
                }
                Action::SaveDataset { location } => {
                    let eval_loc = evaluate_macros(location)?;

                    let sop_class_uid = dataset
                        .element(dicom_dictionary_std::tags::SOP_CLASS_UID)
                        .ok()
                        .and_then(|e| e.to_str().ok())
                        .map(|s| s.to_string());

                    let media_sop_instance_uid = format!("2.25.{}", uuid::Uuid::new_v4().as_u128());

                    let mut meta_builder = dicom_object::FileMetaTableBuilder::new()
                        .media_storage_sop_instance_uid(media_sop_instance_uid)
                        .transfer_syntax(
                            dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.uid(),
                        );

                    if let Some(ref class_uid) = sop_class_uid {
                        meta_builder = meta_builder.media_storage_sop_class_uid(class_uid);
                    }

                    let meta = meta_builder
                        .build()
                        .map_err(|e| TransformError::InvalidOperation(e.to_string()))?;

                    let file_obj = FileDicomObject::new_empty_with_dict_and_meta(
                        StandardDataDictionary,
                        meta,
                    );
                    let mut buf = Vec::new();
                    file_obj.write_meta(&mut buf)?;
                    dataset.write_dataset_with_ts(
                        &mut buf,
                        &dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN
                            .erased(),
                    )?;

                    // Prefix with 128-byte preamble + "DICM" magic header
                    let mut full_buf = vec![0u8; 128];
                    full_buf.extend_from_slice(b"DICM");
                    full_buf.extend_from_slice(&buf);

                    let mut target_loc = eval_loc.clone();
                    let path = std::path::Path::new(&eval_loc);
                    let is_dir_target = eval_loc.ends_with('/') || eval_loc.ends_with('\\') || path.is_dir();

                    if is_dir_target {
                        let filename = resolve_target_filename(dataset, map.source.as_deref(), "dcm");
                        target_loc = path.join(filename).to_string_lossy().to_string();
                    }

                    crate::io::write_bytes(&target_loc, &full_buf)?;
                    actions_effective += 1;
                }
                Action::SaveMap { location } => {
                    let eval_loc = evaluate_macros(location)?;
                    let path = std::path::Path::new(&eval_loc);
                    let is_dir_target = eval_loc.ends_with('/') || eval_loc.ends_with('\\') || path.is_dir();
                    let target_loc = if is_dir_target {
                        let filename = resolve_target_filename(dataset, map.source.as_deref(), "map.json");
                        path.join(filename).to_string_lossy().to_string()
                    } else {
                        eval_loc
                    };

                    map.save(&target_loc)?;
                    actions_effective += 1;
                }
                Action::ExtractPixels {
                    destination,
                    format,
                } => {
                    let eval_dest = evaluate_macros(destination)?;
                    let export_fmt: crate::pixels::PixelExportFormat = format.parse()?;

                    let path = std::path::Path::new(&eval_dest);
                    let is_dir_target = eval_dest.ends_with('/') || eval_dest.ends_with('\\') || path.is_dir();
                    let final_dest = if is_dir_target {
                        let uid_folder = dataset
                            .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
                            .ok()
                            .and_then(|e| e.to_str().ok())
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .or_else(|| {
                                map.source
                                    .as_ref()
                                    .and_then(|s| std::path::Path::new(s).file_stem())
                                    .map(|f| f.to_string_lossy().to_string())
                            })
                            .unwrap_or_else(|| format!("dataset_{}", uuid::Uuid::new_v4()));
                        path.join(uid_folder).to_string_lossy().to_string()
                    } else {
                        eval_dest
                    };

                    let frame_count =
                        crate::pixels::extract_pixel_frames(dataset, &final_dest, export_fmt)?;

                    if frame_count > 0 {
                        actions_effective += 1;
                    }
                }
                Action::SetTag { selector, value } => {
                    let path = selector.resolve_path()?;
                    let eval_val = evaluate_macros(value)?;
                    let orig_val = get_tag_path_str(dataset, &path.segments).unwrap_or_default();

                    let count = apply_set_path(dataset, &path.segments, &eval_val)?;
                    if count > 0 {
                        tags_modified += count;
                        actions_effective += 1;

                        map.add_entry(
                            &path.to_string(),
                            Some(&selector.to_string()),
                            &orig_val,
                            &eval_val,
                        );
                    }
                }
                Action::GenerateUid { selector, source } => {
                    let path = selector.resolve_path()?;
                    let orig_val = get_tag_path_str(dataset, &path.segments).unwrap_or_default();

                    let uid_val = match source {
                        Some(src_expr) => {
                            let eval_src = evaluate_macros(src_expr)?;
                            // If src_expr is a tag selector reference (e.g. StudyInstanceUID), attempt reading dataset value
                            let seed_str = if let Ok(src_selector) = TagSelector::from_str(&eval_src) {
                                if let Ok(src_path) = src_selector.resolve_path() {
                                    get_tag_path_str(dataset, &src_path.segments).unwrap_or(eval_src)
                                } else {
                                    eval_src
                                }
                            } else {
                                eval_src
                            };
                            // Generate deterministic UUID v5 using NAMESPACE_OID and seed_str
                            let generated_uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, seed_str.as_bytes());
                            format!("2.25.{}", generated_uuid.as_u128())
                        }
                        None => {
                            // Generate random UUID v4 derived UID
                            let generated_uuid = uuid::Uuid::new_v4();
                            format!("2.25.{}", generated_uuid.as_u128())
                        }
                    };

                    let count = apply_set_path(dataset, &path.segments, &uid_val)?;
                    if count > 0 {
                        tags_modified += count;
                        actions_effective += 1;

                        map.add_entry(
                            &path.to_string(),
                            Some(&selector.to_string()),
                            &orig_val,
                            &uid_val,
                        );
                    }
                }
                Action::RemoveTag { selector } => {
                    let path = selector.resolve_path()?;
                    let orig_val = get_tag_path_str(dataset, &path.segments).unwrap_or_default();

                    let count = apply_remove_path(dataset, &path.segments)?;
                    if count > 0 {
                        tags_removed += count;
                        actions_effective += 1;

                        map.add_entry(
                            &path.to_string(),
                            Some(&selector.to_string()),
                            &orig_val,
                            "[REMOVED]",
                        );
                    }
                }
                Action::ReplaceValue {
                    selector,
                    pattern,
                    replacement,
                } => {
                    let path = selector.resolve_path()?;
                    let eval_pat = evaluate_macros(pattern)?;
                    let eval_rep = evaluate_macros(replacement)?;
                    let orig_val = get_tag_path_str(dataset, &path.segments).unwrap_or_default();

                    let count = apply_replace_path(dataset, &path.segments, &eval_pat, &eval_rep)?;
                    if count > 0 {
                        tags_modified += count;
                        actions_effective += 1;

                        let new_val = get_tag_path_str(dataset, &path.segments).unwrap_or_default();
                        map.add_entry(
                            &path.to_string(),
                            Some(&selector.to_string()),
                            &orig_val,
                            &new_val,
                        );
                    }
                }
                Action::SaveJson {
                    json_location,
                    raw_pixel_location,
                } => {
                    let eval_json_loc = evaluate_macros(json_location)?;
                    let path = std::path::Path::new(&eval_json_loc);
                    let is_dir_target = eval_json_loc.ends_with('/') || eval_json_loc.ends_with('\\') || path.is_dir();
                    let target_json_loc = if is_dir_target {
                        let filename = resolve_target_filename(dataset, map.source.as_deref(), "json");
                        path.join(filename).to_string_lossy().to_string()
                    } else {
                        eval_json_loc
                    };

                    // If raw_pixel_location is omitted, default to json_file file path + .raw
                    let eval_raw_loc = match raw_pixel_location {
                        Some(ref raw_loc) => {
                            let eval_r = evaluate_macros(raw_loc)?;
                            let r_path = std::path::Path::new(&eval_r);
                            let r_is_dir = eval_r.ends_with('/') || eval_r.ends_with('\\') || r_path.is_dir();
                            if r_is_dir {
                                let filename = resolve_target_filename(dataset, map.source.as_deref(), "raw");
                                r_path.join(filename).to_string_lossy().to_string()
                            } else {
                                eval_r
                            }
                        }
                        None => {
                            let json_path = std::path::Path::new(&target_json_loc);
                            json_path.with_extension("raw").to_string_lossy().to_string()
                        }
                    };

                    // Clone dataset so pixel data modification doesn't alter current in-memory dataset
                    let mut json_ds = dataset.clone();

                    // Check if PixelData exists and extract raw pixel data if present
                    if json_ds.element(dicom_dictionary_std::tags::PIXEL_DATA).is_ok() {
                        let _ = crate::pixels::extract_pixel_frames(
                            &json_ds,
                            &eval_raw_loc,
                            crate::pixels::PixelExportFormat::Raw,
                        );
                        json_ds.remove_element(dicom_dictionary_std::tags::PIXEL_DATA);
                    }

                    let json_val = dicom_json::to_value(&json_ds)?;
                    let json_bytes = serde_json::to_vec_pretty(&json_val)?;
                    crate::io::write_bytes(&target_json_loc, &json_bytes)?;
                    actions_effective += 1;
                }
                Action::Dump { location } => {
                    let eval_loc = evaluate_macros(location)?;
                    let path = std::path::Path::new(&eval_loc);
                    let is_dir_target = eval_loc.ends_with('/') || eval_loc.ends_with('\\') || path.is_dir();
                    let target_loc = if is_dir_target {
                        let filename = resolve_target_filename(dataset, map.source.as_deref(), "txt");
                        path.join(filename).to_string_lossy().to_string()
                    } else {
                        eval_loc
                    };

                    let mut dump_options = dicom_dump::DumpOptions::new();
                    dump_options.no_limit(true);
                    dump_options.color_mode(dicom_dump::ColorMode::Never);

                    let sop_class_uid = dataset
                        .element(dicom_dictionary_std::tags::SOP_CLASS_UID)
                        .ok()
                        .and_then(|e| e.to_str().ok())
                        .map(|s| s.to_string());

                    let media_sop_instance_uid = format!("2.25.{}", uuid::Uuid::new_v4().as_u128());

                    let mut meta_builder = dicom_object::FileMetaTableBuilder::new()
                        .media_storage_sop_instance_uid(media_sop_instance_uid)
                        .transfer_syntax(
                            dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.uid(),
                        );

                    if let Some(ref class_uid) = sop_class_uid {
                        meta_builder = meta_builder.media_storage_sop_class_uid(class_uid);
                    }

                    let meta = meta_builder
                        .build()
                        .unwrap_or_else(|_| dicom_object::FileMetaTableBuilder::new().build().unwrap());

                    let file_obj = dicom_object::FileDicomObject::new_empty_with_dict_and_meta(
                        StandardDataDictionary,
                        meta,
                    );
                    let mut full_file_obj = file_obj;
                    *full_file_obj = dataset.clone();

                    let mut buf = Vec::new();
                    dump_options.dump_file_to(&mut buf, &full_file_obj)?;
                    crate::io::write_bytes(&target_loc, &buf)?;
                    actions_effective += 1;
                }
                Action::Check { check_op, .. } => {
                    use crate::pro::{DefaultLogicStackEvaluator, LogicStackEvaluator};
                    DefaultLogicStackEvaluator.evaluate_logic_action(&format!("CHECK {}", check_op))?;
                }
                Action::Execute => {
                    actions_effective += 1;
                }
                Action::LogicOp { logic_op } => {
                    use crate::pro::{DefaultLogicStackEvaluator, LogicStackEvaluator};
                    DefaultLogicStackEvaluator.evaluate_logic_action(&logic_op)?;
                }
                Action::IfBranch { condition, .. } => {
                    use crate::pro::{DefaultLogicStackEvaluator, LogicStackEvaluator};
                    let op_name = if *condition { "IF_TRUE" } else { "IF_FALSE" };
                    DefaultLogicStackEvaluator.evaluate_logic_action(op_name)?;
                }
                Action::Assemble {
                    input_location,
                    raw_location,
                    output_location,
                    pacs_destination,
                } => {
                    let eval_input = evaluate_macros(input_location)?;
                    let eval_raw = match raw_location {
                        Some(ref r) => Some(evaluate_macros(r)?),
                        None => None,
                    };
                    let in_path = std::path::Path::new(&eval_input);
                    let raw_p = eval_raw.as_deref().map(std::path::Path::new);

                    if in_path.is_dir() {
                        let res = crate::assemble::DicomAssembler::assemble_directory(in_path, raw_p)?;
                        if let Some(ref out) = output_location {
                            let eval_out = evaluate_macros(out)?;
                            let out_path = std::path::Path::new(&eval_out);
                            if !out_path.exists() {
                                std::fs::create_dir_all(out_path)?;
                            }
                            for (idx, obj) in res.objects.iter().enumerate() {
                                let sop_uid = obj
                                    .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
                                    .ok()
                                    .and_then(|e| e.to_str().ok())
                                    .map(|s| s.trim().to_string())
                                    .unwrap_or_else(|| format!("assembled_{}", idx));
                                let filename = format!("{}.dcm", sop_uid);
                                let file_dest = out_path.join(filename);
                                crate::io::save_dicom_object(&file_dest.to_string_lossy(), obj)?;
                            }
                        }
                        if let Some(ref pacs) = pacs_destination {
                            use crate::pro::{DefaultPacsPushHandler, PacsPushHandler};
                            for obj in &res.objects {
                                DefaultPacsPushHandler.push_pacs(pacs, obj)?;
                            }
                        }
                        if let Some(first) = res.objects.into_iter().next() {
                            *dataset = first.into_inner();
                        }
                        actions_effective += 1;
                    } else {
                        let obj = crate::assemble::DicomAssembler::assemble_file(in_path, raw_p)?;
                        if let Some(ref out) = output_location {
                            let eval_out = evaluate_macros(out)?;
                            let out_path = std::path::Path::new(&eval_out);
                            let target_file = if eval_out.ends_with('/') || eval_out.ends_with('\\') || out_path.is_dir() {
                                if !out_path.exists() {
                                    std::fs::create_dir_all(out_path)?;
                                }
                                let sop_uid = obj
                                    .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
                                    .ok()
                                    .and_then(|e| e.to_str().ok())
                                    .map(|s| s.trim().to_string())
                                    .unwrap_or_else(|| "assembled".to_string());
                                out_path.join(format!("{}.dcm", sop_uid)).to_string_lossy().to_string()
                            } else {
                                eval_out
                            };
                            crate::io::save_dicom_object(&target_file, &obj)?;
                        }
                        if let Some(ref pacs) = pacs_destination {
                            use crate::pro::{DefaultPacsPushHandler, PacsPushHandler};
                            DefaultPacsPushHandler.push_pacs(pacs, &obj)?;
                        }
                        *dataset = obj.into_inner();
                        actions_effective += 1;
                    }
                }
                Action::Fetch {
                    filters,
                    from_ae,
                    to_ae,
                } => {
                    use crate::pro::{DefaultDimseHandler, DimseHandler};
                    let eval_from = evaluate_macros(from_ae)?;
                    let eval_to = evaluate_macros(to_ae)?;
                    let mut eval_filters = std::collections::HashMap::new();
                    for (k, v) in filters {
                        eval_filters.insert(k.clone(), evaluate_macros(v)?);
                    }
                    let results = DefaultDimseHandler.fetch_datasets(&eval_filters, &eval_from, &eval_to)?;
                    if let Some(first) = results.into_iter().next() {
                        *dataset = first.into_inner();
                    }
                    actions_effective += 1;
                }
                Action::PushDataset { to_ae } => {
                    use crate::pro::{DefaultDimseHandler, DimseHandler};
                    let eval_to = evaluate_macros(to_ae)?;
                    let meta = dicom_object::FileMetaTableBuilder::new()
                        .media_storage_sop_instance_uid(format!("2.25.{}", uuid::Uuid::new_v4().as_u128()))
                        .transfer_syntax(
                            dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.uid(),
                        )
                        .build()
                        .unwrap_or_else(|_| dicom_object::FileMetaTableBuilder::new().build().unwrap());
                    let file_obj = FileDicomObject::new_empty_with_dict_and_meta(
                        dicom_dictionary_std::StandardDataDictionary,
                        meta,
                    );
                    let mut full_file_obj = file_obj;
                    *full_file_obj = dataset.clone();
                    DefaultDimseHandler.push_dataset(&eval_to, &full_file_obj)?;
                    actions_effective += 1;
                }
                Action::AnonymizePatient {
                    patient_name,
                    patient_id,
                } => {
                    let name_tpl = patient_name.as_deref().unwrap_or("ANONYMOUS");
                    let id_tpl = patient_id.as_deref().unwrap_or("ANON-ID");

                    let eval_name = evaluate_macros(name_tpl)?;
                    let eval_id = evaluate_macros(id_tpl)?;

                    let tag_name = Tag(0x0010, 0x0010);
                    let tag_id = Tag(0x0010, 0x0020);

                    let orig_name = dataset
                        .element(tag_name)
                        .ok()
                        .and_then(|e| e.to_str().ok())
                        .unwrap_or_default()
                        .to_string();
                    let orig_id = dataset
                        .element(tag_id)
                        .ok()
                        .and_then(|e| e.to_str().ok())
                        .unwrap_or_default()
                        .to_string();

                    set_element_value(dataset, tag_name, &eval_name)?;
                    set_element_value(dataset, tag_id, &eval_id)?;
                    tags_modified += 2;
                    actions_effective += 1;

                    map.add_entry("(0010,0010)", Some("PatientName"), &orig_name, &eval_name);
                    map.add_entry("(0010,0020)", Some("PatientID"), &orig_id, &eval_id);
                }
            }
            actions_executed += 1;
        }

        let status = if total_actions == 0 || actions_effective == 0 {
            TransformStatus::None
        } else if actions_effective == total_actions {
            TransformStatus::Success
        } else {
            TransformStatus::Partial
        };

        let duration_ms = start.elapsed().as_millis();
        Ok(TransformReport {
            status,
            total_actions,
            actions_executed,
            actions_effective,
            tags_modified,
            tags_removed,
            duration_ms,
            map,
        })
    }
}

/// Reads the current string value at the given tag path (if present).
fn get_tag_path_str(dataset: &InMemDicomObject, segments: &[TagPathSegment]) -> Option<String> {
    if segments.is_empty() {
        return None;
    }

    let head = &segments[0];
    if segments.len() == 1 {
        return dataset
            .element(head.tag)
            .ok()
            .and_then(|e| e.to_str().ok())
            .map(|s| s.to_string());
    }

    if let Ok(elem) = dataset.element(head.tag) {
        if let Value::Sequence(seq) = elem.value() {
            let idx = head.item_index.unwrap_or(0);
            if idx < seq.items().len() {
                return get_tag_path_str(&seq.items()[idx], &segments[1..]);
            }
        }
    }

    None
}

/// Applies a `SetTag` operation along a tag path.
/// Returns the number of tags modified or added.
fn apply_set_path(
    dataset: &mut InMemDicomObject,
    segments: &[TagPathSegment],
    value: &str,
) -> Result<usize, TransformError> {
    if segments.is_empty() {
        return Ok(0);
    }

    let head = &segments[0];
    if segments.len() == 1 {
        set_element_value(dataset, head.tag, value)?;
        return Ok(1);
    }

    let tail = &segments[1..];

    // Ensure sequence element exists
    if dataset.element(head.tag).is_err() {
        let initial_items_count = head.item_index.map(|i| i + 1).unwrap_or(1);
        let items: Vec<InMemDicomObject> = (0..initial_items_count)
            .map(|_| InMemDicomObject::new_empty())
            .collect();
        let seq_val = DataSetSequence::from(items);
        let elem = DataElement::new(head.tag, VR::SQ, Value::Sequence(seq_val));
        dataset.put(elem);
    }

    if let Ok(elem) = dataset.take_element(head.tag) {
        let (tag, vr, val) = (elem.tag(), elem.vr(), elem.into_value());
        let mut count = 0;

        if let Value::Sequence(mut seq) = val {
            if let Some(target_idx) = head.item_index {
                while seq.items_mut().len() <= target_idx {
                    seq.items_mut().push(InMemDicomObject::new_empty());
                }
                count += apply_set_path(&mut seq.items_mut()[target_idx], tail, value)?;
            } else {
                if seq.items_mut().is_empty() {
                    seq.items_mut().push(InMemDicomObject::new_empty());
                }
                for item in seq.items_mut() {
                    count += apply_set_path(item, tail, value)?;
                }
            }

            let new_elem = DataElement::new(tag, vr, Value::Sequence(seq));
            dataset.put(new_elem);
        } else {
            let orig_elem = DataElement::new(tag, vr, val);
            dataset.put(orig_elem);
        }

        Ok(count)
    } else {
        Ok(0)
    }
}

/// Applies a `RemoveTag` operation along a tag path.
/// Returns the number of tags removed.
fn apply_remove_path(
    dataset: &mut InMemDicomObject,
    segments: &[TagPathSegment],
) -> Result<usize, TransformError> {
    if segments.is_empty() {
        return Ok(0);
    }

    let head = &segments[0];
    if segments.len() == 1 {
        if dataset.element(head.tag).is_ok() {
            dataset.remove_element(head.tag);
            return Ok(1);
        }
        return Ok(0);
    }

    let tail = &segments[1..];
    if let Ok(elem) = dataset.take_element(head.tag) {
        let (tag, vr, val) = (elem.tag(), elem.vr(), elem.into_value());
        let mut count = 0;

        if let Value::Sequence(mut seq) = val {
            if let Some(target_idx) = head.item_index {
                if target_idx < seq.items_mut().len() {
                    count += apply_remove_path(&mut seq.items_mut()[target_idx], tail)?;
                }
            } else {
                for item in seq.items_mut() {
                    count += apply_remove_path(item, tail)?;
                }
            }

            let new_elem = DataElement::new(tag, vr, Value::Sequence(seq));
            dataset.put(new_elem);
        } else {
            let orig_elem = DataElement::new(tag, vr, val);
            dataset.put(orig_elem);
        }

        Ok(count)
    } else {
        Ok(0)
    }
}

/// Applies a `ReplaceValue` operation along a tag path.
/// Returns the number of elements modified.
fn apply_replace_path(
    dataset: &mut InMemDicomObject,
    segments: &[TagPathSegment],
    pattern: &str,
    replacement: &str,
) -> Result<usize, TransformError> {
    if segments.is_empty() {
        return Ok(0);
    }

    let head = &segments[0];
    if segments.len() == 1 {
        if let Ok(elem) = dataset.element(head.tag) {
            if let Ok(current_val) = elem.to_str() {
                if current_val.contains(pattern) {
                    let new_val = current_val.replace(pattern, replacement);
                    set_element_value(dataset, head.tag, &new_val)?;
                    return Ok(1);
                }
            }
        }
        return Ok(0);
    }

    let tail = &segments[1..];
    if let Ok(elem) = dataset.take_element(head.tag) {
        let (tag, vr, val) = (elem.tag(), elem.vr(), elem.into_value());
        let mut count = 0;

        if let Value::Sequence(mut seq) = val {
            if let Some(target_idx) = head.item_index {
                if target_idx < seq.items_mut().len() {
                    count += apply_replace_path(
                        &mut seq.items_mut()[target_idx],
                        tail,
                        pattern,
                        replacement,
                    )?;
                }
            } else {
                for item in seq.items_mut() {
                    count += apply_replace_path(item, tail, pattern, replacement)?;
                }
            }

            let new_elem = DataElement::new(tag, vr, Value::Sequence(seq));
            dataset.put(new_elem);
        } else {
            let orig_elem = DataElement::new(tag, vr, val);
            dataset.put(orig_elem);
        }

        Ok(count)
    } else {
        Ok(0)
    }
}

/// Helper function to set or replace a string value in a DICOM dataset.
fn set_element_value(
    dataset: &mut InMemDicomObject,
    tag: Tag,
    value: &str,
) -> Result<(), TransformError> {
    let dict = StandardDataDictionary;

    // Resolve Value Representation (VR) from standard dictionary or default to LO (Long String)
    let vr = dict
        .by_tag(tag)
        .map(|entry| entry.vr.relaxed())
        .unwrap_or(VR::LO);

    let element = DataElement::new(tag, vr, Value::from(value));
    dataset.put(element);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::TagSelector;

    #[test]
    fn test_set_and_remove_tag() {
        let mut dataset = InMemDicomObject::new_empty();

        let mut spec = TransformSpec::new();
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword("PatientName".to_string()),
            value: "DOE^JOHN".to_string(),
        });
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword("PatientID".to_string()),
            value: "12345".to_string(),
        });

        let transformer = DicomTransformer::new(spec);
        let report = transformer.transform_dataset(&mut dataset).unwrap();

        assert_eq!(report.status, TransformStatus::Success);
        assert_eq!(report.actions_executed, 2);
        assert_eq!(report.actions_effective, 2);
        assert_eq!(report.tags_modified, 2);

        let elem = dataset.element(Tag(0x0010, 0x0010)).unwrap();
        assert_eq!(elem.to_str().unwrap(), "DOE^JOHN");

        // Now remove PatientID
        let mut remove_spec = TransformSpec::new();
        remove_spec.add_action(Action::RemoveTag {
            selector: TagSelector::Keyword("PatientID".to_string()),
        });
        let remove_transformer = DicomTransformer::new(remove_spec);
        let remove_report = remove_transformer.transform_dataset(&mut dataset).unwrap();

        assert_eq!(remove_report.status, TransformStatus::Success);
        assert_eq!(remove_report.tags_removed, 1);
        assert!(dataset.element(Tag(0x0010, 0x0020)).is_err());
    }

    #[test]
    fn test_macro_evaluation_in_engine() {
        let mut dataset = InMemDicomObject::new_empty();
        let mut spec = TransformSpec::new();
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword("PatientName".to_string()),
            value: "ANON-$rand_str(6)".to_string(),
        });

        let transformer = DicomTransformer::new(spec);
        let report = transformer.transform_dataset(&mut dataset).unwrap();

        assert_eq!(report.status, TransformStatus::Success);
        let name = dataset
            .element(Tag(0x0010, 0x0010))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert_eq!(name.len(), 11); // "ANON-XXXXXX"
        assert!(report.map.entries.len() == 1);
    }

    #[test]
    fn test_sequence_path_indexed_set_and_remove() {
        let mut dataset = InMemDicomObject::new_empty();
        let mut spec = TransformSpec::new();
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword(
                "RequestAttributesSequence[0]/ScheduledProcedureStepID".to_string(),
            ),
            value: "PROC-101".to_string(),
        });

        let transformer = DicomTransformer::new(spec);
        let err = transformer.transform_dataset(&mut dataset).unwrap_err();
        match err {
            TransformError::ProFeatureRequired(msg) => {
                assert!(msg.contains("Nested DICOM Sequence path"));
            }
            _ => panic!("Expected ProFeatureRequired error"),
        }
    }

    #[test]
    fn test_sequence_path_wildcard_set_remove_and_replace() {
        let mut dataset = InMemDicomObject::new_empty();
        let mut spec = TransformSpec::new();
        spec.add_action(Action::SetTag {
            selector: TagSelector::Keyword(
                "RequestAttributesSequence/ScheduledProcedureStepID".to_string(),
            ),
            value: "SITE_NORTH_100".to_string(),
        });

        let transformer = DicomTransformer::new(spec);
        let err = transformer.transform_dataset(&mut dataset).unwrap_err();
        match err {
            TransformError::ProFeatureRequired(msg) => {
                assert!(msg.contains("Nested DICOM Sequence path"));
            }
            _ => panic!("Expected ProFeatureRequired error"),
        }
    }
}
