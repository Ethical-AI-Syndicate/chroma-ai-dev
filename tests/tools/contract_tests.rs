use chroma_ai_dev::tools::{run_contract_tests, validate_tool_input, ToolValidationError};
use serde_json::json;

#[test]
fn tool_web_search_contract() {
    run_tool_contracts("web_search", "1.0.0");
}

#[test]
fn tool_execute_sql_query_contract() {
    run_tool_contracts("execute_sql_query", "1.0.0");
}

#[test]
fn validation_error_contains_field_level_details() {
    let input = json!({
        "query": "",
        "max_results": -10
    });

    let error = validate_tool_input("web_search", "1.0.0", &input)
        .expect_err("invalid input should produce schema validation error");

    match error {
        ToolValidationError::SchemaViolation { issues, .. } => {
            assert!(!issues.is_empty(), "schema violation should include issues");
            let joined = issues
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            assert!(joined.contains("shorter than") || joined.contains("minimum"));
        }
        other => panic!("unexpected error type: {other}"),
    }
}

fn run_tool_contracts(tool_name: &str, version: &str) {
    let results = run_contract_tests(tool_name, version)
        .expect("contract runner should return case results for known tool");

    let failed = results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| format!("{} => {}", result.name, result.details))
        .collect::<Vec<_>>();

    assert!(
        failed.is_empty(),
        "contract test failures for {tool_name}@{version}: {}",
        failed.join(" | ")
    );

    assert!(
        !results.is_empty(),
        "contract runner returned no tests for {tool_name}@{version}"
    );
}
