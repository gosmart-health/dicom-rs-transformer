# Cybersecurity Profile & Software Bill of Materials (SBOM)

**Document ID:** SEC-DRT-001  
**Project:** `dicom-rs-transformer`  
**Regulatory Standard Alignment:** FDA Cybersecurity in Medical Devices Guidance (2023), IEC 62304 SOUP Evaluation  

---

## 1. Executive Summary & Security Model

This document details the Software Bill of Materials (SBOM), SOUP (Software of Unknown Provenance) risk management, cybersecurity controls, and privacy safeguards for `dicom-rs-transformer`.

---

## 2. Software Bill of Materials (SBOM) / SOUP Inventory

| Component Name | Version / Spec | License | Source / Repository | Purpose | SOUP Risk Assessment |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`dicom-rs`** | `v0.10.0` | Apache-2.0 / MIT | `github.com/Enet4/dicom-rs` | In-memory DICOM parsing, dictionary lookup, VR encoding | Low risk; widely adopted open-source DICOM library in Rust. |
| **`serde` / `serde_json`** | `^1.0` | Apache-2.0 / MIT | `crates.io/crates/serde` | JSON DSL specification parsing and audit map serialization | Low risk; standard Rust serialization ecosystem. |
| **`clap`** | `^4.4` | Apache-2.0 / MIT | `crates.io/crates/clap` | Command-line argument parsing and subcommand dispatching | Low risk; official standard Rust CLI argument parser. |
| **`tokio`** | `^1.35` | MIT | `crates.io/crates/tokio` | Async runtime for object store communication and REPL IO | Low risk; core Rust async runtime crate. |
| **`uuid`** | `^1.6` | Apache-2.0 / MIT | `crates.io/crates/uuid` | UUID v4 generation for `$uid` OID macro expansion | Low risk; standard Rust UUID crate. |
| **`object_store`** | `^0.9` | Apache-2.0 / MIT | `crates.io/crates/object_store` | Abstract storage operations across local disk, AWS S3, GCS, Azure | Low risk; Apache Arrow ecosystem storage library. |

---

## 3. Cybersecurity & Data Integrity Safeguards

### 3.1 Zero-Disk Staging & In-Memory Execution
* `dicom-rs-transformer` parses, mutates, and serializes DICOM datasets strictly in volatile process memory (`InMemDicomObject`).
* Unencrypted Protected Health Information (PHI) is **never written to temporary local disk files (`/tmp`) or swap buffers** during pipeline execution, eliminating residual storage risk upon crash or power failure.

### 3.2 Customer Data Sovereignty & Zero Telemetry
* The binary runs locally on user workstations, private cloud VPCs (AWS, GCP, Azure), or air-gapped hospital VLANs.
* **No data, telemetry, or DICOM payloads are ever transmitted back to external third-party servers.**

### 3.3 Memory Safety by Design (Rust)
* Built natively in 100% safe Rust, eliminating buffer overflows, use-after-free vulnerabilities, memory corruption, and data races common in legacy C/C++ medical imaging software.

### 3.4 Automated Vulnerability & SBOM Tooling
* **`cargo-audit`**: Checks dependencies against the RustSec Advisory Database.
* **`cargo-cyclonedx`**: Generates CycloneDX 1.3 JSON format Software Bill of Materials.
* Scans are executed via `./scripts/generate_compliance_artifacts.sh` and enforced via continuous integration workflows.

