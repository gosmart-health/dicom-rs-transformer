# DICOM Transformer DSL Specification & Language Guide

This guide documents the **Transformation Domain Specific Language (DSL)** and the equivalent **Line-by-Line Script Language** supported by `dicom-rs-transformer`.

---

## Overview

The Transformer DSL allows developers and automated agents to define precise DICOM dataset transformation pipelines (such as tag editing, anonymization, value replacements, and tag deletions).

The DSL supports two formats:

1. **Structured JSON DSL**: Full declarative format ideal for REST APIs, stored configuration files, and programmatic integration.
2. **Line-by-Line Script Syntax**: Clean, human-readable text format designed for batch script processing, terminal CLIs, and Model Context Protocol (MCP) console interactions.

---

## 1. Targeting DICOM Tags (`TagSelector` & Path Expressions)

You can target DICOM dataset attributes using single tags or multi-level sequence path expressions:

| Format | Example | Description |
| :--- | :--- | :--- |
| **Keyword** | `"PatientName"` | Standard DICOM element name resolved via the standard dictionary. |
| **Hex String** | `"(0010,0010)"` or `"0010,0010"` | Group and element hexadecimal pair string. |
| **Hex Integer Pair** | `{"group": 16, "element": 16}` | Explicit decimal or hex integer representation (`0x0010 = 16`). |
| **Sequence Item Path** *(PRO)* | `"RequestAttributesSequence[0]/ScheduledProcedureStepID"` | 🔒 **PRO Feature**: Targets item index `0` within a sequence element. |
| **Sequence Wildcard Path** *(PRO)* | `"RequestAttributesSequence/ScheduledProcedureStepID"` or `"RequestAttributesSequence[*]/ScheduledProcedureStepID"` | 🔒 **PRO Feature**: Omitting item index scans through **every sequence item** and applies the transformation. |

---

## 2. Transformation Actions (`Action`)

### A. Load Dataset (`load_dataset`)

Loads a DICOM dataset from a local Unix/Windows file path or cloud URI (`s3://`, `gs://`, `az://`).

> [!NOTE]
> Local filesystem paths (`/path/to/file.dcm` or `file://...`) are supported in Community Edition. Cloud URIs (`s3://`, `gs://`, `az://`, `dicom://`) are 🔒 **PRO Features**.

#### JSON DSL
```json
{
  "op": "load_dataset",
  "location": "s3://clinical-trials-bucket/incoming/subject_01.dcm"
}
```

#### Line Script Equivalent
```text
LOAD "s3://clinical-trials-bucket/incoming/subject_01.dcm"
LOAD "/var/dicom/local_input.dcm"
```

---

### B. Save Dataset (`save_dataset`)

Saves the current DICOM dataset to a local Unix/Windows file path or cloud URI (`s3://`, `gs://`, `az://`).

#### JSON DSL
```json
{
  "op": "save_dataset",
  "location": "gs://anonymized-dicom-vault/processed/subject_01.dcm"
}
```

#### Line Script Equivalent
```text
SAVE "gs://anonymized-dicom-vault/processed/subject_01.dcm"
SAVE "/var/dicom/local_output.dcm"
```

---

### Cloud Storage Authentication & Environment Variables

`dicom-rs-transformer` integrates with Apache Arrow's [`object_store`](https://docs.rs/object_store/latest/object_store/) crate to seamlessly parse cloud URIs (`s3://`, `gs://`, `az://`) and read credentials directly from standard environment variables.

#### Amazon S3 (`s3://bucket/key.dcm`)
Requires **all 3** primary AWS environment variables for static key authentication:
- `AWS_ACCESS_KEY_ID`: *(Required)* AWS access key ID.
- `AWS_SECRET_ACCESS_KEY`: *(Required)* AWS secret access key.
- `AWS_DEFAULT_REGION` (or `AWS_REGION`): *(Required)* Target AWS region (e.g., `us-east-1`).
- `AWS_SESSION_TOKEN`: *(Optional)* Temporary session token for IAM roles or STS credentials.

```bash
# Set all 3 required variables for S3
export AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
export AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
export AWS_DEFAULT_REGION="us-east-1"
```

#### Google Cloud Storage (`gs://bucket/key.dcm` or `gcs://...`)
Requires **only 1** of the following environment variable options (GCP automatically resolves authentication from the specified path or ADC):
- `GOOGLE_APPLICATION_CREDENTIALS`: *(Recommended)* Path to GCP service account JSON key file or Application Default Credentials.
- **OR** `GOOGLE_SERVICE_ACCOUNT`: Path to Google Cloud service account JSON key file.
- **OR** `GOOGLE_SERVICE_ACCOUNT_KEY`: Raw JSON string containing service account credentials.

```bash
# Set ONLY ONE variable for GCS
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/gcp_key.json"
```

#### Azure Blob Storage (`az://container/blob.dcm` or `abfs://...`)
Supports two authentication modes:

1. **Storage Account Access Key Mode (Requires both 2 variables)**:
   - `AZURE_STORAGE_ACCOUNT_NAME`: *(Required)* Azure Storage Account name.
   - `AZURE_STORAGE_ACCOUNT_KEY`: *(Required)* Storage account secret key.

   ```bash
   export AZURE_STORAGE_ACCOUNT_NAME="myaccount"
   export AZURE_STORAGE_ACCOUNT_KEY="secretkey=="
   ```

2. **Service Principal Mode (Requires all 4 variables)**:
   - `AZURE_STORAGE_ACCOUNT_NAME`: *(Required)* Storage Account name.
   - `AZURE_STORAGE_CLIENT_ID`: *(Required)* Azure AD Client/Application ID.
   - `AZURE_STORAGE_CLIENT_SECRET`: *(Required)* Client Secret.
   - `AZURE_STORAGE_TENANT_ID`: *(Required)* Azure AD Tenant ID.

> [!TIP]
> For advanced configuration options (such as custom S3-compatible endpoints, MinIO, or bearer tokens), refer to the official [Apache Arrow Rust `object_store` documentation](https://docs.rs/object_store/latest/object_store/).

---

### C. Set Tag Value (`set_tag`)

Assigns or updates a DICOM element value. If the tag is not present in the dataset, it is added automatically using standard Value Representation (VR) definitions.

#### JSON DSL
```json
{
  "op": "set_tag",
  "selector": "PatientName",
  "value": "ANONYMOUS^PATIENT"
}
```

#### Line Script Equivalent
```text
SET PatientName "ANONYMOUS^PATIENT"
SET (0010,0020) = "ANON-1002"
```

---

### B. Remove Tag (`remove_tag`)

Deletes a targeted DICOM element from the dataset if it exists.

#### JSON DSL
```json
{
  "op": "remove_tag",
  "selector": "PatientAddress"
}
```

#### Line Script Equivalent
```text
DELETE PatientAddress
REMOVE (0010,1040)
```

---

### C. Replace Substring Value (`replace_value`)

Searches for matching substring patterns within string-based tag values and replaces them with a specified replacement string.

> [!NOTE]
> **Non-Matching & Missing Tag Behavior**:
> - If the target tag exists but the pattern is **not found** in the value, the string remains unchanged. Pipeline execution **does not stop**, and subsequent actions continue.
> - If the target tag is **missing** from the dataset, the action is skipped safely without stopping the pipeline.

#### JSON DSL
```json
{
  "op": "replace_value",
  "selector": "StudyDescription",
  "pattern": "HOSPITAL_NORTH",
  "replacement": "CLINIC_SITE_A"
}
```

#### Line Script Equivalent
```text
REPLACE StudyDescription "HOSPITAL_NORTH" WITH "CLINIC_SITE_A"
```

---

### D. Anonymize Patient Information (`anonymize_patient`)

Standardized action that replaces standard DICOM patient identification fields (`PatientName` `(0010,0010)` and `PatientID` `(0010,0020)`).

#### JSON DSL
```json
{
  "op": "anonymize_patient",
  "patient_name": "ANON^SUBJECT",
  "patient_id": "SUBJECT-987"
}
```

#### Line Script Equivalent
```text
ANONYMIZE NAME="ANON^SUBJECT" ID="SUBJECT-987"
```

### E. Save Anonymization Map (`save_map`)

Exports the structured JSON audit mapping file linking original tag values to their anonymized replacements. Can be saved locally or uploaded to cloud storage (`s3://`, `gs://`, `az://`).

#### JSON DSL
```json
{
  "op": "save_map",
  "location": "s3://audit-vault/mappings/patient_01_map.json"
}
```

#### Line Script Equivalent
```text
SAVE_MAP "s3://audit-vault/mappings/patient_01_map.json"
EXPORT_MAP "/local/audit/patient_01_map.json"
```

---

### F. Save Dataset to DICOM JSON (`save_json`)

Exports the current dataset to DICOM JSON format (standardized by DICOM Part 18 Chapter F). If raw pixel data exists in the dataset, it is automatically extracted to the specified raw pixel location (or to `<json_file>.raw` if raw pixel location is omitted) and stripped from the JSON payload. Can be written locally or saved to cloud storage (`s3://`, `gs://`, `az://`).

#### JSON DSL
```json
{
  "op": "save_json",
  "json_location": "s3://dicom-vault/output/sample.json",
  "raw_pixel_location": "s3://dicom-vault/output/sample.raw"
}
```

#### Line Script Equivalent
```text
SAVE_JSON "s3://dicom-vault/output/sample.json" "s3://dicom-vault/output/sample.raw"
SAVE_JSON "/local/output/sample.json"
```

---

### G. Dump Dataset (`dump`)

Prints/dumps the dataset structure and values in a human-readable text format (equivalent to `dicom-dump`). Can be saved locally or uploaded to cloud storage (`s3://`, `gs://`, `az://`).

#### JSON DSL
```json
{
  "op": "dump",
  "location": "s3://audit-vault/dumps/sample_dump.txt"
}
```

#### Line Script Equivalent
```text
DUMP "s3://audit-vault/dumps/sample_dump.txt"
DUMP "/local/dumps/sample_dump.txt"
```

---

### H. Extract Pixel Images & Frames (`extract_pixels`)

Extracts single-frame or multi-frame DICOM pixel data into standard JPEG (`0.jpg`, `1.jpg`), PNG (`0.png`, `1.png`), or uncompressed RAW (`0.raw`, `1.raw`) binary files in the target directory or cloud bucket (`s3://`, `gs://`, `az://`). Uses folder-based numbered naming (`0.jpg`, `1.jpg`, ...) to prevent SOPInstanceUID leaks.

#### JSON DSL
```json
{
  "op": "extract_pixels",
  "destination": "s3://ml-dataset/images/patient_01/",
  "format": "jpeg"
}
```

#### Line Script Equivalent
```text
EXTRACT_PIXELS "s3://ml-dataset/images/patient_01/" FORMAT="jpeg"
EXTRACT_PIXELS "/local/dataset/patient_01/" FORMAT="png"
EXTRACT_PIXELS "/local/dataset/patient_01/" FORMAT="raw"
```

---

## 3. Dynamic Value Macros

Value strings in `SET`, `REPLACE`, `ANONYMIZE`, and `SAVE_MAP` actions support dynamic macro expressions prefixed with `$`. To output a literal `$` character, escape it as `$$`.

| Macro | Example Output | Description |
| :--- | :--- | :--- |
| **`$uid`** / **`$UID`** | `2.25.1384029471...` | Generates a valid ISO OID DICOM UID derived from UUID v4. |
| **`$rand_str(n)`** | `KX9A4F` | Generates a random uppercase ASCII string of length `n`. |
| **`$rand_num(low, high)`** | `4921` | Generates a random integer in `[low, high]` inclusive. |
| **`$rand_time()`** | `143022` | Generates a random DICOM formatted time (`HHMMSS`). |
| **`$today(offset)`** | `20260811` | Generates today's date formatted as `YYYYMMDD` +/- offset days (`$today(-5)`). |
| **`$$`** | `$` | Escape sequence evaluating to literal `$`. |

### Macro Examples

```text
SET PatientName "ANON-$rand_str(6)"
SET PatientID "SUBJ-$rand_num(1000, 9999)"
SET StudyInstanceUID "$uid"
SET StudyDate "$today(-30)"
```

---

## 4. Anonymization Audit Mapping (`AnonymizationMap`)

When transforming datasets (especially for AI model training or clinical trials), `dicom-rs-transformer` automatically records an audit log mapping original tag values to anonymized values.

### Sample Audit Map JSON Output (`map.json`)

```json
{
  "source": "s3://clinical-data/raw/patient_01.dcm",
  "timestamp": "2026-08-11T14:28:14Z",
  "entries": [
    {
      "tag": "(0010,0010)",
      "keyword": "PatientName",
      "original_value": "DOE^JOHN",
      "new_value": "ANON-KX9A4F"
    },
    {
      "tag": "(0010,0020)",
      "keyword": "PatientID",
      "original_value": "1234567",
      "new_value": "SUBJ-4921"
    },
    {
      "tag": "(0010,1040)",
      "keyword": "PatientAddress",
      "original_value": "123 MAIN ST",
      "new_value": "[REMOVED]"
    }
  ]
}
```

---

## 5. Complete Transformation Specification (`TransformSpec`)

A complete specification combines metadata (`version`, `name`, `description`) with a sequence of actions executed in order.

### Complete JSON Specification Example

```json
{
  "version": "1.0",
  "name": "Clinical Trial Anonymization",
  "description": "Anonymize patient names, IDs, and strip address fields for clinical trial submission.",
  "actions": [
    {
      "op": "anonymize_patient",
      "patient_name": "TRIAL^SUBJECT^001",
      "patient_id": "SUBJ-001"
    },
    {
      "op": "remove_tag",
      "selector": "PatientAddress"
    },
    {
      "op": "replace_value",
      "selector": "InstitutionName",
      "pattern": "CITY_GENERAL",
      "replacement": "TRIAL_SITE_1"
    }
  ]
}
```

### Complete Line Script Example (`script.txt`)

```text
# Clinical Trial Anonymization Script
# Blank lines and comment lines starting with '#' are ignored

SET PatientName "TRIAL^SUBJECT^001"
SET PatientID "SUBJ-001"
DELETE PatientAddress
REPLACE InstitutionName "CITY_GENERAL" WITH "TRIAL_SITE_1"
```

---

## 4. Usage in CLI & Code

> [!NOTE]
> **Executable Names Across Operating Systems**:
> - **Windows (PowerShell / Command Prompt)**: Run `dicom-transformer.exe` (e.g., `.\dicom-transformer.exe run --input patient.dcm --output output.dcm --script sample_script.txt`).
> - **macOS / Linux**: Run `dicom-transformer` (e.g., `./dicom-transformer run --input patient.dcm --output output.dcm --script sample_script.txt`).
> - **Developing from source**: Prefix commands with `cargo run --` (e.g., `cargo run -- compile --script sample_script.txt -o spec.json`).

### Running DICOM Transformations (`run`)

```bash
# Execute a script against a DICOM file
dicom-transformer run --input patient.dcm --output output.dcm --script sample_script.txt

# Execute a JSON DSL specification against a DICOM file
dicom-transformer run --input patient.dcm --output output.dcm --dsl spec.json
```

### Validating & Compiling Rule Sets Without Executing DICOM Data

```bash
# Validate line script syntax
dicom-transformer validate --script sample_script.txt

# Compile a text script into a canonical, reusable JSON specification
dicom-transformer compile --script sample_script.txt -o spec.json

# Decompile/format a JSON specification into a text script
dicom-transformer compile --dsl spec.json -o script.txt
```

### Discovering MCP Tools & Commands (`schema`)

MCP clients and AI agents can auto-discover all available tool definitions, action names, input schemas, and parameter requirements using the `schema` subcommand:

```bash
dicom-transformer schema
```

In interactive REPL console mode (`dicom-transformer console`), typing **`HELP`** or **`COMMANDS`** lists all supported text script syntax:

```text
> HELP
Available commands: LOAD <path/uri>, SAVE <path/uri>, SET <tag> <value>, DELETE <tag>, REPLACE <tag> <pattern> WITH <replacement>, ANONYMIZE NAME="<name>" ID="<id>"
```

### Executing via Library API (Rust)
```rust
use dicom_rs_transformer::{TransformSpec, DicomTransformer, TransformStatus};

let json_data = std::fs::read_to_string("spec.json")?;
let spec = TransformSpec::from_json(&json_data)?;

let transformer = DicomTransformer::new(spec);
let report = transformer.transform_file(&mut dicom_object)?;

match report.status {
    TransformStatus::Success => println!("All {} actions succeeded!", report.total_actions),
    TransformStatus::Partial => println!("Partial success: {}/{} actions were effective.", report.actions_effective, report.total_actions),
    TransformStatus::None => println!("No actions resulted in changes."),
}
```

---

## 5. Execution Outcome Status (`TransformStatus`)

Every transformation execution returns a structured `TransformReport` containing a `status` field:

| Status Enum | JSON Value | Description |
| :--- | :--- | :--- |
| `TransformStatus::Success` | `"success"` | Every action defined in the specification was effective and modified/removed tags. |
| `TransformStatus::Partial` | `"partial"` | At least one action modified the dataset, but some actions were skipped or no-ops (e.g. pattern didn't match or tag to delete didn't exist). |
| `TransformStatus::None` | `"none"` | No actions resulted in any modifications to the dataset. |

### JSON Execution Report Example

```json
{
  "status": "partial",
  "total_actions": 3,
  "actions_executed": 3,
  "actions_effective": 2,
  "tags_modified": 1,
  "tags_removed": 1,
  "duration_ms": 4
}
```

---

## 6. Edition Comparison & PRO Features

`dicom-rs-transformer` is available in two editions: **Community Edition** (Open Source / Free) and **Enterprise PRO Edition**.

| Feature | Community Edition | PRO Edition |
| :--- | :--- | :--- |
| **Top-Level Tag Operations** (`SET`, `DELETE`, `REPLACE`, `ANONYMIZE`) | ✅ Supported | ✅ Supported |
| **Local Filesystem I/O** (`file://` and local paths) | ✅ Supported | ✅ Supported |
| **JSON DSL & Script Compilation** | ✅ Supported | ✅ Supported |
| **Export Formats** (`SAVE_JSON`, `DUMP`, `EXTRACT_PIXELS`) | ✅ Supported | ✅ Supported |
| **Developer Extension Traits** (`CloudStorageHandler`, `SequencePathEvaluator`) | ✅ Stubs provided | ✅ Fully Implemented |
| **Cloud Storage I/O** (`s3://`, `gs://`, `az://`) | 🔒 **PRO Feature** | ✅ Supported |
| **Network & DICOM Web Protocols** (`dicom://`, `dicoms://`, `http://`, `https://`) | 🔒 **PRO Feature** | ✅ Supported |
| **Nested DICOM Sequence Path Evaluation** (`Seq[0]/Tag`, `Seq/Tag`, `Seq[*]/Tag`) | 🔒 **PRO Feature** | ✅ Supported |

> [!NOTE]
> Community Edition includes developer extension traits (`CloudStorageHandler` and `SequencePathEvaluator` in `dicom_rs_transformer::pro`) so developers can plug in custom implementations if needed.

### Enterprise Subscription & Licensing

For enterprise deployment, cloud storage integration, sequence hierarchy evaluation, and commercial SLA support, please contact **Gosmart.Health** for a professional subscription:

🌐 **Contact Form**: [https://www.gosmart.health/contact/](https://www.gosmart.health/contact/)  
📧 **Email**: support@gosmart.health

