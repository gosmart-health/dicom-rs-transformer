# dicom-rs-transformer

Programmatically transform DICOM objects for anonymization or value changes with an MCP-enabled CLI console included.

## Features

- **Library & API**: Clean, well-documented Rust library (`dicom-rs-transformer`) designed for developers building DICOM data transformation pipelines.
- **JSON DSL Rules**: Define reusable DICOM transformation specifications in structured JSON format.
- **Line-by-Line Script Language**: Write simple, human-readable text scripts (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`, `SAVE_JSON`, `ASSEMBLE`, `DUMP`) for batch processing or interactive tool use.
- **RPN Boolean Logic & Sub-Script Branching (PRO)**: Execute complex conditional predicate logic (`CHECK <tag> MATCHES/EXISTS/DATE_*`), stack operators (`AND`, `OR`, `XOR`, `NOT`, `DUP`, `DROP`, `CLEAR`), and sub-script branching (`IF_TRUE`, `IF_FALSE`) with compiled sub-script caching.
- **MCP Terminal Friendly**: Includes an interactive REPL console compatible with Model Context Protocol (MCP) tool pipelines.
- **Powered by `dicom-rs`**: Built on top of the latest [`dicom-rs`](https://github.com/Enet4/dicom-rs) (`v0.10.0`) ecosystem.

> [!CAUTION]
> **Single-User & Single-Dataset Execution Model (Community Edition)**:
> In Community Edition, `dicom-rs-transformer` is designed for transformation work by a **single user** operating on a **single dataset** (or sequential directory batch) at a time with global in-memory state. Thread-safe concurrent multi-session partitioning and multi-tenant isolation are implemented in the **PRO version** (`dicom-rs-transformer-pro`). Do not expose a Community Edition instance directly as a shared multi-user service without isolated worker processes.

## Installation

### Option 1: Pre-built Binaries (Recommended)

Pre-compiled standalone binaries for **Linux**, **macOS** (Intel & Apple Silicon), and **Windows** are available on the [GitHub Releases](https://github.com/gosmart-health/dicom-rs-transformer/releases) page.

1. Download the archive for your operating system and architecture (`.tar.gz` for Linux/macOS, `.zip` for Windows).
2. Extract the archive and place `dicom-transformer` in your `PATH` (or run it directly).
3. (Optional) Automatically configure MCP integration for your local AI tools (Antigravity IDE, Cursor, Claude Desktop):
   ```bash
   dicom-transformer install-mcp --target all
   ```

### Option 2: Build & Install via Cargo

If you have the Rust toolchain installed:

```bash
cargo install --path .
```

---

## De-Identification Configuration

De-identification is controlled by `configs/anonymization_profile.current.json`, which implements the [DICOM PS3.15 Annex E](https://dicom.nema.org/medical/dicom/current/output/chtml/part15/chapter_E.html) basic profile. You can customize this file to adjust which tags are retained, cleared, or removed.

By default, all private data elements encountered during processing are removed. You can override this behavior using the `K` (Keep) directive to retain specific private tags. Exercise caution when doing so: the de-identification engine does not inspect the contents of private data elements, which may inadvertently expose Protected Health Information (PHI).

> **Important:** The configuration profile does not support direct value replacement (such as remapping `PatientID` to a new pseudonym). To assign custom values, run the de-identification step first to blank or remove the original attribute, then apply the new value in a post-processing step.

This configuration file was generated using the GoSmart.Health [dicom-py-anonymizer-kit](https://github.com/gosmart-health/dicom-py-anonymizer-kit) against the current DICOM standard. Profiles can also be generated against earlier DICOM releases if needed.

--

## Quick Start for Developers

Whether you are an experienced Rust developer or completely new to the language, check out our step-by-step setup guide:

👉 **[Developer Initial Setup Guide](docs/developing/initial-setup.md)**

### Automatic MCP Registration

Register `dicom-transformer` into your local AI developer tools (Antigravity IDE, Cursor, Claude Desktop) with a single command:

```bash
cargo run -- install-mcp --target all
```

### Running the Example

```bash
cargo run --example transform_sample
```

### Running the Interactive Console

```bash
cargo run -- console
```

### Running Tests

```bash
cargo test
```

## Security & Compliance

This repository includes automated Rust dependency security auditing (`cargo-audit`) and CycloneDX Software Bill of Materials (SBOM) generation (`cargo-cyclonedx`).

### Running Compliance Artifacts Locally

Run the compliance script to execute security scans and generate a CycloneDX SBOM:

```bash
./scripts/generate_compliance_artifacts.sh
```

This generates:
- `compliance-reports/cargo-audit-report.json` (Vulnerability audit report against RustSec advisory database)
- `compliance-reports/cargo-cyclonedx.json` (CycloneDX 1.3 JSON format Software Bill of Materials)

### Continuous Integration

- **GitHub Actions**: Automated PR and push scans are configured in [.github/workflows/compliance.yml](.github/workflows/compliance.yml).

## Documentation

- [Developer Initial Setup Guide](docs/developing/initial-setup.md)
- [Development Workflow & Branching Strategy](docs/developing/development-workflow.md)
- [Design Controls & Regulatory Documentation Suite](docs/designs/README.md)
  - [Software Requirements Specification (SRS)](docs/designs/gsdtp_000_software_requirements_spec.md)
  - [System Design Specification (SDS)](docs/designs/gsdtp_010_system_design_specification.md)
  - [Hazard Analysis & Risk Management Plan](docs/designs/gsdtp_020_hazard_analysis_risk_management.md)
  - [Software Verification & Validation Plan](docs/designs/gsdtp_030_verification_and_validation_plan.md)
  - [Requirements Traceability Matrix (RTM)](docs/designs/gsdtp_040_traceability_matrix.md)
  - [Cybersecurity Profile & SOUP SBOM](docs/designs/gsdtp_050_cybersecurity_and_soup_bom.md)
- [Transformer DSL & Script Language Guide](docs/transformer-dsl-guide/transformer-dsl.md)
- Generated API documentation: Run `cargo doc --open` locally.

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.

> **NOTICE:** This software is for educational, research, or informational purposes only.
> It is NOT certified as a medical device and is NOT intended to diagnose, treat,
> cure, or prevent any disease or medical condition.

# Contacting the Developer Community

Join [Discussions](https://github.com/gosmart-health/dicom-rs-transformer/discussions)

Add or Inspect [Issues](https://github.com/gosmart-health/dicom-rs-transformer/issues)
