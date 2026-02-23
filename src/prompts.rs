use crate::generated;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::{Map, Value};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub instance_path: String,
    pub schema_path: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum PromptRenderError {
    #[error("unknown prompt schema {id}@{version}")]
    UnknownPrompt { id: String, version: String },

    #[error("prompt schema {id}@{version} missing template")]
    MissingTemplate { id: String, version: String },

    #[error("prompt schema {id}@{version} missing variables")]
    MissingVariables { id: String, version: String },

    #[error("failed to compile variable schema for {id}@{version}: {message}")]
    VariableSchemaCompile {
        id: String,
        version: String,
        message: String,
    },

    #[error("prompt variable validation failed for {id}@{version} ({issues_count} issue(s))")]
    VariableValidation {
        id: String,
        version: String,
        issues_count: usize,
        issues: Vec<ValidationIssue>,
    },

    #[error("failed to render prompt {id}@{version}: {message}")]
    Render {
        id: String,
        version: String,
        message: String,
    },
}

pub fn validate_prompt_variables(
    prompt_id: &str,
    version: &str,
    variables: &Value,
) -> Result<(), PromptRenderError> {
    let prompt =
        generated::prompts::find_by_name_and_version(prompt_id, version).ok_or_else(|| {
            PromptRenderError::UnknownPrompt {
                id: prompt_id.to_string(),
                version: version.to_string(),
            }
        })?;

    let variable_definitions =
        prompt
            .get("variables")
            .ok_or_else(|| PromptRenderError::MissingVariables {
                id: prompt_id.to_string(),
                version: version.to_string(),
            })?;

    let variable_schema = build_variable_json_schema(variable_definitions);
    let compiled = jsonschema::JSONSchema::compile(&variable_schema).map_err(|err| {
        PromptRenderError::VariableSchemaCompile {
            id: prompt_id.to_string(),
            version: version.to_string(),
            message: err.to_string(),
        }
    })?;

    if let Err(errors) = compiled.validate(variables) {
        let issues = errors
            .map(|err| ValidationIssue {
                instance_path: err.instance_path.to_string(),
                schema_path: err.schema_path.to_string(),
                message: err.to_string(),
            })
            .collect::<Vec<_>>();

        return Err(PromptRenderError::VariableValidation {
            id: prompt_id.to_string(),
            version: version.to_string(),
            issues_count: issues.len(),
            issues,
        });
    }

    Ok(())
}

pub fn render_prompt(
    prompt_id: &str,
    version: &str,
    variables: &Value,
) -> Result<String, PromptRenderError> {
    validate_prompt_variables(prompt_id, version, variables)?;

    let prompt =
        generated::prompts::find_by_name_and_version(prompt_id, version).ok_or_else(|| {
            PromptRenderError::UnknownPrompt {
                id: prompt_id.to_string(),
                version: version.to_string(),
            }
        })?;

    let template = prompt
        .get("template")
        .and_then(Value::as_str)
        .ok_or_else(|| PromptRenderError::MissingTemplate {
            id: prompt_id.to_string(),
            version: version.to_string(),
        })?;

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_helper("inc", Box::new(inc_helper));

    handlebars
        .render_template(template, variables)
        .map_err(|err| PromptRenderError::Render {
            id: prompt_id.to_string(),
            version: version.to_string(),
            message: err.to_string(),
        })
}

fn build_variable_json_schema(variable_definitions: &Value) -> Value {
    let mut properties = Map::<String, Value>::new();
    let mut required = Vec::<Value>::new();

    if let Some(def_map) = variable_definitions.as_object() {
        for (name, def) in def_map {
            if let Some(schema_def) = def.as_object() {
                let mut property = schema_def.clone();
                let is_required = property
                    .remove("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                property.remove("description");
                properties.insert(name.to_string(), Value::Object(property));
                if is_required {
                    required.push(Value::String(name.to_string()));
                }
            }
        }
    }

    let mut schema = Map::<String, Value>::new();
    schema.insert("type".to_string(), Value::String("object".to_string()));
    schema.insert("properties".to_string(), Value::Object(properties));
    schema.insert("required".to_string(), Value::Array(required));
    schema.insert("additionalProperties".to_string(), Value::Bool(false));

    Value::Object(schema)
}

fn inc_helper(
    helper: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let value = helper
        .param(0)
        .and_then(|v| v.value().as_i64())
        .unwrap_or(0)
        + 1;
    out.write(&value.to_string())?;
    Ok(())
}
