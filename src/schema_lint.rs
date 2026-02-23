use semver::Version;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintLevel {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintFinding {
    pub level: LintLevel,
    pub code: String,
    pub message: String,
    pub file: String,
    pub line: usize,
}

pub fn lint_markdown(file_name: &str, markdown: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    for (block, line) in extract_schema_blocks(markdown) {
        match serde_yaml::from_str::<Value>(&block) {
            Ok(value) => {
                lint_version(file_name, line, &value, &mut findings);
                lint_description(file_name, line, &value, &mut findings);
            }
            Err(err) => findings.push(LintFinding {
                level: LintLevel::Error,
                code: "invalid-yaml".to_string(),
                message: format!("schema block is not valid YAML: {err}"),
                file: file_name.to_string(),
                line,
            }),
        }
    }
    findings
}

fn lint_version(file_name: &str, line: usize, value: &Value, findings: &mut Vec<LintFinding>) {
    let Some(version) = value.get("version").and_then(Value::as_str) else {
        findings.push(LintFinding {
            level: LintLevel::Error,
            code: "missing-version".to_string(),
            message: "schema is missing version".to_string(),
            file: file_name.to_string(),
            line,
        });
        return;
    };

    if Version::parse(version).is_err() {
        findings.push(LintFinding {
            level: LintLevel::Error,
            code: "invalid-semver".to_string(),
            message: format!("version '{version}' is not valid semver"),
            file: file_name.to_string(),
            line,
        });
    }
}

fn lint_description(file_name: &str, line: usize, value: &Value, findings: &mut Vec<LintFinding>) {
    if value.get("description").and_then(Value::as_str).is_none() {
        findings.push(LintFinding {
            level: LintLevel::Warning,
            code: "missing-description".to_string(),
            message: "schema should include description".to_string(),
            file: file_name.to_string(),
            line,
        });
    }
}

fn extract_schema_blocks(markdown: &str) -> Vec<(String, usize)> {
    let mut blocks = Vec::new();
    let mut in_schema = false;
    let mut current = String::new();
    let mut start_line = 0usize;

    for (idx, line) in markdown.lines().enumerate() {
        let line_no = idx + 1;
        if line.starts_with("```yaml schema ") {
            in_schema = true;
            current.clear();
            start_line = line_no;
            continue;
        }
        if in_schema && line.starts_with("```") {
            blocks.push((current.clone(), start_line));
            in_schema = false;
            current.clear();
            continue;
        }
        if in_schema {
            current.push_str(line);
            current.push('\n');
        }
    }

    blocks
}
