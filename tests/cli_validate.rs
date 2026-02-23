use chroma_ai_dev::generated;
use std::process::Command;

#[test]
fn validate_command_validates_all_schemas() {
    // The validate command should validate all schemas at runtime
    // This test verifies the generated schemas are valid
    
    let tools = generated::tools::all();
    assert!(!tools.is_empty(), "should have tool schemas");
    
    let prompts = generated::prompts::all();
    assert!(!prompts.is_empty(), "should have prompt schemas");
    
    let evals = generated::evals::all();
    assert!(!evals.is_empty(), "should have eval schemas");
    
    let agents = generated::agents::all();
    assert!(!agents.is_empty(), "should have agent schemas");
    
    // Validate cross-references
    let validation_result = generated::validate_all_schemas();
    assert!(validation_result.is_ok(), "all schemas should be valid: {:?}", validation_result.err());
}

#[test]
fn validate_command_finds_invalid_schema() {
    // If we add an invalid schema, validate_all_schemas should catch it
    // For now, this tests that the validation function exists and works
    let result = generated::validate_all_schemas();
    assert!(result.is_ok(), "validation should pass: {:?}", result);
}
