use chroma_ai_dev::generated;
use serde_json::Value;
use std::collections::HashSet;

#[test]
fn all_schemas_parse_and_validate() {
    generated::validate_all_schemas().expect("generated schemas should validate");
}

#[test]
fn no_duplicate_ids() {
    assert_unique(generated::tools::all(), "name");
    assert_unique(generated::prompts::all(), "id");
    assert_unique(generated::evals::all(), "suite_id");
    assert_unique(generated::agents::all(), "name");
    assert_unique(generated::mcp_servers::all(), "name");
    assert_unique(generated::claude::all(), "name");
}

#[test]
fn cross_references_resolve() {
    let mut valid_models = HashSet::new();
    for config in generated::claude::all() {
        if let Some(models) = config.get("models").and_then(Value::as_array) {
            for model in models {
                if let Some(model_id) = model.get("model_id").and_then(Value::as_str) {
                    valid_models.insert(model_id.to_string());
                }
            }
        }
    }

    for prompt in generated::prompts::all() {
        if let Some(models) = prompt.get("allowed_models").and_then(Value::as_array) {
            for model in models {
                let model = model
                    .as_str()
                    .expect("allowed_models entries should be strings");
                assert!(
                    valid_models.contains(model),
                    "prompt references unknown model: {model}"
                );
            }
        }
    }
}

#[test]
fn version_format_valid() {
    for schemas in [
        generated::tools::all(),
        generated::prompts::all(),
        generated::evals::all(),
        generated::agents::all(),
        generated::mcp_servers::all(),
        generated::claude::all(),
    ] {
        for schema in schemas {
            let version = schema
                .get("version")
                .and_then(Value::as_str)
                .expect("schema must contain version string");
            semver::Version::parse(version).expect("schema version must be valid semver");
        }
    }
}

#[test]
fn policy_tags_valid() {
    let valid_classifications = ["public", "internal", "confidential", "restricted", "varies"];
    let valid_retention = ["NONE", "SHORT", "STANDARD", "LEGAL_HOLD"];

    for schemas in [
        generated::tools::all(),
        generated::prompts::all(),
        generated::evals::all(),
        generated::agents::all(),
        generated::claude::all(),
    ] {
        for schema in schemas {
            if let Some(tags) = schema.get("policy_tags") {
                if let Some(classification) =
                    tags.get("data_classification").and_then(Value::as_str)
                {
                    assert!(
                        valid_classifications.contains(&classification),
                        "invalid data_classification: {classification}"
                    );
                }
                if let Some(retention) = tags.get("retention_class").and_then(Value::as_str) {
                    assert!(
                        valid_retention.contains(&retention),
                        "invalid retention_class: {retention}"
                    );
                }
            }
        }
    }
}

#[test]
fn tool_schema_count_meets_phase1_target() {
    assert!(
        generated::tools::all().len() >= 10,
        "phase 1 target requires at least 10 tool schemas; found {}",
        generated::tools::all().len()
    );
}

#[test]
fn uniqueness_allows_same_id_across_versions() {
    let sample = vec![
        serde_json::json!({"name": "demo", "version": "1.0.0"}),
        serde_json::json!({"name": "demo", "version": "1.1.0"}),
    ];

    assert_unique(&sample, "name");
}

fn assert_unique(schemas: &[Value], id_field: &str) {
    let mut keys = HashSet::new();
    for schema in schemas {
        let id = schema
            .get(id_field)
            .and_then(Value::as_str)
            .expect("schema must include identifier field")
            .to_string();
        let version = schema
            .get("version")
            .and_then(Value::as_str)
            .expect("schema must include version field")
            .to_string();
        let key = (id.clone(), version.clone());
        assert!(
            keys.insert(key),
            "duplicate schema id+version pair: {id}@{version}"
        );
    }
}
