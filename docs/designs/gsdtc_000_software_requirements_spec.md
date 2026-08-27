# Software Requirements Specification (SRS)

**Document ID:** SRS-DRT-001  
**Project:** `dicom-rs-transformer`  
**Regulatory Standard Alignment:** IEC 62304 Clause 5.2, FDA 21 CFR 820.30  

---

## 1. Scope & Purpose

This document specifies the functional, performance, safety, and interface requirements for the `dicom-rs-transformer` software engine and CLI tool. Downstream medical device integrators and healthcare data pipeline developers can use this document as a technical baseline for software verification under an ISO 13485 / IEC 62304 Quality Management System.

---

## 2. Functional Requirements (REQ-FUN)

| Requirement ID | Title | Description | Priority |
| :--- | :--- | :--- | :--- |
| **REQ-FUN-001** | In-Memory DICOM Parsing & Serialization | The system SHALL parse DICOM datasets into in-memory Rust structures (`InMemDicomObject`) via `dicom-rs` and serialize mutated datasets cleanly to output streams. | High |
| **REQ-FUN-002** | Declarative JSON DSL Specification | The system SHALL parse and execute structured JSON transformation specifications (`TransformSpec`) containing valid `Action` items (`SetTag`, `RemoveTag`, `ReplaceValue`, `AnonymizePatient`, `LoadDataset`, `SaveDataset`, `SaveMap`, `ExtractPixels`, `SaveJson`, `Assemble`, `DumpDataset`). | High |
| **REQ-FUN-003** | Line-by-Line Script Language | The system SHALL parse and execute line-by-line script commands (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`, `SAVE_JSON`, `ASSEMBLE`, `DUMP`) with syntax error validation. | High |
| **REQ-FUN-004** | Nested Sequence Path Traversal | The system SHALL evaluate path expressions for nested DICOM Sequence elements (VR = `SQ`) supporting keyword/hex selectors, item indexing (`[n]`), and wildcard scanning across all sequence items (`/`). | High |
| **REQ-FUN-005** | Dynamic Macro Expansion | The system SHALL dynamically evaluate runtime value generation macros (`$uid`, `$today`, `$rand_str(n)`, `$rand_num(low, high)`, `$rand_time()`) when executing `SET` or `REPLACE` operations. | High |
| **REQ-FUN-006** | De-Identification Audit Mapping | The system SHALL maintain an in-memory audit ledger (`AnonymizationMap`) recording pre-transformation values, modified output values, and tag paths, exportable to JSON (`map.json`). | High |
| **REQ-FUN-007** | RPN Predicate Logic & Sub-Scripting | The system SHALL evaluate reverse-polish notation (RPN) boolean predicates (`CHECK <tag> MATCHES/EXISTS/DATE_*`), stack operations (`AND`, `OR`, `XOR`, `NOT`, `DUP`, `DROP`, `CLEAR`), and conditional sub-script execution (`IF_TRUE`, `IF_FALSE`). | High |
| **REQ-FUN-008** | Model Context Protocol (MCP) Server | The system SHALL provide an interactive REPL console and JSON-RPC Stdio transport compatible with the Model Context Protocol (MCP) specification for AI agent integration. | High |
| **REQ-FUN-009** | Pixel Frame Extraction | The system SHALL extract raw pixel data buffers from DICOM datasets (`ExtractPixels`) into uncompressed or raw binary output files for downstream processing. | Medium |
| **REQ-FUN-010** | CLI Subcommand Interface | The system SHALL provide command-line interfaces for dataset transformation (`run`), REPL interaction (`console`), syntax validation (`validate`), script compilation (`compile`), schema generation (`schema`), and host MCP installer (`install-mcp`). | High |

---

## 3. Performance & Quality Requirements (REQ-PERF)

| Requirement ID | Title | Performance Metric |
| :--- | :--- | :--- |
| **REQ-PERF-001** | Transformation Throughput | DICOM dataset parsing, sequence tag mutation, and macro evaluation SHALL execute in <50ms per instance for standard diagnostic modalities (CT, MR, CR). |
| **REQ-PERF-002** | Zero-Disk Staging Memory Bounds | De-identification operations SHALL execute strictly in volatile Rust RAM without creating intermediate unencrypted PHI temporary disk files (`/tmp`). |
| **REQ-PERF-003** | MCP Stdio Response Latency | Stdio JSON-RPC REPL tool execution responses SHALL be delivered within <100ms per action invocation. |

---

## 4. Regulatory & Data Standards Compliance (REQ-REG)

| Requirement ID | Standard | Requirement Description |
| :--- | :--- | :--- |
| **REQ-REG-001** | NEMA PS3.15 Annex E | Tag modification and anonymization capabilities SHALL enable compliance with HIPAA Safe Harbor and NEMA PS3.15 Annex E de-identification profiles. |
| **REQ-REG-002** | DICOM Part 5 Standard | The engine SHALL strictly observe DICOM Value Representation (VR) definitions and Transfer Syntax encodings during element insertion and replacement. |

