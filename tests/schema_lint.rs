use chroma_ai_dev::schema_lint::{lint_markdown, LintLevel};

#[test]
fn lint_reports_missing_description() {
    let markdown = r#"
```yaml schema tool
name: demo_tool
version: "1.0.0"
risk_rating: low
allowed_environments: [dev]
connector_binding: demo
input_schema:
  type: object
  properties: {}
output_schema:
  type: object
  properties: {}
error_behavior:
  timeout: fail_immediately
policy_tags:
  data_classification: internal
  retention_class: SHORT
contract_tests: []
```
"#;

    let findings = lint_markdown("TOOLS.md", markdown);
    assert!(findings.iter().any(
        |finding| finding.code == "missing-description" && finding.level == LintLevel::Warning
    ));
}

#[test]
fn lint_reports_non_semver_version() {
    let markdown = r#"
```yaml schema prompt
id: test-prompt
version: "1.0"
type: user
description: x
template: hello
variables: {}
allowed_models: [claude-sonnet-4-5]
```
"#;

    let findings = lint_markdown("PROMPTS.md", markdown);
    assert!(findings
        .iter()
        .any(|finding| finding.code == "invalid-semver" && finding.level == LintLevel::Error));
}

#[test]
fn lint_passes_valid_schema_block() {
    let markdown = r#"
```yaml schema tool
name: demo_tool
version: "1.0.0"
description: Example tool
risk_rating: low
allowed_environments: [dev]
connector_binding: demo
input_schema:
  type: object
  properties: {}
output_schema:
  type: object
  properties: {}
error_behavior:
  timeout: fail_immediately
policy_tags:
  data_classification: internal
  retention_class: SHORT
contract_tests: []
```
"#;

    let findings = lint_markdown("TOOLS.md", markdown);
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}
