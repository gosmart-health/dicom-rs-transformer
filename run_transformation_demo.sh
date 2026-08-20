#!/usr/bin/env bash
set -e

# Path definitions
INPUT_DIR="target/dicom_test_files/pydicom"
OUTPUT_DIR="target/dicom_test_files/anonymized"
JSON_DIR="target/dicom_test_files/json"
PIXELS_DIR="target/dicom_test_files/pixels"
MAPS_DIR="target/dicom_test_files/maps"
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
    echo "  ✔ Anonymized DICOM files created (${OUTPUT_DIR}): ${FILE_COUNT}"
fi

if [ -d "${JSON_DIR}" ]; then
    JSON_COUNT=$(find "${JSON_DIR}" -maxdepth 1 -type f -name "*.json" | wc -l | tr -d ' ')
    echo "  ✔ DICOM JSON metadata files created (${JSON_DIR}): ${JSON_COUNT}"
fi

if [ -d "${PIXELS_DIR}" ]; then
    PIXEL_FOLDERS=$(find "${PIXELS_DIR}" -maxdepth 1 -type d | tail -n +2 | wc -l | tr -d ' ')
    echo "  ✔ Extracted pixel payload folders created (${PIXELS_DIR}): ${PIXEL_FOLDERS}"
fi

if [ -d "${MAPS_DIR}" ]; then
    MAP_COUNT=$(find "${MAPS_DIR}" -maxdepth 1 -type f -name "*.json" | wc -l | tr -d ' ')
    echo "  ✔ Anonymization audit maps created (${MAPS_DIR}): ${MAP_COUNT}"
fi

echo "============================================================"
echo "Scenario Execution Complete!"
echo "============================================================"
