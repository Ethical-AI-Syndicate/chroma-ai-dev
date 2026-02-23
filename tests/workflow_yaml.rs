use serde_yaml::Value;

#[test]
fn schema_validation_workflow_yaml_is_valid() {
    let content = std::fs::read_to_string(".github/workflows/schema-validation.yml")
        .expect("workflow file should be readable");
    serde_yaml::from_str::<Value>(&content).expect("workflow yaml should parse");
}
