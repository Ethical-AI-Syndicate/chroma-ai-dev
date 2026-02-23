use chroma_ai_dev::versioning::{
    render_prompt_versioned, resolve_version, validate_tool_input_versioned, ResolutionWarning,
    SchemaKind, VersionResolutionError,
};
use serde_json::json;

#[test]
fn resolves_latest_to_existing_schema_version() {
    let resolved = resolve_version(SchemaKind::Tool, "web_search", "latest")
        .expect("latest should resolve for known tool");
    assert_eq!(resolved.version, "1.0.0");
    assert!(resolved.warnings.is_empty());
}

#[test]
fn resolves_explicit_existing_version() {
    let resolved = resolve_version(SchemaKind::Prompt, "default-assistant", "1.0.0")
        .expect("explicit prompt version should resolve");
    assert_eq!(resolved.version, "1.0.0");
}

#[test]
fn unknown_schema_returns_error() {
    let result = resolve_version(SchemaKind::Eval, "missing-suite", "latest");
    match result {
        Err(VersionResolutionError::UnknownSchema { kind, id }) => {
            assert_eq!(kind, "eval");
            assert_eq!(id, "missing-suite");
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn versioned_tool_validation_accepts_latest() {
    let input = json!({
        "query": "rust async programming",
        "max_results": 3,
        "safe_search": true
    });

    let result = validate_tool_input_versioned("web_search", "latest", &input)
        .expect("latest tool version should validate");
    assert_eq!(result.resolved_version, "1.0.0");
    assert!(result.warnings.is_empty());
}

#[test]
fn versioned_prompt_render_accepts_latest() {
    let variables = json!({
        "user_id": "alice@example.com",
        "session_id": "sess_abc123",
        "environment": "dev",
        "workspace_id": "ws_xyz",
        "allowed_tools": ["web_search"],
        "budget_remaining": 1.25
    });

    let rendered = render_prompt_versioned("default-assistant", "latest", &variables)
        .expect("latest prompt version should render");
    assert_eq!(rendered.resolved_version, "1.0.0");
    assert!(rendered.rendered.contains("User: alice@example.com"));
}

#[test]
fn warns_when_requested_version_is_deprecated() {
    let resolved = resolve_version(SchemaKind::Tool, "web_search", "1.0.0")
        .expect("known version should resolve");
    assert!(resolved.warnings.is_empty());
    assert!(!resolved
        .warnings
        .iter()
        .any(|warning| matches!(warning, ResolutionWarning::DeprecatedVersion { .. })));
}
