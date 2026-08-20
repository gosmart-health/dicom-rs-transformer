# Developer Initial Setup & Getting Started Guide

> [!WARNING]
> **IMPORTANT REMINDER:** This software can create and store files containing protected patient information (PHI). Do not commit data files that contain actual patient information to git or public repositories. Always check and secure the file permissions of directories where you develop, test, and deploy.

Welcome to the `dicom-rs-transformer` project! This guide will help you set up your development environment, understand the repository layout, and build and run the project—even if you are new to Rust.

---

## 1. Prerequisites

To build and run this project, you need the Rust programming language toolchain installed on your computer.

### Installing Rust

The recommended way to install Rust is using `rustup`, the official installer:

- **macOS / Linux**: Open your terminal and run:
  ```bash
  curl --proto '=https' --ltsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Windows**: Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs/).

Follow the default installation instructions. Once installed, restart your terminal.

### Verify Installation

Run the following commands to make sure Rust and its package manager (`cargo`) are installed:

```bash
rustc --version
cargo --version
```

- **`rustc`** is the Rust compiler.
- **`cargo`** is Rust's all-in-one package manager, build system, and test runner (similar to `npm` for Node.js, `pip` for Python, or `maven`/`gradle` for Java).

---

## 2. Project Layout

Here is a quick map of the repository to help you navigate the codebase:

```text
dicom-rs-transformer/
├── Cargo.toml                  # Project configuration & dependency list
├── README.md                   # Main project overview
├── sample_script.txt           # Sample transformation script
├── docs/
│   └── developing/
│       └── initial-setup.md    # This guide
├── src/                        # Core source code
│   ├── lib.rs                  # Main library entry point
│   ├── dsl.rs                  # JSON transformation rule models
│   ├── engine.rs               # Core DICOM modification engine
│   ├── error.rs                # Error types and handling
│   ├── script.rs               # Line-by-line script parser
│   └── main.rs                 # Command-line interface (CLI) application
├── examples/                   # Code examples showing how to use the library
│   └── transform_sample.rs
└── tests/                      # Automated test suite
    └── transformation_tests.rs
```

---

## 3. Building and Running the Code

### Step 1: Clone and Navigate to the Repository

```bash
git clone https://github.com/gosmart-health/dicom-rs-transformer.git
cd dicom-rs-transformer
```

### Step 2: Build the Project

Compile the library and command-line tool by running:

```bash
cargo build
```

*(Note: The very first build will download `dicom-rs` dependencies. Subsequent builds will be very fast.)*

### Step 3: Run the Automated Tests

Verify that everything is working properly on your system by running the test suite:

```bash
cargo test
```

You should see output indicating all unit tests, integration tests, and documentation tests have passed successfully.

---

## 4. Running the Project Features

### A. Run the Code Example

We provide a sample program in the `examples/` folder demonstrating how developers can import and use `dicom-rs-transformer` in their own Rust code:

```bash
cargo run --example transform_sample
```

### B. Run the Interactive Command-Line Console (CLI / MCP Mode)

Start the interactive console where you can type transformation commands line-by-line:

```bash
cargo run -- console
```

Inside the console, try typing:
```text
SET PatientName "ANONYMOUS^PATIENT"
DELETE PatientAddress
QUIT
```

### C. Validate a Script File

Validate the syntax of a text script file without modifying any files:

```bash
cargo run -- validate --script sample_script.txt
```

---

## 5. Daily Development Workflow

Here are a few handy commands you will use during everyday development:

- **Quick Code Check** (fast feedback without producing binary output):
  ```bash
  cargo check
  ```
- **Code Linter / Quality Check** (identifies improvements and common issues):
  ```bash
  cargo clippy
  ```
- **Generate & View Local API Documentation**:
  ```bash
  cargo doc --open
  ```

## Next Steps & Guides

- **[Development Workflow & Branching Strategy Guide](development-workflow.md)**: Reference for branching models, pull requests, and QA staging processes.
- **[Transformer DSL & Script Language Guide](../transformer-dsl-guide/transformer-dsl.md)**: Detailed reference for JSON DSL operations and line-by-line script syntax.

---

## Need Help?

If you encounter any issues during setup or development, please open an issue in the project repository or reach out to the project maintainers.
