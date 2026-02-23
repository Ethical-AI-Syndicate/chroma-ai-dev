use chroma_ai_dev::generated::tools;
use regex::Regex;
use serde_json::Value;

#[test]
fn tool_web_search_contract() {
    run_tool_contracts("web_search");
}

#[test]
fn tool_execute_sql_query_contract() {
    run_tool_contracts("execute_sql_query");
}

fn run_tool_contracts(tool_name: &str) {
    let schema = tools::all()
        .iter()
        .find(|schema| schema.get("name").and_then(Value::as_str) == Some(tool_name))
        .expect("tool schema must exist");

    let version = schema
        .get("version")
        .and_then(Value::as_str)
        .expect("tool version required");

    let contract_tests = schema
        .get("contract_tests")
        .and_then(Value::as_array)
        .expect("tool contract_tests must exist");

    for test_case in contract_tests {
        let test_name = test_case
            .get("name")
            .and_then(Value::as_str)
            .expect("test case must have name");
        let input = test_case
            .get("input")
            .expect("test case must include input payload");

        let expect_success = test_case
            .get("expect_success")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let expect_error = test_case
            .get("expect_error")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let result = tools::validate_tool_input(tool_name, version, input);

        if expect_success {
            assert!(
                result.is_ok(),
                "contract test '{test_name}' expected success but failed: {:?}",
                result.err()
            );
            continue;
        }

        if expect_error {
            let error = result.expect_err("expected validation error").to_string();
            if let Some(pattern) = test_case.get("error_pattern").and_then(Value::as_str) {
                let regex = Regex::new(pattern).expect("error_pattern must be valid regex");
                assert!(
                    regex.is_match(&error),
                    "contract test '{test_name}' did not match error pattern '{pattern}'. got: {error}"
                );
            }
        }
    }
}
