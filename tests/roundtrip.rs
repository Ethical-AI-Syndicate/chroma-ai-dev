use chroma_ai_dev::generated;
use serde_json::Value;
use std::fs;

#[test]
fn markdown_to_generated_code_roundtrip() {
    let tools_markdown = fs::read_to_string("TOOLS.md").expect("TOOLS.md should be readable");
    let prompts_markdown = fs::read_to_string("PROMPTS.md").expect("PROMPTS.md should be readable");
    let evals_markdown = fs::read_to_string("EVALS.md").expect("EVALS.md should be readable");

    assert_eq!(
        tools_markdown.matches("```yaml schema tool").count(),
        generated::tools::all().len(),
        "generated tool count should match markdown source"
    );
    assert_eq!(
        prompts_markdown.matches("```yaml schema prompt").count(),
        generated::prompts::all().len(),
        "generated prompt count should match markdown source"
    );
    assert_eq!(
        evals_markdown.matches("```yaml schema eval").count(),
        generated::evals::all().len(),
        "generated eval count should match markdown source"
    );
}

#[test]
fn schema_stability_on_rebuild() {
    let json1: Value = serde_json::from_str(generated::tools::SCHEMAS_JSON)
        .expect("generated tool schemas should be valid json");
    let json2: Value = serde_json::from_str(generated::tools::SCHEMAS_JSON)
        .expect("generated tool schemas should be valid json");
    assert_eq!(
        json1, json2,
        "generated tool schema output must be deterministic"
    );
}
