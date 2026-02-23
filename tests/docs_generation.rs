use chroma_ai_dev::docs_generation::generate_html_docs;
use tempfile::tempdir;

#[test]
fn generates_html_docs_for_all_schema_markdown_files() {
    let dir = tempdir().expect("temp dir should be created");
    let output = dir.path().join("html");

    let generated =
        generate_html_docs(&output).expect("html docs generation should succeed from repo root");

    assert!(generated.iter().any(|path| path.ends_with("TOOLS.html")));
    assert!(generated.iter().any(|path| path.ends_with("PROMPTS.html")));
    assert!(generated.iter().any(|path| path.ends_with("EVALS.html")));
}

#[test]
fn generated_html_contains_document_title() {
    let dir = tempdir().expect("temp dir should be created");
    let output = dir.path().join("html");

    let generated = generate_html_docs(&output).expect("html docs generation should succeed");
    let tools_html = generated
        .iter()
        .find(|path| path.ends_with("TOOLS.html"))
        .expect("TOOLS.html should be generated");

    let content = std::fs::read_to_string(tools_html).expect("generated html should be readable");
    assert!(content.contains("<title>TOOLS.md</title>"));
    assert!(content.contains("<h1>TOOLS.md"));
}
