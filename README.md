# dicom-rs-transformer

Programmatically transform DICOM objects for anonymization or value changes with an MCP-enabled CLI console included.

## Features

- **Library & API**: Clean, well-documented Rust library (`dicom-rs-transformer`) designed for developers building DICOM data transformation pipelines.
- **JSON DSL Rules**: Define reusable DICOM transformation specifications in structured JSON format.
- **Line-by-Line Script Language**: Write simple, human-readable text scripts (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`, `SAVE_JSON`, `DUMP`) for batch processing or interactive tool use.
- **RPN Boolean Logic & Sub-Script Branching (PRO)**: Execute complex conditional predicate logic (`CHECK <tag> MATCHES/EXISTS/DATE_*`), stack operators (`AND`, `OR`, `XOR`, `NOT`, `DUP`, `DROP`, `CLEAR`), and sub-script branching (`IF_TRUE`, `IF_FALSE`) with compiled sub-script caching.
- **MCP Terminal Friendly**: Includes an interactive REPL console compatible with Model Context Protocol (MCP) tool pipelines.
- **Powered by `dicom-rs`**: Built on top of the latest [`dicom-rs`](https://github.com/Enet4/dicom-rs) (`v0.10.0`) ecosystem.

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
- [Security Architecture & MCP Guidelines](docs/designs/security.md)
- [System Design & ISO Architecture Document](docs/designs/System-Design-Document.md)
- [Transformer DSL & Script Language Guide](docs/transformer-dsl-guide/transformer-dsl.md)
- Generated API documentation: Run `cargo doc --open` locally.

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](LICENSE) file for details.

> **NOTICE:** This software is for educational, research, or informational purposes only.
> It is NOT certified as a medical device and is NOT intended to diagnose, treat,
> cure, or prevent any disease or medical condition.


