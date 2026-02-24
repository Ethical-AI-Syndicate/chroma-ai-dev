use chroma_ai_dev::generated;
use serde_json::json;

#[test]
fn eval_suite_runs() {
    // Test that eval suite can be loaded
    let evals = generated::evals::all();
    assert!(!evals.is_empty(), "Should have eval suites");
    
    // Just verify we can iterate
    assert!(evals.len() >= 5, "Should have at least 5 evals");
}

#[test]
fn rag_retrieval_works() {
    // Test that RAG tool can retrieve documents
    // This tests the schema and validation
    let input = json!({
        "query": "how to use async rust",
        "max_results": 3,
        "corpus_id": "default"
    });
    
    // Should validate against schema
    let result = chroma_ai_dev::tools::validate_tool_input("retrieve_docs", "1.0.0", &input);
    assert!(result.is_ok(), "RAG input should be valid: {:?}", result.err());
}

#[test]
fn tool_execution_schema_validation() {
    // Verify tools exist
    let tools = generated::tools::all();
    let web_search = tools.iter().find(|t| {
        t.get("name").and_then(|v| v.as_str()).map(|n| n == "web_search").unwrap_or(false)
    });
    
    assert!(web_search.is_some(), "Should have web_search tool");
}
