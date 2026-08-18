# Project Rules & Agent Instructions

## DICOM Transformer DSL Maintenance Rules

Whenever modifying, adding, or extending DICOM transformation DSL actions, commands, or execution logic in this repository:

1. **Update MCP JSON Discovery Schema**:
   * Synchronize the JSON Schema output in [`src/main.rs`](file:///Users/manabutokunaga/development/dicom-rs-transformer/src/main.rs) under the `Commands::Schema` match arm to include the new tool/action definition, parameter descriptions, required fields, and types.

2. **Update Language Documentation Guide**:
   * Update the documentation in [`docs/transformer-dsl-guide/transformer-dsl.md`](file:///Users/manabutokunaga/development/dicom-rs-transformer/docs/transformer-dsl-guide/transformer-dsl.md) with both the JSON DSL format example and the Line-by-Line script equivalent for any new or modified actions.
