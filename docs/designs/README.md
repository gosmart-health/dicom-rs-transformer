# Design Controls & Regulatory Documentation

This directory contains the FDA 510(k) and IEC 62304 / ISO 14971 aligned Design Controls documentation suite for `dicom-rs-transformer`.

The documents are prefixed with `gsdtc_XXX` in recommended reading order (from requirements to design, risk, V&V, traceability, and cybersecurity).

---

> [!IMPORTANT]
> **Regulatory Intent & Quality Management Scope**
> 
> While this project closely follows best quality control and design control management practices (aligned with IEC 62304, ISO 14971, and FDA 21 CFR 820.30), the existence and structure of these documents do **not** claim or certify that this open-source software has been developed under an audited or certified Quality Management System (QMS).
> 
> These artifacts are provided to **minimize regulatory friction and accelerate technical documentation** for downstream medical device manufacturers, system integrators, and healthcare organizations who are adopting, extending, and validating this software for actual clinical use within their own accredited QMS.

---

## Document Walkthrough Index

| Document | Regulatory Standard | Description |
| :--- | :--- | :--- |
| **[gsdtc_000_software_requirements_spec.md](./gsdtc_000_software_requirements_spec.md)** | IEC 62304 Cl. 5.2 | **Software Requirements Specification (SRS)**: Functional specifications, DICOM VR structures, DSL engine operations, dynamic macros, RPN logic, and NEMA PS3.15 de-identification compliance. |
| **[gsdtc_010_system_design_specification.md](./gsdtc_010_system_design_specification.md)** | IEC 62304 Cl. 5.3 / 5.4 | **System Design Specification (SDS / SAD)**: Subsystem decomposition (`main/cli`, `script/dsl`, `engine`, `map/audit`), sequence path traversal contracts, and memory safety model. |
| **[gsdtc_020_hazard_analysis_risk_management.md](./gsdtc_020_hazard_analysis_risk_management.md)** | ISO 14971:2019 / IEC 62304 Cl. 7 | **Hazard Analysis & Risk Management**: Software Risk Matrix identifying clinical hazards (PHI leakage, sequence path out-of-bounds, VR tag corruption, MCP stdio bounds) and software design risk controls. |
| **[gsdtc_030_verification_and_validation_plan.md](./gsdtc_030_verification_and_validation_plan.md)** | IEC 62304 Cl. 5.5 - 5.7 | **Verification & Validation Plan**: Test protocols across unit tests, sequence path integration tests, CLI script validation, and automated compliance security scans. |
| **[gsdtc_040_traceability_matrix.md](./gsdtc_040_traceability_matrix.md)** | FDA Design Controls | **Requirements Traceability Matrix (RTM)**: Bi-directional matrix mapping **Requirements (SRS) <-> System Design (SDS) <-> Hazards (ISO 14971) <-> Verification Tests (V&V)**. |
| **[gsdtc_050_cybersecurity_and_soup_bom.md](./gsdtc_050_cybersecurity_and_soup_bom.md)** | FDA Cybersecurity Guidance | **Cybersecurity & SOUP BOM**: Software Bill of Materials (SBOM) for SOUP components (`dicom-rs`, `serde`, `clap`, `tokio`, `uuid`, etc.), threat modeling, zero-disk staging, and PHI privacy rules. |

