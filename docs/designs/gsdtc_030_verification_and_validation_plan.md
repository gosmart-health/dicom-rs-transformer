# Software Verification & Validation (V&V) Plan

**Document ID:** VVP-DRT-001  
**Project:** `dicom-rs-transformer`  
**Regulatory Standard Alignment:** IEC 62304 Clause 5.5 / 5.6 / 5.7, FDA 21 CFR 820.30  

---

## 1. Introduction & Strategy

This document outlines the Verification & Validation strategy for `dicom-rs-transformer`. The goal is to verify that all functional, performance, safety, and regulatory data requirements defined in the SRS (`SRS-DRT-001`) are satisfied without defect.

---

## 2. Testing Levels & Protocol Scope

### 2.1 Unit Testing (Level 1)
* **Scope:** Core Rust modules (`src/dsl.rs`, `src/script.rs`, `src/engine.rs`, `src/macro_eval.rs`, `src/map.rs`, `src/pixels.rs`).
* **Framework:** Standard Rust testing framework (`#[test]`).
* **Coverage Target:** High branch and line coverage across parser, sequence path traversal, macro expansion, and audit map logic.

### 2.2 Integration & Pipeline Testing (Level 2)
* **Scope:** End-to-end line script parsing, JSON DSL compilation, dataset loading, sequence attribute transformations, and audit map generation.
* **Verification Methods:**
  * Multi-action JSON DSL execution (`tests/transformation_tests.rs`).
  * Line script transpilation and validation via CLI subcommands (`validate`, `compile`).
  * Sequence path wildcard scanning across multi-item sequences (`RequestAttributesSequence`).

### 2.3 Security & Compliance Auditing (Level 3)
* **Scope:** Dependency vulnerability auditing and Software Bill of Materials (SBOM) generation.
* **Verification Methods:**
  * `cargo-audit`: Scans Rust dependency graph against the RustSec Advisory Database.
  * `cargo-cyclonedx`: Generates CycloneDX 1.3 JSON format Software Bill of Materials.
  * Automated compliance script execution (`scripts/generate_compliance_artifacts.sh`).

---

## 3. Automated Test Protocols & Commands

| Test Suite | Execution Command | Description |
| :--- | :--- | :--- |
| **Cargo Unit & Integration Suite** | `cargo test` | Executes unit tests and integration pipelines across all engine components. |
| **CLI Script Validation** | `cargo run -- validate --script sample_script.txt` | Validates syntax correctness of line script without dataset mutation. |
| **Script Compilation** | `cargo run -- compile --script sample_script.txt -o spec.json` | Transpiles line script into canonical JSON spec. |
| **Security Advisory Audit** | `cargo audit` | Checks Rust crate dependencies for known CVE security advisories. |
| **CycloneDX SBOM Generation** | `./scripts/generate_compliance_artifacts.sh` | Generates compliance reports in `compliance-reports/`. |

---

## 4. Acceptance Criteria

1. All automated test suites (`cargo test`) pass with **0 failures**.
2. Script parser (`validate`) correctly rejects invalid syntax lines with informative line-number error messages.
3. Generated `map.json` audit logs accurately match transformed tag paths and pre/post values.
4. Dependency security audit (`cargo audit`) reports 0 unmitigated vulnerabilities.

