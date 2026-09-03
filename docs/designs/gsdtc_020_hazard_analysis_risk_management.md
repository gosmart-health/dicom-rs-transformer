# Hazard Analysis & Software Risk Management Plan

**Document ID:** RMF-DRT-001  
**Project:** `dicom-rs-transformer`  
**Regulatory Standard Alignment:** ISO 14971:2019, IEC 62304 Clause 7, FDA SaMD Safety Guidance  

---

## 1. Risk Management Framework

This document provides a Software Hazard Analysis for `dicom-rs-transformer`. It identifies potential software hazards associated with DICOM attribute mutations, sequence path traversals, macro expansions, and MCP stdio tool execution, along with software design risk control measures implemented in the architecture.

---

## 2. Hazard Analysis Matrix

| Hazard ID | Hazard Description | Cause / Trigger | Potential Severity | Initial Risk | Software Risk Mitigation (Design Control) | Residual Risk | Verification Method |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **HAZ-001** | **Accidental PHI Leakage** in Output Dataset | Failure to anonymize sensitive patient identification tags (`PatientName`, `PatientID`, `PatientBirthDate`) or proprietary private data elements. | Critical | High | **Design Control:** Provide `Deidentify` action, PS3.15 Annex E Table E.1-1 action code engine (`X`/`Z`/`D`/`C`/`U`/`K`), default private tag stripping (`group % 2 != 0`), automated `map.json` audit logging (`AnonymizationMap`), tag path whitelist selectors, and pre-validated de-identification macros (`$uid`, `$rand_str`). | Negligible | Unit test `test_deidentification_execution_and_private_tags` & integration test `test_load_di_profile_and_deidentify_pipeline`. |
| **HAZ-002** | **Sequence Path Index Out-of-Bounds** | Accessing sequence item index exceeding existing item array bounds during set/remove operations. | Moderate | Medium | **Design Control:** `DicomTransformer::ensure_sequence_path()` safely appends empty sequence items to target index without array panics. | Negligible | Unit test `test_sequence_path_indexed_set_and_remove` for out-of-bounds indexing. |
| **HAZ-003** | **DICOM VR / Tag Element Corruption** | Writing malformed or mismatched value string violating DICOM Value Representation (VR) rules. | High | Medium | **Design Control:** `dicom-rs` strong type validation checks VR constraints during `DicomValue` conversion before mutating dataset. | Negligible | Integration tests across diverse VR types (`PN`, `UI`, `DA`, `TM`, `LO`, `SH`). |
| **HAZ-004** | **Unauthorized Stdio MCP Action Execution** | Malicious host tool or injected script attempting arbitrary file system access via stdio. | Moderate | Medium | **Design Control:** Interactive REPL and MCP tools strictly enforce strong enum parsing (`Action`), restricting mutations to valid DICOM operations. | Negligible | Security audit of CLI subcommands and MCP tool parameter validation schemas. |
| **HAZ-005** | **Unintended File Overwrite / Loss** | Output dataset path set identically to input path without backup or explicit intent. | Moderate | Medium | **Design Control:** File IO streams perform path validation and atomic target writes, ensuring partial writes do not corrupt source datasets. | Negligible | Pipeline integration test `test_full_transformation_pipeline`. |
| **HAZ-006** | **Incomplete Macro Expansion** | Unrecognized macro keyword leaves literal `$macro` string in DICOM tag output. | Low | Low | **Design Control:** `MacroEvaluator` uses strict regex parsing and returns explicit error results on invalid macro signatures. | Negligible | Unit test `test_macro_evaluation_in_engine`. |

---

## 3. Risk Management Conclusion

All identified hazards have been mitigated through software design controls. The residual risk for `dicom-rs-transformer` components is assessed as **acceptable** for medical imaging data transformation and research workflows.

