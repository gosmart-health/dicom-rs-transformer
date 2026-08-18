//! CLI utility binary for `dicom-rs-transformer`.
//! Supports batch DICOM dataset transformation and line-by-line interactive console mode (MCP friendly).

use clap::{Parser, Subcommand, ValueEnum};
use dicom_object::{open_file, FileDicomObject, InMemDicomObject};
use dicom_rs_transformer::{DicomTransformer, ScriptParser, TransformError, TransformSpec};
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "dicom-transformer",
    author = "Manabu Tokunaga",
    version = "0.1.0",
    about = "DICOM Dataset Transformation CLI & MCP Interactive Console"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a transformation on an input DICOM file using a script or JSON DSL specification.
    Run {
        /// Path to input DICOM file.
        #[arg(short = 'i', long)]
        input: PathBuf,

        /// Path to output transformed DICOM file.
        #[arg(short = 'o', long)]
        output: PathBuf,

        /// Path to text script file containing line-by-line commands.
        #[arg(short = 's', long)]
        script: Option<PathBuf>,

        /// Path to JSON-encoded DSL specification file.
        #[arg(short = 'd', long)]
        dsl: Option<PathBuf>,
    },

    /// Launch interactive console (REPL mode) for line-by-line execution (MCP tool compatible).
    Console {
        /// Optional path to DICOM file to transform in-memory during console session.
        #[arg(short = 'i', long)]
        input: Option<PathBuf>,

        /// Optional path to save final transformed DICOM file on exit.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Validate the syntax of a line-by-line script or JSON DSL specification file.
    Validate {
        /// Path to text script file to validate.
        #[arg(short = 's', long)]
        script: Option<PathBuf>,

        /// Path to JSON DSL file to validate.
        #[arg(short = 'd', long)]
        dsl: Option<PathBuf>,
    },

    /// Compile a line script or JSON spec into a reusable JSON DSL or text script file without processing DICOM data.
    Compile {
        /// Path to text script file to compile.
        #[arg(short = 's', long)]
        script: Option<PathBuf>,

        /// Path to JSON DSL specification file to compile.
        #[arg(short = 'd', long)]
        dsl: Option<PathBuf>,

        /// Destination output file path (.json or .txt).
        #[arg(short = 'o', long)]
        output: PathBuf,
    },

    /// Output MCP tool discovery schema listing all available transformation commands and DSL actions.
    Schema,

    /// Automatically register this CLI binary as an MCP server in local AI developer tools.
    InstallMcp {
        /// Target AI tool configuration (all, antigravity, cursor, claude).
        #[arg(short = 't', long, value_enum, default_value_t = McpTarget::All)]
        target: McpTarget,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum McpTarget {
    All,
    Antigravity,
    Cursor,
    Claude,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run {
            input,
            output,
            script,
            dsl,
        }) => {
            let spec = load_spec(script, dsl)?;
            println!("Loaded specification with {} actions.", spec.actions.len());

            let mut dicom_obj = open_file(&input)?;
            let transformer = DicomTransformer::new(spec);

            let report = transformer.transform_file(&mut dicom_obj)?;
            println!(
                "Transformation complete: Status: {:?}, {}/{} effective actions ({} tags modified, {} tags removed) in {}ms.",
                report.status, report.actions_effective, report.total_actions, report.tags_modified, report.tags_removed, report.duration_ms
            );

            dicom_obj.write_to_file(&output)?;
            println!("Transformed DICOM saved to: {}", output.display());
        }
        Some(Commands::Console { input, output }) => {
            run_console_session(input, output)?;
        }
        Some(Commands::Validate { script, dsl }) => {
            let spec = load_spec(script, dsl)?;
            println!(
                "Validation SUCCESS: Specification is valid with {} actions defined.",
                spec.actions.len()
            );
        }
        Some(Commands::Compile {
            script,
            dsl,
            output,
        }) => {
            let spec = load_spec(script, dsl)?;
            let is_script_output = output
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.eq_ignore_ascii_case("txt") || s.eq_ignore_ascii_case("script"))
                .unwrap_or(false);

            let content = if is_script_output {
                spec.to_script()
            } else {
                spec.to_json()?
            };

            std::fs::write(&output, content)?;
            println!(
                "Compile SUCCESS: Written compiled specification ({} actions) to {}",
                spec.actions.len(),
                output.display()
            );
        }
        Some(Commands::Schema) => {
            let schema_json = serde_json::json!({
                "mcp_version": "1.0",
                "tools": [
                    {
                        "name": "load_dataset",
                        "description": "Loads a DICOM dataset from a local Unix/Windows file path or cloud URI (s3://, gs://, az://).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "location": { "type": "string", "description": "Local file path or s3://, gs://, az:// cloud URI" }
                            },
                            "required": ["location"]
                        }
                    },
                    {
                        "name": "save_dataset",
                        "description": "Saves the current DICOM dataset to a local file path or cloud URI (s3://, gs://, az://).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "location": { "type": "string", "description": "Local file path or s3://, gs://, az:// cloud URI" }
                            },
                            "required": ["location"]
                        }
                    },
                    {
                        "name": "set_tag",
                        "description": "Set or update the value of a specific DICOM tag by keyword or hex pair.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "selector": { "type": "string", "description": "DICOM keyword (e.g. PatientName) or hex pair (0010,0010)" },
                                "value": { "type": "string", "description": "Value to set" }
                            },
                            "required": ["selector", "value"]
                        }
                    },
                    {
                        "name": "remove_tag",
                        "description": "Remove a DICOM tag from the dataset.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "selector": { "type": "string", "description": "DICOM keyword or hex pair" }
                            },
                            "required": ["selector"]
                        }
                    },
                    {
                        "name": "replace_value",
                        "description": "Perform substring replacement on a tag's value.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "selector": { "type": "string", "description": "DICOM keyword or hex pair" },
                                "pattern": { "type": "string", "description": "Substring pattern to match" },
                                "replacement": { "type": "string", "description": "Replacement string" }
                            },
                            "required": ["selector", "pattern", "replacement"]
                        }
                    },
                    {
                        "name": "generate_uid",
                        "description": "Generate a standard DICOM PS3.5 Annex B.2 (2.25.<u128>) UID. Generates a random UID (v4) or deterministic UID (v5) if source seed tag is provided.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "selector": { "type": "string", "description": "Target DICOM keyword (e.g. StudyInstanceUID, SeriesInstanceUID) or hex pair" },
                                "source": { "type": "string", "description": "Optional seed tag or string value for deterministic UID derivation" }
                            },
                            "required": ["selector"]
                        }
                    },
                    {
                        "name": "extract_pixels",
                        "description": "Extract DICOM dataset image frames to JPEG, PNG, or RAW files.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "destination": { "type": "string", "description": "Destination directory path or cloud folder URI" },
                                "format": { "type": "string", "enum": ["jpeg", "png", "raw"], "description": "Export format (default: jpeg)" }
                            },
                            "required": ["destination"]
                        }
                    },
                    {
                        "name": "save_json",
                        "description": "Export dataset to DICOM JSON format, optionally extracting raw pixel data.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "json_location": { "type": "string", "description": "Destination path/URI for DICOM JSON file" },
                                "raw_pixel_location": { "type": "string", "description": "Optional destination path/URI for raw pixel data" }
                            },
                            "required": ["json_location"]
                        }
                    },
                    {
                        "name": "anonymize_patient",
                        "description": "Anonymize patient identification fields (PatientName, PatientID).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "patient_name": { "type": "string", "description": "Replacement name (default: ANONYMOUS)" },
                                "patient_id": { "type": "string", "description": "Replacement ID (default: ANON-ID)" }
                            }
                        }
                    }
                ]
            });
            println!("{}", serde_json::to_string_pretty(&schema_json)?);
        }
        Some(Commands::InstallMcp { target }) => {
            install_mcp_config(target)?;
        }
        None => {
            // Default to console session if no subcommand provided
            run_console_session(None, None)?;
        }
    }

    Ok(())
}

fn install_mcp_config(target: McpTarget) -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let exe_path = current_exe.to_string_lossy().to_string();
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))?;

    let targets = match target {
        McpTarget::All => vec![McpTarget::Antigravity, McpTarget::Cursor, McpTarget::Claude],
        other => vec![other],
    };

    println!(
        "Installing dicom-transformer MCP configuration for binary: {}",
        exe_path
    );

    for t in targets {
        let config_path = match t {
            McpTarget::Antigravity => {
                PathBuf::from(&home).join(".gemini/antigravity-ide/mcp/dicom-transformer.json")
            }
            McpTarget::Cursor => PathBuf::from(&home)
                .join("Library/Application Support/Cursor/User/globalStorage/mcp.json"),
            McpTarget::Claude => PathBuf::from(&home)
                .join("Library/Application Support/Claude/claude_desktop_config.json"),
            McpTarget::All => continue,
        };

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut root: serde_json::Value = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if !root.is_object() {
            root = serde_json::json!({});
        }

        let mcp_entry = serde_json::json!({
            "command": exe_path,
            "args": ["schema"]
        });

        if t == McpTarget::Antigravity
            && config_path.extension().and_then(|s| s.to_str()) == Some("json")
            && config_path.file_name().and_then(|s| s.to_str()) == Some("dicom-transformer.json")
        {
            let single_config = serde_json::json!({
                "name": "dicom-transformer",
                "command": exe_path,
                "args": ["schema"]
            });
            std::fs::write(&config_path, serde_json::to_string_pretty(&single_config)?)?;
        } else {
            if root.get("mcpServers").is_none() {
                root["mcpServers"] = serde_json::json!({});
            }
            root["mcpServers"]["dicom-transformer"] = mcp_entry;
            std::fs::write(&config_path, serde_json::to_string_pretty(&root)?)?;
        }

        println!(" [SUCCESS] Registered into: {}", config_path.display());
    }

    Ok(())
}

fn load_spec(
    script_path: Option<PathBuf>,
    dsl_path: Option<PathBuf>,
) -> Result<TransformSpec, TransformError> {
    if let Some(sp) = script_path {
        let file = File::open(&sp)?;
        let reader = BufReader::new(file);
        let parser = ScriptParser::new();
        parser.parse_script(reader)
    } else if let Some(dp) = dsl_path {
        let content = std::fs::read_to_string(dp)?;
        TransformSpec::from_json(&content)
    } else {
        Err(TransformError::InvalidOperation(
            "Must provide either --script (-s) or --dsl (-d)".to_string(),
        ))
    }
}

fn run_console_session(
    input_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("dicom-transformer Interactive Console (MCP Compatible)");
    println!("Type commands line-by-line (e.g. SET PatientName \"ANONYMOUS\", DELETE PatientAddress, QUIT).");
    println!(
        "-----------------------------------------------------------------------------------------"
    );

    let mut dataset: Option<FileDicomObject<InMemDicomObject>> = if let Some(ip) = &input_path {
        println!("Loading DICOM file: {}", ip.display());
        Some(open_file(ip)?)
    } else {
        None
    };

    let parser = ScriptParser::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line_count = 0;

    loop {
        print!("> ");
        stdout.flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("QUIT") || trimmed.eq_ignore_ascii_case("EXIT") {
            println!("Exiting console.");
            break;
        }

        line_count += 1;
        match parser.parse_line(line_count, trimmed) {
            Ok(Some(action)) => {
                println!("[OK] Parsed action: {:?}", action);

                // Handle LOAD directly in console to initialize/reload dataset session
                if let dicom_rs_transformer::Action::LoadDataset { ref location } = action {
                    let eval_loc = match dicom_rs_transformer::evaluate_macros(location) {
                        Ok(loc) => loc,
                        Err(e) => {
                            println!("     [EXEC ERROR] Failed to evaluate macro in location: {}", e);
                            continue;
                        }
                    };
                    match dicom_rs_transformer::io::load_dicom_object(&eval_loc) {
                        Ok(loaded) => {
                            println!("     [EXEC] Successfully loaded DICOM dataset from: {}", eval_loc);
                            dataset = Some(loaded);
                        }
                        Err(e) => {
                            println!("     [EXEC ERROR] Failed to load DICOM object: {}", e);
                        }
                    }
                    continue;
                }

                // If dataset is not loaded yet, initialize an empty dataset so actions (e.g., SET, DUMP) can execute
                let dcm = dataset.get_or_insert_with(|| {
                    let media_sop_instance_uid = format!("2.25.{}", uuid::Uuid::new_v4().as_u128());
                    let meta = dicom_object::FileMetaTableBuilder::new()
                        .media_storage_sop_instance_uid(media_sop_instance_uid)
                        .transfer_syntax(
                            dicom_transfer_syntax_registry::entries::EXPLICIT_VR_LITTLE_ENDIAN.uid(),
                        )
                        .build()
                        .unwrap_or_else(|_| dicom_object::FileMetaTableBuilder::new().build().unwrap());
                    FileDicomObject::new_empty_with_dict_and_meta(
                        dicom_dictionary_std::StandardDataDictionary,
                        meta,
                    )
                });

                let mut spec = TransformSpec::new();
                spec.add_action(action);
                let transformer = DicomTransformer::new(spec);
                match transformer.transform_file(dcm) {
                    Ok(report) => {
                        println!(
                            "     [EXEC] Applied to dataset: {} modified, {} removed",
                            report.tags_modified, report.tags_removed
                        );
                    }
                    Err(e) => {
                        println!("     [EXEC ERROR] Failed to apply action: {}", e);
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                println!("[PARSE ERROR] {}", e);
            }
        }
    }

    if let (Some(dcm), Some(op)) = (dataset, output_path) {
        println!("Saving modified DICOM dataset to: {}", op.display());
        dcm.write_to_file(&op)?;
    }

    Ok(())
}
