# Requirements Traceability Matrix (RTM)

**Document ID:** RTM-DRT-001  
**Project:** `dicom-rs-transformer`  
**Regulatory Standard Alignment:** IEC 62304 Clause 5.1.1 / 5.2.6, FDA Design Controls  

---

## 1. Bi-Directional Traceability Overview

This matrix establishes complete bi-directional traceability linking **Software Requirements (SRS)** $\leftrightarrow$ **System Design Specs (SDS)** $\leftrightarrow$ **Software Hazards (ISO 14971)** $\leftrightarrow$ **Verification Test Suites (V&V)**.

---

## 2. Traceability Matrix

| Requirement ID | Software Requirement Description | Design Spec Module | Hazard ID | Risk Mitigation | Verification Test Method | Pass Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **REQ-FUN-001** | In-Memory DICOM Parsing | `dicom-rs` integration (`src/engine.rs`) | - | Native Rust memory allocation & error handling | Unit test `test_set_and_remove_tag` | Pass |
| **REQ-FUN-002** | Declarative JSON DSL | `TransformSpec` / `Action` (`src/dsl.rs`) | HAZ-004 | Type-safe JSON deserialization | Integration test `test_full_transformation_pipeline` | Pass |
| **REQ-FUN-003** | Line-by-Line Script Language | `ScriptParser` (`src/script.rs`) | HAZ-004 | Explicit line syntax validation | Unit test `test_script_parser_and_execution` | Pass |
| **REQ-FUN-004** | Sequence Path Traversal | `TagPath` / `DicomTransformer` (`src/engine.rs`) | HAZ-002 | Safe sequence item initialization (`ensure_sequence_path`) | Unit test `test_sequence_path_indexed_set_and_remove` | Pass |
| **REQ-FUN-005** | Dynamic Macro Expansion | `MacroEvaluator` (`src/macro_eval.rs`) | HAZ-006 | Regex macro parsing & pattern matching | Unit test `test_macro_evaluation_in_engine` | Pass |
| **REQ-FUN-006** | Audit Mapping (`map.json`) | `AnonymizationMap` (`src/map.rs`) | HAZ-001 | Pre/post modification value tracking | Unit test `test_full_transformation_pipeline` | Pass |
| **REQ-FUN-007** | RPN Predicate Logic | `macro_eval.rs` & `script.rs` | HAZ-004 | Stack evaluator bounds checking | Integration test `transformation_tests.rs` | Pass |
| **REQ-FUN-008** | Model Context Protocol Server | REPL / MCP handler (`src/main.rs`) | HAZ-004 | Stdio sub-process local user isolation | CLI test `schema` / `install-mcp` | Pass |
| **REQ-FUN-009** | Pixel Frame Extraction | `PixelExtractor` (`src/pixels.rs`) | HAZ-005 | Raw frame buffer bounds extraction | Integration test `pixels_test` | Pass |
| **REQ-FUN-010** | CLI Subcommand Interface | Clap parser (`src/main.rs`) | HAZ-005 | Subcommand argument verification | CLI execution tests (`run`, `validate`, `compile`) | Pass |
| **REQ-FUN-011** | De-Identification Profile Loader | Loader (`src/deidentification_config_loader.rs`) | HAZ-001 | Embedded default JSON profile fallback | Unit test `test_load_default_deidentification_rules` | Pass |
| **REQ-FUN-012** | Profile Option Action Resolution | Models (`src/models/deidentification_config.rs`) | HAZ-001 | Table E.1-1 option override resolution | Unit test `test_resolve_action_overrides` | Pass |
| **REQ-FUN-013** | Annex E Action Code Engine | Transformer Engine (`src/engine.rs`) | HAZ-001 | Execution of Annex E actions (X/Z/D/C/U/K) | Unit test `test_deidentification_execution_and_private_tags` | Pass |
| **REQ-FUN-014** | Default Private Tag Removal | Transformer Engine (`src/engine.rs`) | HAZ-001 | Automatic removal of odd group tags (`group % 2 != 0`) | Unit test `test_deidentification_execution_and_private_tags` & Integration test `test_load_di_profile_and_deidentify_pipeline` | Pass |
| **REQ-REG-001** | De-Identification Standards | `Deidentify` / `AnonymizationMap` | HAZ-001 | Audit log generation & Safe Harbor / Annex E rules | Integration test `test_load_di_profile_and_deidentify_pipeline` | Pass |
| **REQ-REG-002** | DICOM Part 5 VR Compliance | `dicom-rs` VR engine | HAZ-003 | Strict Value Representation checks | Unit & integration test suite | Pass |

