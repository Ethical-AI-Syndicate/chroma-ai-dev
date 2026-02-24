use chroma_ai_dev::generated;
use serde_json::json;

#[test]
fn web_search_tool_accepts_valid_input() {
    // Test that web_search tool validates valid input
    let input = json!({
        "query": "rust async programming",
        "max_results": 5,
        "safe_search": true
    });
    
    let result = chroma_ai_dev::tools::validate_tool_input("web_search", "1.0.0", &input);
    assert!(result.is_ok(), "Valid input should pass: {:?}", result.err());
}

#[test]
fn web_search_tool_rejects_empty_query() {
    // Test that web_search tool rejects empty query
    let input = json!({
        "query": "",
        "max_results": 5
    });
    
    let result = chroma_ai_dev::tools::validate_tool_input("web_search", "1.0.0", &input);
    assert!(result.is_err(), "Empty query should be rejected");
}

#[test]
fn tool_registry_has_web_search() {
    // Verify web_search is in the registry
    let tool = generated::tools::find_by_name_and_version("web_search", "1.0.0");
    assert!(tool.is_some(), "web_search should be in registry");
    
    let tool = tool.unwrap();
    assert_eq!(tool["name"], "web_search");
    assert_eq!(tool["risk_rating"], "low");
}
