use crate::{evals, generated, prompts, tools};
use semver::Version;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaKind {
    Tool,
    Prompt,
    Eval,
    Agent,
    McpServer,
    Claude,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionWarning {
    DeprecatedVersion {
        kind: String,
        id: String,
        requested: String,
        migration_guide: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionResolution {
    pub kind: SchemaKind,
    pub id: String,
    pub version: String,
    pub warnings: Vec<ResolutionWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolValidationOutcome {
    pub resolved_version: String,
    pub warnings: Vec<ResolutionWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRenderOutcome {
    pub resolved_version: String,
    pub warnings: Vec<ResolutionWarning>,
    pub rendered: String,
}

#[derive(Debug, Error)]
pub enum VersionResolutionError {
    #[error("unknown schema kind '{kind}' id '{id}'")]
    UnknownSchema { kind: String, id: String },

    #[error("unknown version '{version}' for {kind} '{id}'")]
    UnknownVersion {
        kind: String,
        id: String,
        version: String,
    },

    #[error("invalid semver '{version}' for {kind} '{id}'")]
    InvalidSemver {
        kind: String,
        id: String,
        version: String,
    },

    #[error(transparent)]
    ToolValidation(#[from] tools::ToolValidationError),

    #[error(transparent)]
    PromptRender(#[from] prompts::PromptRenderError),

    #[error(transparent)]
    EvalExecution(#[from] evals::EvalExecutionError),
}

pub fn resolve_version(
    kind: SchemaKind,
    id: &str,
    requested_version: &str,
) -> Result<VersionResolution, VersionResolutionError> {
    let mut candidates = schema_candidates(kind, id)?;
    candidates.sort_by(|a, b| a.0.cmp(&b.0));

    let (resolved_version, schema) = if requested_version == "latest" {
        let (_, version_str, schema) =
            candidates
                .last()
                .ok_or_else(|| VersionResolutionError::UnknownSchema {
                    kind: kind.as_str().to_string(),
                    id: id.to_string(),
                })?;
        (version_str.clone(), *schema)
    } else {
        let (_, version_str, schema) = candidates
            .iter()
            .find(|(_, version, _)| version == requested_version)
            .ok_or_else(|| VersionResolutionError::UnknownVersion {
                kind: kind.as_str().to_string(),
                id: id.to_string(),
                version: requested_version.to_string(),
            })?;
        (version_str.clone(), *schema)
    };

    let warnings = deprecated_warning(kind, id, &resolved_version, schema);

    Ok(VersionResolution {
        kind,
        id: id.to_string(),
        version: resolved_version,
        warnings,
    })
}

pub fn validate_tool_input_versioned(
    tool_name: &str,
    requested_version: &str,
    input: &Value,
) -> Result<ToolValidationOutcome, VersionResolutionError> {
    let resolved = resolve_version(SchemaKind::Tool, tool_name, requested_version)?;
    tools::validate_tool_input(tool_name, &resolved.version, input)?;
    Ok(ToolValidationOutcome {
        resolved_version: resolved.version,
        warnings: resolved.warnings,
    })
}

pub fn render_prompt_versioned(
    prompt_id: &str,
    requested_version: &str,
    variables: &Value,
) -> Result<PromptRenderOutcome, VersionResolutionError> {
    let resolved = resolve_version(SchemaKind::Prompt, prompt_id, requested_version)?;
    let rendered = prompts::render_prompt(prompt_id, &resolved.version, variables)?;
    Ok(PromptRenderOutcome {
        resolved_version: resolved.version,
        warnings: resolved.warnings,
        rendered,
    })
}

fn schema_candidates(
    kind: SchemaKind,
    id: &str,
) -> Result<Vec<(Version, String, &'static Value)>, VersionResolutionError> {
    let schemas = kind.schemas();
    let id_field = kind.id_field();

    let mut matches = Vec::new();
    for schema in schemas {
        if schema.get(id_field).and_then(Value::as_str) != Some(id) {
            continue;
        }
        let version = schema
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| VersionResolutionError::UnknownVersion {
                kind: kind.as_str().to_string(),
                id: id.to_string(),
                version: "<missing>".to_string(),
            })?;
        let parsed =
            Version::parse(version).map_err(|_| VersionResolutionError::InvalidSemver {
                kind: kind.as_str().to_string(),
                id: id.to_string(),
                version: version.to_string(),
            })?;
        matches.push((parsed, version.to_string(), schema));
    }

    if matches.is_empty() {
        return Err(VersionResolutionError::UnknownSchema {
            kind: kind.as_str().to_string(),
            id: id.to_string(),
        });
    }

    Ok(matches)
}

fn deprecated_warning(
    kind: SchemaKind,
    id: &str,
    resolved_version: &str,
    schema: &Value,
) -> Vec<ResolutionWarning> {
    let deprecated = schema
        .get("deprecated_versions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .any(|item| item == resolved_version)
        })
        .unwrap_or(false);

    if deprecated {
        vec![ResolutionWarning::DeprecatedVersion {
            kind: kind.as_str().to_string(),
            id: id.to_string(),
            requested: resolved_version.to_string(),
            migration_guide: schema
                .get("migration_guide")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        }]
    } else {
        Vec::new()
    }
}

impl SchemaKind {
    fn as_str(self) -> &'static str {
        match self {
            SchemaKind::Tool => "tool",
            SchemaKind::Prompt => "prompt",
            SchemaKind::Eval => "eval",
            SchemaKind::Agent => "agent",
            SchemaKind::McpServer => "mcp_server",
            SchemaKind::Claude => "claude",
        }
    }

    fn id_field(self) -> &'static str {
        match self {
            SchemaKind::Prompt => "id",
            SchemaKind::Eval => "suite_id",
            _ => "name",
        }
    }

    fn schemas(self) -> &'static [Value] {
        match self {
            SchemaKind::Tool => generated::tools::all(),
            SchemaKind::Prompt => generated::prompts::all(),
            SchemaKind::Eval => generated::evals::all(),
            SchemaKind::Agent => generated::agents::all(),
            SchemaKind::McpServer => generated::mcp_servers::all(),
            SchemaKind::Claude => generated::claude::all(),
        }
    }
}
