# Changelog

All notable changes to `dicom-rs-transformer` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1-alpha-7] - 2026-08-24

### Added
- **JSON DSL Transformation Engine**: Declarative JSON rule definitions for DICOM dataset transformations.
- **Line-by-Line Scripting Language**: Human-readable script language (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`, `SAVE_JSON`, `DUMP`).
- **RPN Boolean Logic & Sub-Script Branching**: Conditional predicates (`CHECK <tag> MATCHES/EXISTS/DATE_*`), stack operators (`AND`, `OR`, `XOR`, `NOT`, `DUP`, `DROP`, `CLEAR`), and sub-script execution (`IF_TRUE`, `IF_FALSE`).
- **Model Context Protocol (MCP) Integration**: Native stdio MCP server mode and automatic IDE/Tool configuration via `dicom-transformer install-mcp --target all`.
- **Pre-compiled Binary Releases**: Automated multi-platform GitHub release pipeline for Linux (x86_64, musl, aarch64), macOS (Intel, Apple Silicon), and Windows (x86_64).
- **Compliance & Security Tooling**: Automated dependency security auditing (`cargo-audit`) and CycloneDX SBOM generation (`cargo-cyclonedx`).

## [0.1.0] - 2026-08-24

### Added
- **JSON DSL Transformation Engine**: Declarative JSON rule definitions for DICOM dataset transformations.
- **Line-by-Line Scripting Language**: Human-readable script language (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`, `SAVE_JSON`, `DUMP`).
- **RPN Boolean Logic & Sub-Script Branching**: Conditional predicates (`CHECK <tag> MATCHES/EXISTS/DATE_*`), stack operators (`AND`, `OR`, `XOR`, `NOT`, `DUP`, `DROP`, `CLEAR`), and sub-script execution (`IF_TRUE`, `IF_FALSE`).
- **Model Context Protocol (MCP) Integration**: Native stdio MCP server mode and automatic IDE/Tool configuration via `dicom-transformer install-mcp --target all`.
- **Pre-compiled Binary Releases**: Automated multi-platform GitHub release pipeline for Linux (x86_64, musl, aarch64), macOS (Intel, Apple Silicon), and Windows (x86_64).
- **Compliance & Security Tooling**: Automated dependency security auditing (`cargo-audit`) and CycloneDX SBOM generation (`cargo-cyclonedx`).
