use crate::generated;
use regex::Regex;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub instance_path: String,
    pub schema_path: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ToolValidationError {
    #[error("unknown tool schema {tool}@{version}")]
    UnknownTool { tool: String, version: String },

    #[error("tool schema {tool}@{version} missing input_schema")]
    MissingInputSchema { tool: String, version: String },

    #[error("failed to compile input schema for {tool}@{version}: {message}")]
    SchemaCompile {
        tool: String,
        version: String,
        message: String,
    },

    #[error("tool input validation failed for {tool}@{version} ({issues_count} issue(s))")]
    SchemaViolation {
        tool: String,
        version: String,
        issues_count: usize,
        issues: Vec<ValidationIssue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractCaseResult {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

pub fn validate_tool_input(
    tool_name: &str,
    version: &str,
    input: &Value,
) -> Result<(), ToolValidationError> {
    let schema =
        generated::tools::find_by_name_and_version(tool_name, version).ok_or_else(|| {
            ToolValidationError::UnknownTool {
                tool: tool_name.to_string(),
                version: version.to_string(),
            }
        })?;

    let input_schema =
        schema
            .get("input_schema")
            .ok_or_else(|| ToolValidationError::MissingInputSchema {
                tool: tool_name.to_string(),
                version: version.to_string(),
            })?;

    let compiled = jsonschema::JSONSchema::compile(input_schema).map_err(|err| {
        ToolValidationError::SchemaCompile {
            tool: tool_name.to_string(),
            version: version.to_string(),
            message: err.to_string(),
        }
    })?;

    if let Err(errors) = compiled.validate(input) {
        let issues = errors
            .map(|err| ValidationIssue {
                instance_path: err.instance_path.to_string(),
                schema_path: err.schema_path.to_string(),
                message: err.to_string(),
            })
            .collect::<Vec<_>>();

        return Err(ToolValidationError::SchemaViolation {
            tool: tool_name.to_string(),
            version: version.to_string(),
            issues_count: issues.len(),
            issues,
        });
    }

    Ok(())
}

pub fn run_contract_tests(
    tool_name: &str,
    version: &str,
) -> Result<Vec<ContractCaseResult>, ToolValidationError> {
    let schema =
        generated::tools::find_by_name_and_version(tool_name, version).ok_or_else(|| {
            ToolValidationError::UnknownTool {
                tool: tool_name.to_string(),
                version: version.to_string(),
            }
        })?;

    let Some(cases) = schema.get("contract_tests").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };

    let mut results = Vec::with_capacity(cases.len());
    for case in cases {
        let name = case
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unnamed-contract-test")
            .to_string();

        let Some(input) = case.get("input") else {
            results.push(ContractCaseResult {
                name,
                passed: false,
                details: "contract test missing input payload".to_string(),
            });
            continue;
        };

        let expect_success = case
            .get("expect_success")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let expect_error = case
            .get("expect_error")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let validation = validate_tool_input(tool_name, version, input);

        let result = if expect_success {
            match validation {
                Ok(()) => ContractCaseResult {
                    name,
                    passed: true,
                    details: "validation succeeded as expected".to_string(),
                },
                Err(err) => ContractCaseResult {
                    name,
                    passed: false,
                    details: format!("expected success, got error: {err}"),
                },
            }
        } else if expect_error {
            match validation {
                Ok(()) => ContractCaseResult {
                    name,
                    passed: false,
                    details: "expected validation error, got success".to_string(),
                },
                Err(err) => {
                    if let Some(pattern) = case.get("error_pattern").and_then(Value::as_str) {
                        match Regex::new(pattern) {
                            Ok(re) => {
                                let haystack = render_error_for_pattern_match(&err);
                                let passed = re.is_match(&haystack);
                                ContractCaseResult {
                                    name,
                                    passed,
                                    details: if passed {
                                        "validation error matched expected pattern".to_string()
                                    } else {
                                        format!(
                                            "error did not match pattern '{pattern}': {haystack}"
                                        )
                                    },
                                }
                            }
                            Err(regex_err) => ContractCaseResult {
                                name,
                                passed: false,
                                details: format!("invalid contract regex '{pattern}': {regex_err}"),
                            },
                        }
                    } else {
                        ContractCaseResult {
                            name,
                            passed: true,
                            details: "validation failed as expected".to_string(),
                        }
                    }
                }
            }
        } else {
            ContractCaseResult {
                name,
                passed: false,
                details: "contract test must set expect_success or expect_error".to_string(),
            }
        };

        results.push(result);
    }

    Ok(results)
}

fn render_error_for_pattern_match(error: &ToolValidationError) -> String {
    match error {
        ToolValidationError::SchemaViolation { issues, .. } => issues
            .iter()
            .map(|issue| issue.message.clone())
            .collect::<Vec<_>>()
            .join("; "),
        _ => error.to_string(),
    }
}
