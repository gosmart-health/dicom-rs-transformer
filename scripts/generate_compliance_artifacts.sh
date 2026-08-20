#!/usr/bin/env bash
set -euo pipefail

# Ensure script runs from project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Output directory setup
OUTPUT_DIR="compliance-reports"
mkdir -p "$OUTPUT_DIR"

echo "=== Rust Dependency Security Audit & Software BOM (SBOM) Generation ==="
echo "Output directory: $OUTPUT_DIR"
echo ""

# 1. Dependency Vulnerability Audit (cargo-audit)
if command -v cargo-audit >/dev/null 2>&1; then
    echo "Running cargo audit against RustSec Advisory Database..."
    cargo audit --json > "$OUTPUT_DIR/cargo-audit-report.json" || echo "⚠ Cargo audit detected advisories or warnings."
    echo "✔ Vulnerability report saved to $OUTPUT_DIR/cargo-audit-report.json"
else
    echo "ℹ 'cargo-audit' is not installed. Installing via 'cargo install cargo-audit --locked'..."
    cargo install cargo-audit --locked
    cargo audit --json > "$OUTPUT_DIR/cargo-audit-report.json" || echo "⚠ Cargo audit detected advisories or warnings."
    echo "✔ Vulnerability report saved to $OUTPUT_DIR/cargo-audit-report.json"
fi

echo ""

# 2. CycloneDX Software Bill of Materials (cargo-cyclonedx)
if command -v cargo-cyclonedx >/dev/null 2>&1; then
    echo "Generating CycloneDX Software Bill of Materials (SBOM)..."
    cargo cyclonedx --format json
    if [ -f "dicom-rs-transformer.cdx.json" ]; then
        mv "dicom-rs-transformer.cdx.json" "$OUTPUT_DIR/cargo-cyclonedx.json"
    fi
    echo "✔ CycloneDX SBOM saved to $OUTPUT_DIR/cargo-cyclonedx.json"
else
    echo "ℹ 'cargo-cyclonedx' is not installed. Installing via 'cargo install cargo-cyclonedx'..."
    cargo install cargo-cyclonedx
    cargo cyclonedx --format json
    if [ -f "dicom-rs-transformer.cdx.json" ]; then
        mv "dicom-rs-transformer.cdx.json" "$OUTPUT_DIR/cargo-cyclonedx.json"
    fi
    echo "✔ CycloneDX SBOM saved to $OUTPUT_DIR/cargo-cyclonedx.json"
fi

echo ""
echo "=== Compliance Artifacts Summary ==="
ls -la "$OUTPUT_DIR"
