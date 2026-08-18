#!/usr/bin/env bash
set -e

# Path definitions
INPUT_DIR="target/dicom_test_files/pydicom"
OUTPUT_DIR="target/dicom_test_files/anonymized"
SCRIPT_FILE="examples/batch_anonymize.txt"

echo "============================================================"
echo "DICOM Batch Transformation Demo Runner"
echo "============================================================"

# Step 1: Build binary
echo "[1/4] Building dicom-transformer CLI..."
cargo build --quiet

# Step 2: Download test files to target directory
echo "[2/4] Downloading sample DICOM test files into '${INPUT_DIR}'..."
cargo run --quiet -- download-test-files --destination "${INPUT_DIR}"

# Step 3: Run interactive console batch execution using the text script
echo "[3/4] Running batch anonymization script '${SCRIPT_FILE}'..."
cargo run --quiet -- console --input "${INPUT_DIR}" < "${SCRIPT_FILE}"

# Step 4: Verify output results
echo "[4/4] Verifying output directory results..."
if [ -d "${OUTPUT_DIR}" ]; then
    FILE_COUNT=$(find "${OUTPUT_DIR}" -maxdepth 1 -type f -name "*.dcm" | wc -l | tr -d ' ')
    echo "  Processed DICOM files in output folder (${OUTPUT_DIR}): ${FILE_COUNT}"
fi

if [ -f "target/dicom_test_files/anonymized_audit_map.json" ]; then
    echo "  Anonymization audit map created successfully at target/dicom_test_files/anonymized_audit_map.json"
fi

echo "============================================================"
echo "Scenario Execution Complete!"
echo "============================================================"
