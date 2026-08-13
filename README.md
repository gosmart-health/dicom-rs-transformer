# dicom-rs-transformer

Programmatically transform DICOM objects for anonymization or value changes with an MCP-enabled CLI console included.

## Features

- **Library & API**: Clean, well-documented Rust library (`dicom-rs-transformer`) designed for developers building DICOM data transformation pipelines.
- **JSON DSL Rules**: Define reusable DICOM transformation specifications in structured JSON format.
- **Line-by-Line Script Language**: Write simple, human-readable text scripts (`SET`, `DELETE`/`REMOVE`, `REPLACE`, `ANONYMIZE`) for batch processing or interactive tool use.
- **MCP Terminal Friendly**: Includes an interactive REPL console compatible with Model Context Protocol (MCP) tool pipelines.
- **Powered by `dicom-rs`**: Built on top of the latest [`dicom-rs`](https://github.com/Enet4/dicom-rs) (`v0.10.0`) ecosystem.

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

## Documentation

- [Developer Initial Setup Guide](docs/developing/initial-setup.md)
- [Development Workflow & Branching Strategy](docs/developing/development-workflow.md)
- [Transformer DSL & Script Language Guide](docs/transformer-dsl-guide/transformer-dsl.md)
- Generated API documentation: Run `cargo doc --open` locally.

## License

This project is licensed under the Apache License, Version 2.0 - see the [LICENSE](file:///Users/manabutokunaga/development/dicom-rs-transformer/LICENSE) file for details.

> **NOTICE:** This software is for educational, research, or informational purposes only.
> It is NOT certified as a medical device and is NOT intended to diagnose, treat,
> cure, or prevent any disease or medical condition.

