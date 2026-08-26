use clap::{Parser, Subcommand, ValueEnum};
use dicom_object::{open_file, FileDicomObject, InMemDicomObject};
use dicom_rs_transformer::{
    Action, DicomTransformer, ScriptParser, TagSelector, TransformError, TransformSpec,
};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "dicom-transformer",
    author = "Manabu Tokunaga",
    version = "0.1.0",
    about = "DICOM Dataset Transformation CLI & MCP Interactive Console"
)]
struct Cli {
    /// Automatically register this CLI binary as an MCP server in local AI developer tools.
    #[arg(long = "install-mcp")]
    install_mcp: bool,

    /// Target AI tool configuration (all, antigravity, cursor, claude).
    #[arg(short = 't', long = "target", value_enum, default_value_t = McpTarget::All)]
    target: McpTarget,

    /// Run as a standard JSON-RPC 2.0 Model Context Protocol (MCP) server over stdio.
    #[arg(long = "mcp")]
    mcp: bool,

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

    /// Launch interactive console (REPL mode) for line-by-line execution.
    Console {
        /// Optional path to DICOM file to transform in-memory during console session.
        #[arg(short = 'i', long)]
        input: Option<PathBuf>,

        /// Optional path to save final transformed DICOM file on exit.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },

    /// Run as a standard JSON-RPC 2.0 Model Context Protocol (MCP) server over stdio.
    Mcp,

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

    /// Reassemble DICOM dataset(s) from JSON metadata headers and raw pixel data back into memory, with optional local save or PACS push.
    Assemble {
        /// Path to input directory containing JSON files (and optional companion raw files) or single JSON file.
        #[arg(short = 'i', long)]
        input: PathBuf,

        /// Optional path to directory or file containing raw pixel data.
        #[arg(short = 'r', long)]
        raw: Option<PathBuf>,

        /// Optional destination directory or file path to save assembled DICOM files.
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Optional PACS C-STORE target URI (e.g. dicom://pacs.hospital.org:104/AETITLE).
        #[arg(short = 'p', long)]
        pacs: Option<String>,
    },

    /// Output MCP tool discovery schema listing all available transformation commands and DSL actions.
    Schema,

    /// Download sample pydicom test DICOM files to a target directory.
    DownloadTestFiles {
        /// Destination directory path (defaults to target/dicom_test_files/pydicom).
        #[arg(short = 'd', long, default_value = "target/dicom_test_files/pydicom")]
        destination: PathBuf,
    },

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
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    if cli.install_mcp {
        install_mcp_config(cli.target)?;
        return Ok(());
    }

    if cli.mcp {
        run_mcp_server()?;
        return Ok(());
    }

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
        Some(Commands::Mcp) => {
            run_mcp_server()?;
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
        Some(Commands::Assemble {
            input,
            raw,
            output,
            pacs,
        }) => {
            if input.is_dir() {
                let res = dicom_rs_transformer::DicomAssembler::assemble_directory(&input, raw.as_deref())?;
                println!(
                    "Assembly complete: Reassembled {} datasets ({} with pixel data attached).",
                    res.total_assembled, res.with_pixel_data
                );

                if let Some(ref out_dir) = output {
                    if !out_dir.exists() {
                        std::fs::create_dir_all(out_dir)?;
                    }
                    for (idx, obj) in res.objects.iter().enumerate() {
                        let sop_uid = obj
                            .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
                            .ok()
                            .and_then(|e| e.to_str().ok())
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|| format!("assembled_{}", idx));
                        let file_path = out_dir.join(format!("{}.dcm", sop_uid));
                        obj.write_to_file(&file_path)?;
                    }
                    println!(
                        "Saved {} assembled DICOM files to: {}",
                        res.objects.len(),
                        out_dir.display()
                    );
                }

                if let Some(ref pacs_uri) = pacs {
                    use dicom_rs_transformer::pro::{DefaultPacsPushHandler, PacsPushHandler};
                    for obj in &res.objects {
                        DefaultPacsPushHandler.push_pacs(pacs_uri, obj)?;
                    }
                    println!("Pushed {} datasets to PACS: {}", res.objects.len(), pacs_uri);
                }
            } else {
                let obj = dicom_rs_transformer::DicomAssembler::assemble_file(&input, raw.as_deref())?;
                let has_pixels = obj.element(dicom_dictionary_std::tags::PIXEL_DATA).is_ok();
                println!(
                    "Assembly complete: Reassembled dataset from {} (pixel data: {}).",
                    input.display(),
                    if has_pixels { "attached" } else { "none" }
                );

                if let Some(ref out_path) = output {
                    let target_file = if out_path.is_dir() {
                        let sop_uid = obj
                            .element(dicom_dictionary_std::tags::SOP_INSTANCE_UID)
                            .ok()
                            .and_then(|e| e.to_str().ok())
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|| "assembled".to_string());
                        out_path.join(format!("{}.dcm", sop_uid))
                    } else {
                        out_path.clone()
                    };
                    obj.write_to_file(&target_file)?;
                    println!("Saved assembled DICOM to: {}", target_file.display());
                }

                if let Some(ref pacs_uri) = pacs {
                    use dicom_rs_transformer::pro::{DefaultPacsPushHandler, PacsPushHandler};
                    DefaultPacsPushHandler.push_pacs(pacs_uri, &obj)?;
                    println!("Pushed dataset to PACS: {}", pacs_uri);
                }
            }
        }
        Some(Commands::Schema) => {
            let schema_json = serde_json::json!({
                "mcp_version": "1.0",
                "tools": get_mcp_tools_list()
            });
            println!("{}", serde_json::to_string_pretty(&schema_json)?);
        }
        Some(Commands::DownloadTestFiles { destination }) => {
            download_pydicom_test_files(&destination)?;
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

fn get_mcp_tools_list() -> serde_json::Value {
    serde_json::json!([
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
            "name": "assemble",
            "description": "Reassemble DICOM dataset(s) from JSON metadata headers and raw pixel data back into memory, with optional local save or PACS push.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "input_location": { "type": "string", "description": "Input directory containing JSON files or single JSON file path" },
                    "raw_location": { "type": "string", "description": "Optional companion raw pixel data directory or file" },
                    "output_location": { "type": "string", "description": "Optional destination directory or file path to save assembled DICOM" },
                    "pacs_destination": { "type": "string", "description": "Optional PACS C-STORE destination URI (e.g. dicom://host:port/AETITLE)" }
                },
                "required": ["input_location"]
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
            "name": "dump_dataset",
            "description": "Dump text summary or tree of DICOM elements. Can inspect the currently loaded in-memory dataset or directly load and dump from an optional local file path or cloud URI.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "location": { "type": "string", "description": "Optional local file path (e.g. /tmp/file.dcm) or cloud URI (s3://, gs://, az://) to load and dump" }
                }
            }
        },
        {
            "name": "anonymize_patient",
            "description": "Anonymize patient name and patient ID tags in DICOM dataset.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "patient_name": { "type": "string", "description": "Replacement name (default: ANONYMOUS)" },
                    "patient_id": { "type": "string", "description": "Replacement ID (default: ANON-ID)" }
                }
            }
        },
        {
            "name": "download_test_files",
            "description": "Downloads sample pydicom test DICOM files to a target local directory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "destination": { "type": "string", "description": "Target destination directory (default: target/dicom_test_files/pydicom)" }
                }
            }
        },
        {
            "name": "execute",
            "description": "Explicitly execute buffered transformation pipeline for directory batch processing or script completion.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ])
}

fn apply_action_to_dataset(
    dataset: &mut Option<FileDicomObject<InMemDicomObject>>,
    action: Action,
) -> (String, bool) {
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
        Ok(report) => (
            format!(
                "Successfully executed action. Tags modified: {}, tags removed: {}.",
                report.tags_modified, report.tags_removed
            ),
            false,
        ),
        Err(e) => (format!("Error executing action: {}", e), true),
    }
}

fn handle_mcp_tool_call(
    name: &str,
    arguments: &serde_json::Value,
    dataset: &mut Option<FileDicomObject<InMemDicomObject>>,
) -> (String, bool) {
    match name {
        "load_dataset" => {
            let loc = match arguments.get("location").and_then(|v| v.as_str()) {
                Some(l) if !l.trim().is_empty() => l,
                _ => return ("Error: Missing required parameter 'location'".to_string(), true),
            };
            match dicom_rs_transformer::io::load_dicom_object(loc) {
                Ok(obj) => {
                    *dataset = Some(obj);
                    (format!("Successfully loaded DICOM dataset from: {}", loc), false)
                }
                Err(e) => (format!("Error loading DICOM dataset from '{}': {}", loc, e), true),
            }
        }
        "save_dataset" => {
            let loc = match arguments.get("location").and_then(|v| v.as_str()) {
                Some(l) if !l.trim().is_empty() => l,
                _ => return ("Error: Missing required parameter 'location'".to_string(), true),
            };
            if let Some(ref dcm) = dataset {
                match dicom_rs_transformer::io::save_dicom_object(loc, dcm) {
                    Ok(_) => (format!("Successfully saved DICOM dataset to: {}", loc), false),
                    Err(e) => (format!("Error saving DICOM dataset to '{}': {}", loc, e), true),
                }
            } else {
                ("Error: No active DICOM dataset in memory. Load one first or perform transformations.".to_string(), true)
            }
        }
        "assemble" => {
            let input_location = match arguments.get("input_location").and_then(|v| v.as_str()) {
                Some(i) if !i.trim().is_empty() => i,
                _ => return ("Error: Missing required parameter 'input_location'".to_string(), true),
            };
            let raw_location = arguments.get("raw_location").and_then(|v| v.as_str()).map(|s| s.to_string());
            let output_location = arguments.get("output_location").and_then(|v| v.as_str()).map(|s| s.to_string());
            let pacs_destination = arguments.get("pacs_destination").and_then(|v| v.as_str()).map(|s| s.to_string());
            let action = Action::Assemble {
                input_location: input_location.to_string(),
                raw_location,
                output_location,
                pacs_destination,
            };
            apply_action_to_dataset(dataset, action)
        }
        "set_tag" => {
            let selector_str = match arguments.get("selector").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s,
                _ => return ("Error: Missing required parameter 'selector'".to_string(), true),
            };
            let value = match arguments.get("value").and_then(|v| v.as_str()) {
                Some(v) => v,
                _ => return ("Error: Missing required parameter 'value'".to_string(), true),
            };
            let action = Action::SetTag {
                selector: TagSelector::Keyword(selector_str.to_string()),
                value: value.to_string(),
            };
            apply_action_to_dataset(dataset, action)
        }
        "remove_tag" => {
            let selector_str = match arguments.get("selector").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s,
                _ => return ("Error: Missing required parameter 'selector'".to_string(), true),
            };
            let action = Action::RemoveTag {
                selector: TagSelector::Keyword(selector_str.to_string()),
            };
            apply_action_to_dataset(dataset, action)
        }
        "replace_value" => {
            let selector_str = match arguments.get("selector").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s,
                _ => return ("Error: Missing required parameter 'selector'".to_string(), true),
            };
            let pattern = match arguments.get("pattern").and_then(|v| v.as_str()) {
                Some(p) => p,
                _ => return ("Error: Missing required parameter 'pattern'".to_string(), true),
            };
            let replacement = match arguments.get("replacement").and_then(|v| v.as_str()) {
                Some(r) => r,
                _ => return ("Error: Missing required parameter 'replacement'".to_string(), true),
            };
            let action = Action::ReplaceValue {
                selector: TagSelector::Keyword(selector_str.to_string()),
                pattern: pattern.to_string(),
                replacement: replacement.to_string(),
            };
            apply_action_to_dataset(dataset, action)
        }
        "generate_uid" => {
            let selector_str = match arguments.get("selector").and_then(|v| v.as_str()) {
                Some(s) if !s.trim().is_empty() => s,
                _ => return ("Error: Missing required parameter 'selector'".to_string(), true),
            };
            let source = arguments.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
            let action = Action::GenerateUid {
                selector: TagSelector::Keyword(selector_str.to_string()),
                source,
            };
            apply_action_to_dataset(dataset, action)
        }
        "extract_pixels" => {
            let destination = match arguments.get("destination").and_then(|v| v.as_str()) {
                Some(d) if !d.trim().is_empty() => d,
                _ => return ("Error: Missing required parameter 'destination'".to_string(), true),
            };
            let format = arguments.get("format").and_then(|v| v.as_str()).map(|s| s.to_string());
            let action = Action::ExtractPixels {
                destination: destination.to_string(),
                format: format.unwrap_or_else(|| "jpeg".to_string()),
            };
            apply_action_to_dataset(dataset, action)
        }
        "save_json" => {
            let json_location = match arguments.get("json_location").and_then(|v| v.as_str()) {
                Some(j) if !j.trim().is_empty() => j,
                _ => return ("Error: Missing required parameter 'json_location'".to_string(), true),
            };
            let raw_pixel_location = arguments.get("raw_pixel_location").and_then(|v| v.as_str()).map(|s| s.to_string());
            let action = Action::SaveJson {
                json_location: json_location.to_string(),
                raw_pixel_location,
            };
            apply_action_to_dataset(dataset, action)
        }
        "dump_dataset" => {
            if let Some(loc) = arguments.get("location").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()) {
                match dicom_rs_transformer::io::load_dicom_object(loc) {
                    Ok(obj) => {
                        *dataset = Some(obj);
                    }
                    Err(e) => return (format!("Error loading DICOM dataset from '{}': {}", loc, e), true),
                }
            }
            if let Some(ref dcm) = dataset {
                let mut dump_options = dicom_dump::DumpOptions::new();
                dump_options.no_limit(true);
                dump_options.color_mode(dicom_dump::ColorMode::Never);
                let mut output = Vec::new();
                if dump_options.dump_file_to(&mut output, dcm).is_ok() {
                    (String::from_utf8_lossy(&output).to_string(), false)
                } else {
                    ("Dataset loaded in memory.".to_string(), false)
                }
            } else {
                ("No DICOM dataset currently loaded in memory. Provide a 'location' parameter or call 'load_dataset' first.".to_string(), false)
            }
        }
        "anonymize_patient" => {
            let patient_name = arguments.get("patient_name").and_then(|v| v.as_str()).map(|s| s.to_string());
            let patient_id = arguments.get("patient_id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let action = Action::AnonymizePatient {
                patient_name,
                patient_id,
            };
            apply_action_to_dataset(dataset, action)
        }
        "download_test_files" => {
            let destination = arguments.get("destination").and_then(|v| v.as_str()).unwrap_or("target/dicom_test_files/pydicom");
            match download_pydicom_test_files(std::path::Path::new(destination)) {
                Ok(_) => (format!("Successfully downloaded test DICOM files to: {}", destination), false),
                Err(e) => (format!("Error downloading test files: {}", e), true),
            }
        }
        "execute" => {
            ("Transformation state synchronized.".to_string(), false)
        }
        unknown => {
            (format!("Unknown tool: {}", unknown), true)
        }
    }
}

fn run_mcp_server() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    let reader = BufReader::new(stdin.lock());

    let mut dataset: Option<FileDicomObject<InMemDicomObject>> = None;
    let tools_list = get_mcp_tools_list();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err_resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                writeln!(stdout_lock, "{}", serde_json::to_string(&err_resp)?)?;
                stdout_lock.flush()?;
                continue;
            }
        };

        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or_default();
        let id = request.get("id").cloned();

        // If it's a notification (no id), we do not send a response
        if id.is_none() || id.as_ref() == Some(&serde_json::Value::Null) {
            continue;
        }
        let id = id.unwrap();

        match method {
            "initialize" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "dicom-transformer",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    }
                });
                writeln!(stdout_lock, "{}", serde_json::to_string(&response)?)?;
                stdout_lock.flush()?;
            }
            "ping" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {}
                });
                writeln!(stdout_lock, "{}", serde_json::to_string(&response)?)?;
                stdout_lock.flush()?;
            }
            "tools/list" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools_list
                    }
                });
                writeln!(stdout_lock, "{}", serde_json::to_string(&response)?)?;
                stdout_lock.flush()?;
            }
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or_else(|| serde_json::json!({}));
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or_default();
                let arguments = params.get("arguments").cloned().unwrap_or_else(|| serde_json::json!({}));

                let (text, is_error) = handle_mcp_tool_call(name, &arguments, &mut dataset);

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": text
                            }
                        ],
                        "isError": is_error
                    }
                });
                writeln!(stdout_lock, "{}", serde_json::to_string(&response)?)?;
                stdout_lock.flush()?;
            }
            _ => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    }
                });
                writeln!(stdout_lock, "{}", serde_json::to_string(&response)?)?;
                stdout_lock.flush()?;
            }
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
        let config_paths = match t {
            McpTarget::Antigravity => vec![
                // Global Antigravity configuration (used by Antigravity CLI and App)
                PathBuf::from(&home).join(".gemini/config/mcp_config.json"),
                // Antigravity IDE configuration
                PathBuf::from(&home).join(".gemini/antigravity-ide/mcp/dicom-transformer.json"),
            ],
            McpTarget::Cursor => vec![
                PathBuf::from(&home).join("Library/Application Support/Cursor/User/globalStorage/mcp.json"),
            ],
            McpTarget::Claude => vec![
                PathBuf::from(&home).join("Library/Application Support/Claude/claude_desktop_config.json"),
            ],
            McpTarget::All => continue,
        };

        for config_path in config_paths {
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
                "args": ["mcp"]
            });

            if config_path.ends_with(".gemini/antigravity-ide/mcp/dicom-transformer.json") {
                let single_config = serde_json::json!({
                    "name": "dicom-transformer",
                    "command": exe_path,
                    "args": ["mcp"]
                });
                std::fs::write(&config_path, serde_json::to_string_pretty(&single_config)?)?;
            } else {
                if root.get("mcpServers").is_none() || !root["mcpServers"].is_object() {
                    root["mcpServers"] = serde_json::json!({});
                }
                root["mcpServers"]["dicom-transformer"] = mcp_entry;
                std::fs::write(&config_path, serde_json::to_string_pretty(&root)?)?;
            }

            println!(" [SUCCESS] Registered into: {}", config_path.display());
        }
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

    let mut directory_source: Option<String> = None;
    let mut dataset: Option<FileDicomObject<InMemDicomObject>> = if let Some(ip) = &input_path {
        if ip.is_dir() {
            println!("Targeting DICOM directory: {}", ip.display());
            directory_source = Some(ip.to_string_lossy().to_string());
            None
        } else {
            println!("Loading DICOM file: {}", ip.display());
            Some(dicom_rs_transformer::io::load_dicom_object(&ip.to_string_lossy())?)
        }
    } else {
        None
    };

    let parser = ScriptParser::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut line_count = 0;
    let mut buffered_actions: Vec<dicom_rs_transformer::Action> = Vec::new();

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

                // Handle LOAD: Check if single file or directory batch
                if let dicom_rs_transformer::Action::LoadDataset { ref location } = action {
                    let eval_loc = match dicom_rs_transformer::evaluate_macros(location) {
                        Ok(loc) => loc,
                        Err(e) => {
                            println!("     [EXEC ERROR] Failed to evaluate macro in location: {}", e);
                            continue;
                        }
                    };

                    let p = std::path::Path::new(&eval_loc);
                    if p.is_dir() {
                        match dicom_rs_transformer::scan_dicom_directory(&eval_loc) {
                            Ok(files) => {
                                println!(
                                    "     [BATCH LOAD] Found {} DICOM files in directory: {}",
                                    files.len(),
                                    eval_loc
                                );
                                directory_source = Some(eval_loc);
                                buffered_actions.clear();
                                buffered_actions.push(action.clone());
                            }
                            Err(e) => {
                                println!("     [EXEC ERROR] Failed to scan directory: {}", e);
                            }
                        }
                        continue;
                    } else {
                        directory_source = None;
                        buffered_actions.clear();
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
                }

                // If in directory batch mode, buffer actions until EXECUTE is called
                if let Some(ref dir_src) = directory_source {
                    if let dicom_rs_transformer::Action::Execute = action {
                        println!(
                            "     [BATCH EXECUTE] Starting batch processing for directory: {}",
                            dir_src
                        );
                        let files = match dicom_rs_transformer::scan_dicom_directory(dir_src) {
                            Ok(f) => f,
                            Err(e) => {
                                println!("     [EXEC ERROR] Failed to read directory files: {}", e);
                                continue;
                            }
                        };

                        let mut spec = TransformSpec::new();
                        for act in &buffered_actions {
                            if !matches!(act, dicom_rs_transformer::Action::LoadDataset { .. } | dicom_rs_transformer::Action::Execute) {
                                spec.add_action(act.clone());
                            }
                        }

                        let total = files.len();
                        let mut success_count = 0;
                        let transformer = DicomTransformer::new(spec);

                        for (idx, file_path) in files.iter().enumerate() {
                            match dicom_rs_transformer::io::load_dicom_object(&file_path.to_string_lossy()) {
                                Ok(file_obj) => {
                                    let mut ds = file_obj.into_inner();
                                    match transformer.transform_dataset(&mut ds) {
                                        Ok(_) => {
                                            success_count += 1;
                                            println!(
                                                "     [{}/{}] Processed: {}",
                                                idx + 1,
                                                total,
                                                file_path.file_name().unwrap_or_default().to_string_lossy()
                                            );
                                        }
                                        Err(e) => {
                                            println!(
                                                "     [{}/{}] Failed: {} ({})",
                                                idx + 1,
                                                total,
                                                file_path.file_name().unwrap_or_default().to_string_lossy(),
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!(
                                        "     [{}/{}] Read error: {} ({})",
                                        idx + 1,
                                        total,
                                        file_path.file_name().unwrap_or_default().to_string_lossy(),
                                        e
                                    );
                                }
                            }
                        }

                        println!(
                            "     [BATCH COMPLETE] Finished batch execution: {}/{} files succeeded",
                            success_count, total
                        );
                        buffered_actions.clear();
                        continue;
                    } else {
                        buffered_actions.push(action);
                        println!(
                            "     [BATCH BUFFER] Queued action #{} (type EXECUTE to run batch)",
                            buffered_actions.len() - 1
                        );
                        continue;
                    }
                }

                // Single File Mode Execution
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

fn download_pydicom_test_files(destination: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    const PYDICOM_FILES: &[&str] = &[
        "pydicom/693_J2KI.dcm",
        "pydicom/CT_small.dcm",
        "pydicom/ExplVR_LitEndNoMeta.dcm",
        "pydicom/JPEG-LL.dcm",
        "pydicom/JPEG-lossy.dcm",
        "pydicom/JPEG2000.dcm",
        "pydicom/JPEG2000_UNC.dcm",
        "pydicom/JPGLosslessP14SV1_1s_1f_8b.dcm",
        "pydicom/MR_small.dcm",
        "pydicom/SC_rgb.dcm",
        "pydicom/SC_rgb_16bit.dcm",
        "pydicom/SC_rgb_2frame.dcm",
        "pydicom/SC_rgb_jpeg_dcmtk.dcm",
        "pydicom/SC_rgb_jpeg_gdcm.dcm",
        "pydicom/SC_rgb_jpeg_lossy_gdcm.dcm",
        "pydicom/SC_rgb_rle.dcm",
        "pydicom/SC_rgb_rle_16bit.dcm",
        "pydicom/SC_rgb_rle_2frame.dcm",
        "pydicom/color-px.dcm",
        "pydicom/color3d_jpeg_baseline.dcm",
        "pydicom/image_dfl.dcm",
        "pydicom/liver.dcm",
    ];

    std::fs::create_dir_all(destination)?;
    println!("Downloading {} test DICOM files to {}...", PYDICOM_FILES.len(), destination.display());

    let mut count = 0;
    for file_name in PYDICOM_FILES {
        let test_file_path = dicom_test_files::path(file_name)
            .map_err(|e| format!("Failed to download/locate {}: {:?}", file_name, e))?;
        let dest_filename = file_name.replace('/', "_");
        let dest_path = destination.join(&dest_filename);
        std::fs::copy(&test_file_path, &dest_path)?;
        count += 1;
    }

    println!("Successfully downloaded {} files to {}", count, destination.display());
    Ok(())
}
