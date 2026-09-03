# Changelog

All notable changes to `dicom-rs-transformer` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-03

### Added
- **Full DICOM PS3.15 Annex E De-Identification Engine**:
  - Implemented structured data models (`DeidentificationProfile`, `DeidentificationConfig`, `TableE11Rule`, `ProfileOptions`, `ActionCode`) for DICOM PS3.15 Annex E Table E.1-1 profile rules.
  - Implemented tag action code execution (`X` remove, `Z` zero-length, `D` dummy string `$rand_str(8)`, `C` clean `"CLEANED"`, `U` generate OID UID `$uid`, `K` keep).
  - Added option override action resolution (`TableE11Rule::resolve_action`) supporting runtime profile flags (`retain_uids`, `retain_safe_private`, `clean_desc`, etc.).
  - Added default flat-out private data element removal (`group % 2 != 0`) unless `retain_safe_private` option is enabled or tag has an explicit `K` rule.
  - Built JSON profile loader with compile-time zero-disk embedded fallback (`configs/anonymization_profile.current.json`).
- **New DSL & Line Script Commands**:
  - `LOAD_DI_PROFILE` / `load_di_profile`: Loads a custom DICOM de-identification profile from JSON file or URI (defaults to embedded profile).
  - `DEIDENTIFY` / `deidentify`: Applies PS3.15 Annex E de-identification profile rules to the active DICOM dataset.
  - `ASSEMBLE` / `assemble`: Reassembles DICOM dataset(s) from JSON metadata headers and raw pixel data back into memory with optional local save or PACS push.
  - `FETCH` / `fetch` & `PUSH_DATASET` / `push_dataset`: DIMSE C-FIND / C-MOVE query-retrieve and C-STORE dataset dispatch commands with PRO edition feature bounds.
- **Model Context Protocol (MCP) Server Integration**:
  - Registered `load_di_profile` and `deidentify` tools in MCP tool discovery schema (`schema` subcommand and stdio JSON-RPC server).
- **ISO/FDA Design Controls Documentation Suite**:
  - Standardized regulatory documentation under `docs/designs/` with `gsdtc_` prefix (`gsdtc_000_software_requirements_spec.md`, `gsdtc_010_system_design_specification.md`, `gsdtc_020_hazard_analysis_risk_management.md`, `gsdtc_030_verification_and_validation_plan.md`, `gsdtc_040_traceability_matrix.md`, `gsdtc_050_cybersecurity_and_soup_bom.md`).

### Fixed & Refactored
- **Single Dataset Save File Extension Handling**: Fixed bug where saving a single dataset item to a non-existent path without an extension did not auto-append default extensions (`.dcm` for datasets, `.json` for metadata/maps).
- **Model Naming Cleanup**: Eliminated legacy `Shade` / `ShadeDeidentificationProfile` naming references across the codebase in favor of `DeidentificationProfile`.

## [0.0.1-alpha-9] - 2026-08-24

### Added
- **JSON DSL Transformation Engine**: Declarative JSON rule definitions for DICOM dataset transformations.
- **Line-by-Line Scripting Language**: Human-readable script language (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`, `SAVE_JSON`, `DUMP`).
- **RPN Boolean Logic & Sub-Script Branching**: Conditional predicates (`CHECK <tag> MATCHES/EXISTS/DATE_*`), stack operators (`AND`, `OR`, `XOR`, `NOT`, `DUP`, `DROP`, `CLEAR`), and sub-script execution (`IF_TRUE`, `IF_FALSE`).
- **Model Context Protocol (MCP) Integration**: Native stdio MCP server mode and automatic IDE/Tool configuration via `dicom-transformer install-mcp --target all`.
- **Pre-compiled Binary Releases**: Automated multi-platform GitHub release pipeline for Linux (x86_64, musl, aarch64), macOS (Apple Silicon), and Windows (x86_64).
- **Compliance & Security Tooling**: Automated dependency security auditing (`cargo-audit`) and CycloneDX SBOM generation (`cargo-cyclonedx`).
