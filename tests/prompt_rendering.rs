use chroma_ai_dev::prompts::{render_prompt, validate_prompt_variables, PromptRenderError};
use serde_json::json;

#[test]
fn render_default_assistant_with_valid_variables() {
    let variables = json!({
        "user_id": "alice@example.com",
        "session_id": "sess_abc123",
        "environment": "prod",
        "workspace_id": "ws_xyz",
        "allowed_tools": ["web_search", "retrieve_docs"],
        "budget_remaining": 0.75
    });

    let rendered = render_prompt("default-assistant", "1.0.0", &variables)
        .expect("rendering should succeed with valid variables");

    assert!(rendered.contains("User: alice@example.com"));
    assert!(rendered.contains("Allowed tools: web_search, retrieve_docs"));
    assert!(rendered.contains("Budget remaining: $0.75"));
}

#[test]
fn validate_prompt_variables_rejects_missing_required_fields() {
    let variables = json!({
        "user_id": "alice@example.com"
    });

    let error = validate_prompt_variables("default-assistant", "1.0.0", &variables)
        .expect_err("missing required variables should fail validation");

    match error {
        PromptRenderError::VariableValidation { issues, .. } => {
            assert!(!issues.is_empty(), "validation issues should be present");
            let joined = issues
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            assert!(joined.contains("required"));
        }
        other => panic!("unexpected error type: {other}"),
    }
}

#[test]
fn validate_prompt_variables_rejects_invalid_enum_value() {
    let variables = json!({
        "user_id": "alice@example.com",
        "session_id": "sess_abc123",
        "environment": "qa",
        "workspace_id": "ws_xyz",
        "allowed_tools": ["web_search"],
        "budget_remaining": 0.75
    });

    let error = validate_prompt_variables("default-assistant", "1.0.0", &variables)
        .expect_err("invalid enum value should fail validation");

    match error {
        PromptRenderError::VariableValidation { issues, .. } => {
            let joined = issues
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            assert!(joined.contains("not one of") || joined.contains("enum"));
        }
        other => panic!("unexpected error type: {other}"),
    }
}

#[test]
fn render_rag_query_uses_inc_helper_for_document_numbering() {
    let variables = json!({
        "query": "What are key requirements?",
        "corpus_id": "product-spec",
        "corpus_version": "v1.0",
        "retrieved_docs": [
            {
                "content": "Requirement A: Use OIDC.",
                "source_id": "spec-1",
                "score": 0.9,
                "acl_groups": ["eng"],
                "ingested_at": "2026-02-23T10:30:00Z"
            }
        ]
    });

    let rendered = render_prompt("rag-query", "1.0.0", &variables)
        .expect("rag query should render with valid variables");

    assert!(rendered.contains("Document 1"));
    assert!(rendered.contains("spec-1"));
}

#[test]
fn render_prompt_rejects_unknown_prompt_id() {
    let variables = json!({});
    let error = render_prompt("missing-prompt", "1.0.0", &variables)
        .expect_err("unknown prompt should return explicit error");

    match error {
        PromptRenderError::UnknownPrompt { id, version } => {
            assert_eq!(id, "missing-prompt");
            assert_eq!(version, "1.0.0");
        }
        other => panic!("unexpected error type: {other}"),
    }
}
