use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct SchemaBlock {
    content: String,
    source_file: String,
    line_number: usize,
}

fn main() {
    println!("cargo:rerun-if-changed=TOOLS.md");
    println!("cargo:rerun-if-changed=PROMPTS.md");
    println!("cargo:rerun-if-changed=EVALS.md");
    println!("cargo:rerun-if-changed=AGENTS.md");
    println!("cargo:rerun-if-changed=MCP_SERVERS.md");
    println!("cargo:rerun-if-changed=CLAUDE.md");
    println!("cargo:rerun-if-changed=docs/schemas/tool-schema.json");
    println!("cargo:rerun-if-changed=docs/schemas/prompt-schema.json");
    println!("cargo:rerun-if-changed=docs/schemas/eval-schema.json");
    println!("cargo:rerun-if-changed=docs/schemas/agent-config-schema.json");
    println!("cargo:rerun-if-changed=docs/schemas/mcp-server-schema.json");
    println!("cargo:rerun-if-changed=docs/schemas/claude-config-schema.json");

    let tools = extract_and_validate("TOOLS.md", "tool", "docs/schemas/tool-schema.json")
        .expect("failed to process TOOLS.md");
    let prompts = extract_and_validate("PROMPTS.md", "prompt", "docs/schemas/prompt-schema.json")
        .expect("failed to process PROMPTS.md");
    let evals = extract_and_validate("EVALS.md", "eval", "docs/schemas/eval-schema.json")
        .expect("failed to process EVALS.md");
    let agents = extract_and_validate(
        "AGENTS.md",
        "agent-config",
        "docs/schemas/agent-config-schema.json",
    )
    .expect("failed to process AGENTS.md");
    let mcp_servers = extract_and_validate(
        "MCP_SERVERS.md",
        "mcp-server",
        "docs/schemas/mcp-server-schema.json",
    )
    .expect("failed to process MCP_SERVERS.md");
    let claude_configs = extract_and_validate(
        "CLAUDE.md",
        "claude-config",
        "docs/schemas/claude-config-schema.json",
    )
    .expect("failed to process CLAUDE.md");

    validate_cross_references(&prompts, &evals, &claude_configs)
        .expect("cross-reference validation failed");

    generate_modules(
        Path::new("src/generated"),
        &tools,
        &prompts,
        &evals,
        &agents,
        &mcp_servers,
        &claude_configs,
    )
    .expect("failed to generate schema code");

    println!(
        "cargo:warning=schema generation complete (tools={}, prompts={}, evals={}, agents={}, mcp={}, claude={})",
        tools.len(),
        prompts.len(),
        evals.len(),
        agents.len(),
        mcp_servers.len(),
        claude_configs.len()
    );
}

fn extract_and_validate(
    markdown_path: &str,
    schema_type: &str,
    meta_schema_path: &str,
) -> Result<Vec<SchemaBlock>, String> {
    if !Path::new(markdown_path).exists() {
        return Err(format!("file not found: {markdown_path}"));
    }

    let markdown = fs::read_to_string(markdown_path)
        .map_err(|err| format!("failed to read {markdown_path}: {err}"))?;
    let schemas = extract_schemas(&markdown, schema_type, markdown_path);

    let meta_schema_raw = fs::read_to_string(meta_schema_path)
        .map_err(|err| format!("failed to read {meta_schema_path}: {err}"))?;
    let meta_schema: Value = serde_json::from_str(&meta_schema_raw)
        .map_err(|err| format!("failed to parse {meta_schema_path}: {err}"))?;
    let validator = jsonschema::JSONSchema::compile(&meta_schema)
        .map_err(|err| format!("failed to compile meta-schema {meta_schema_path}: {err}"))?;

    for schema in &schemas {
        validate_schema_block(schema, &validator)?;
    }
    check_unique_ids(&schemas)?;

    Ok(schemas)
}

fn extract_schemas(markdown: &str, schema_type: &str, source_file: &str) -> Vec<SchemaBlock> {
    let parser = Parser::new(markdown).into_offset_iter();
    let mut schemas = Vec::new();
    let mut code_lang: Option<String> = None;
    let mut code_start_offset = 0usize;
    let mut code_text = String::new();

    for (event, range) in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                code_text.clear();
                code_start_offset = range.start;
                code_lang = Some(match kind {
                    CodeBlockKind::Fenced(lang) => lang.to_string(),
                    CodeBlockKind::Indented => String::new(),
                });
            }
            Event::Text(text) => {
                if code_lang.is_some() {
                    code_text.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(lang) = &code_lang {
                    let expected = format!("yaml schema {schema_type}");
                    if lang.starts_with(&expected) {
                        let line_number = byte_offset_to_line(markdown, code_start_offset);
                        schemas.push(SchemaBlock {
                            content: code_text.clone(),
                            source_file: source_file.to_string(),
                            line_number,
                        });
                    }
                }
                code_lang = None;
            }
            _ => {}
        }
    }

    schemas
}

fn byte_offset_to_line(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count()
        + 1
}

fn validate_schema_block(
    schema: &SchemaBlock,
    compiled_validator: &jsonschema::JSONSchema,
) -> Result<(), String> {
    let schema_value: Value = serde_yaml::from_str(&schema.content).map_err(|err| {
        format!(
            "{}:{}: yaml parse error: {err}",
            schema.source_file, schema.line_number
        )
    })?;

    if let Err(errors) = compiled_validator.validate(&schema_value) {
        let mut messages = Vec::new();
        for error in errors {
            messages.push(format!("- {error}"));
        }
        return Err(format!(
            "{}:{}: schema validation failed\n{}",
            schema.source_file,
            schema.line_number,
            messages.join("\n")
        ));
    }

    if let Some(version) = schema_value.get("version").and_then(Value::as_str) {
        semver::Version::parse(version).map_err(|err| {
            format!(
                "{}:{}: invalid semver '{version}': {err}",
                schema.source_file, schema.line_number
            )
        })?;
    }

    Ok(())
}

fn check_unique_ids(schemas: &[SchemaBlock]) -> Result<(), String> {
    let mut seen = HashMap::<(String, String), (String, usize)>::new();

    for schema in schemas {
        let schema_value: Value = serde_yaml::from_str(&schema.content).map_err(|err| {
            format!(
                "{}:{}: failed to parse schema for uniqueness: {err}",
                schema.source_file, schema.line_number
            )
        })?;

        let id = schema_value
            .get("name")
            .or_else(|| schema_value.get("id"))
            .or_else(|| schema_value.get("suite_id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "{}:{}: schema missing identifier field (name/id/suite_id)",
                    schema.source_file, schema.line_number
                )
            })?;

        let version = schema_value
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "{}:{}: schema missing version field",
                    schema.source_file, schema.line_number
                )
            })?;

        let key = (id.to_string(), version.to_string());
        if let Some((prev_file, prev_line)) = seen.get(&key) {
            return Err(format!(
                "{}:{}: duplicate id+version pair '{id}@{version}' (already defined at {prev_file}:{prev_line})",
                schema.source_file, schema.line_number
            ));
        }

        seen.insert(key, (schema.source_file.clone(), schema.line_number));
    }

    Ok(())
}

fn validate_cross_references(
    prompts: &[SchemaBlock],
    evals: &[SchemaBlock],
    claude_configs: &[SchemaBlock],
) -> Result<(), String> {
    let prompt_ids: HashSet<String> = prompts
        .iter()
        .filter_map(|schema| extract_id(&schema.content, "id"))
        .collect();

    let valid_models = extract_known_models(claude_configs)?;
    for prompt in prompts {
        let prompt_value: Value = serde_yaml::from_str(&prompt.content).map_err(|err| {
            format!(
                "{}:{}: failed to parse prompt during cross-reference checks: {err}",
                prompt.source_file, prompt.line_number
            )
        })?;

        if let Some(models) = prompt_value.get("allowed_models").and_then(Value::as_array) {
            for model in models {
                if let Some(model_str) = model.as_str() {
                    if !valid_models.is_empty() && !valid_models.contains(model_str) {
                        return Err(format!(
                            "{}:{}: unknown model '{model_str}' referenced in allowed_models",
                            prompt.source_file, prompt.line_number
                        ));
                    }
                }
            }
        }
    }

    for eval in evals {
        let eval_value: Value = serde_yaml::from_str(&eval.content).map_err(|err| {
            format!(
                "{}:{}: failed to parse eval during cross-reference checks: {err}",
                eval.source_file, eval.line_number
            )
        })?;

        if let Some(cases) = eval_value.get("cases").and_then(Value::as_array) {
            for case in cases {
                if let Some(prompt_id) = case
                    .get("input")
                    .and_then(|input| input.get("prompt_id"))
                    .and_then(Value::as_str)
                {
                    if !prompt_ids.contains(prompt_id) {
                        return Err(format!(
                            "{}:{}: eval references unknown prompt_id '{prompt_id}'",
                            eval.source_file, eval.line_number
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_known_models(claude_configs: &[SchemaBlock]) -> Result<HashSet<String>, String> {
    let mut models = HashSet::new();
    for config in claude_configs {
        let value: Value = serde_yaml::from_str(&config.content).map_err(|err| {
            format!(
                "{}:{}: failed to parse claude config: {err}",
                config.source_file, config.line_number
            )
        })?;

        if let Some(entries) = value.get("models").and_then(Value::as_array) {
            for entry in entries {
                if let Some(model_id) = entry.get("model_id").and_then(Value::as_str) {
                    models.insert(model_id.to_string());
                }
            }
        }
    }

    Ok(models)
}

fn extract_id(content: &str, field_name: &str) -> Option<String> {
    serde_yaml::from_str::<Value>(content)
        .ok()?
        .get(field_name)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn generate_modules(
    output_dir: &Path,
    tools: &[SchemaBlock],
    prompts: &[SchemaBlock],
    evals: &[SchemaBlock],
    agents: &[SchemaBlock],
    mcp_servers: &[SchemaBlock],
    claude_configs: &[SchemaBlock],
) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;

    let tools_json = to_json_array(tools)?;
    let prompts_json = to_json_array(prompts)?;
    let evals_json = to_json_array(evals)?;
    let agents_json = to_json_array(agents)?;
    let mcp_json = to_json_array(mcp_servers)?;
    let claude_json = to_json_array(claude_configs)?;

    write_if_changed(
        &output_dir.join("tools.rs"),
        &render_schema_module("tool", "name", &tools_json, true),
    )?;
    write_if_changed(
        &output_dir.join("prompts.rs"),
        &render_schema_module("prompt", "id", &prompts_json, false),
    )?;
    write_if_changed(
        &output_dir.join("evals.rs"),
        &render_schema_module("eval", "suite_id", &evals_json, false),
    )?;
    write_if_changed(
        &output_dir.join("agents.rs"),
        &render_schema_module("agent", "name", &agents_json, false),
    )?;
    write_if_changed(
        &output_dir.join("mcp_servers.rs"),
        &render_schema_module("mcp_server", "name", &mcp_json, false),
    )?;
    write_if_changed(
        &output_dir.join("claude.rs"),
        &render_schema_module("claude", "name", &claude_json, false),
    )?;
    write_if_changed(&output_dir.join("mod.rs"), &render_mod_file())?;

    Ok(())
}

fn to_json_array(schemas: &[SchemaBlock]) -> Result<String, String> {
    let mut values = Vec::new();
    for schema in schemas {
        let value: Value = serde_yaml::from_str(&schema.content).map_err(|err| {
            format!(
                "{}:{}: failed to convert schema yaml to json: {err}",
                schema.source_file, schema.line_number
            )
        })?;
        values.push(value);
    }
    serde_json::to_string_pretty(&values)
        .map_err(|err| format!("failed to serialize generated schema json: {err}"))
}

fn render_schema_module(
    kind: &str,
    id_field: &str,
    schemas_json: &str,
    include_tool_validation: bool,
) -> String {
    let json_literal = format!("{schemas_json:?}");
    let tool_validation = if include_tool_validation {
        r#"
pub fn validate_tool_input(name: &str, version: &str, input: &Value) -> anyhow::Result<()> {
    let schema = find_by_name_and_version(name, version)
        .ok_or_else(|| anyhow::anyhow!("unknown tool schema {name}@{version}"))?;

    let input_schema = schema
        .get("input_schema")
        .ok_or_else(|| anyhow::anyhow!("tool schema {name}@{version} missing input_schema"))?;

    let compiled = jsonschema::JSONSchema::compile(input_schema)
        .map_err(|err| anyhow::anyhow!("failed to compile input schema for {name}@{version}: {err}"))?;

    if let Err(errors) = compiled.validate(input) {
        let details: Vec<String> = errors.map(|err| err.to_string()).collect();
        return Err(anyhow::anyhow!(
            "tool input validation failed for {name}@{version}: {}",
            details.join("; ")
        ));
    }

    Ok(())
}
"#
    } else {
        ""
    };

    format!(
        "// @generated by build.rs. Do not edit manually.\n\
use serde_json::Value;\n\
use std::sync::OnceLock;\n\
\n\
pub const SCHEMAS_JSON: &str = {json_literal};\n\
\n\
static SCHEMAS: OnceLock<Vec<Value>> = OnceLock::new();\n\
\n\
pub fn all() -> &'static [Value] {{\n\
    SCHEMAS\n\
        .get_or_init(|| serde_json::from_str(SCHEMAS_JSON).expect(\"generated {kind} schemas must be valid json\"))\n\
        .as_slice()\n\
}}\n\
\n\
pub fn find_by_name_and_version(name: &str, version: &str) -> Option<&'static Value> {{\n\
    all().iter().find(|schema| {{\n\
        schema.get(\"{id_field}\").and_then(Value::as_str) == Some(name)\n\
            && schema.get(\"version\").and_then(Value::as_str) == Some(version)\n\
    }})\n\
}}\n\
{tool_validation}"
    )
}

fn render_mod_file() -> String {
    "// @generated by build.rs. Do not edit manually.
pub mod agents;
pub mod claude;
pub mod evals;
pub mod mcp_servers;
pub mod prompts;
pub mod tools;

pub fn validate_all_schemas() -> anyhow::Result<()> {
    if tools::all().is_empty() {
        return Err(anyhow::anyhow!(\"no tool schemas loaded\"));
    }
    if prompts::all().is_empty() {
        return Err(anyhow::anyhow!(\"no prompt schemas loaded\"));
    }
    if evals::all().is_empty() {
        return Err(anyhow::anyhow!(\"no eval schemas loaded\"));
    }
    if agents::all().is_empty() {
        return Err(anyhow::anyhow!(\"no agent schemas loaded\"));
    }
    if mcp_servers::all().is_empty() {
        return Err(anyhow::anyhow!(\"no mcp server schemas loaded\"));
    }
    if claude::all().is_empty() {
        return Err(anyhow::anyhow!(\"no claude config schemas loaded\"));
    }
    Ok(())
}
"
    .to_string()
}

fn write_if_changed(path: &PathBuf, content: &str) -> Result<(), String> {
    let existing = fs::read_to_string(path).ok();
    if existing.as_deref() == Some(content) {
        return Ok(());
    }
    fs::write(path, content).map_err(|err| format!("failed to write {}: {err}", path.display()))
}
