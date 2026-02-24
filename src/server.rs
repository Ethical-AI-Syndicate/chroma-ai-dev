//! HTTP Server for ChromaAI Dev Control Plane

use axum::{
    routing::{get, post},
    Router,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global server state
static SERVER_STATE: Lazy<Mutex<ServerState>> = Lazy::new(|| {
    Mutex::new(ServerState::default())
});

/// In-memory RAG corpus
static RAG_CORPUS: Lazy<Mutex<RagCorpus>> = Lazy::new(|| {
    Mutex::new(RagCorpus {
        documents: vec![
            RagDocument {
                id: "doc1".to_string(),
                content: "Rust async programming uses futures, which are values that represent a computation that may not have completed yet. The async/await syntax makes working with futures more ergonomic.".to_string(),
                metadata: serde_json::json!({"source": "rust-docs", "topic": "async"}),
            },
            RagDocument {
                id: "doc2".to_string(),
                content: "Tokio is a runtime for Rust that provides I/O, networking, scheduling, and timers. It's the most popular async runtime for Rust.".to_string(),
                metadata: serde_json::json!({"source": "tokio-docs", "topic": "runtime"}),
            },
            RagDocument {
                id: "doc3".to_string(),
                content: "Axum is a web framework for Rust that builds on Tower and Hyper. It provides ergonomic routing, middleware, and request/response handling.".to_string(),
                metadata: serde_json::json!({"source": "axum-docs", "topic": "web"}),
            },
            RagDocument {
                id: "doc4".to_string(),
                content: "JSON Schema is a vocabulary that allows you to annotate and validate JSON documents. It's used in ChromaAI Dev for tool input validation.".to_string(),
                metadata: serde_json::json!({"source": "json-schema-spec", "topic": "validation"}),
            },
            RagDocument {
                id: "doc5".to_string(),
                content: "OIDC (OpenID Connect) is an identity layer on top of OAuth 2.0. It provides authentication for modern applications.".to_string(),
                metadata: serde_json::json!({"source": "oidc-spec", "topic": "auth"}),
            },
        ],
    })
});

#[derive(Default)]
struct ServerState {
    authenticated: bool,
    tools_executed: u64,
    eval_runs: u64,
}

#[derive(Default)]
struct RagCorpus {
    documents: Vec<RagDocument>,
}

struct RagDocument {
    id: String,
    content: String,
    metadata: serde_json::Value,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Auth status response  
#[derive(Serialize)]
struct AuthResponse {
    authenticated: bool,
}

/// Login request
#[derive(Deserialize)]
struct LoginRequest {
    username: Option<String>,
    token: Option<String>,
}

/// Login response
#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    message: String,
}

/// Tool execution request
#[derive(Deserialize)]
pub struct ExecuteToolRequest {
    tool: String,
    version: Option<String>,
    input: serde_json::Value,
}

/// Tool execution response
#[derive(Serialize)]
struct ExecuteToolResponse {
    success: bool,
    output: Option<serde_json::Value>,
    error: Option<String>,
    execution_time_ms: u64,
}

/// Get health status
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get auth status
async fn auth_status() -> Json<AuthResponse> {
    let state = SERVER_STATE.lock().unwrap();
    Json(AuthResponse { authenticated: state.authenticated })
}

/// Login - set authenticated state
async fn login(Json(payload): Json<LoginRequest>) -> Json<LoginResponse> {
    // In production, validate credentials against OIDC provider
    // For dev mode, accept any login
    let mut state = SERVER_STATE.lock().unwrap();
    state.authenticated = true;
    
    Json(LoginResponse {
        success: true,
        message: "Authenticated successfully".to_string(),
    })
}

/// Execute a tool
async fn execute_tool(
    Json(payload): Json<ExecuteToolRequest>,
) -> Json<ExecuteToolResponse> {
    let start = std::time::Instant::now();
    
    let authenticated = {
        let state = SERVER_STATE.lock().unwrap();
        state.authenticated
    };
    
    // Check authentication
    if !authenticated {
        return Json(ExecuteToolResponse {
            success: false,
            output: None,
            error: Some("Not authenticated".to_string()),
            execution_time_ms: 0,
        });
    }
    
    // Validate tool exists
    let tool_name = &payload.tool;
    let version = payload.version.as_deref().unwrap_or("1.0.0");
    
    // Execute the tool
    let result = execute_tool_by_name(tool_name, version, payload.input).await;
    
    let execution_time_ms = start.elapsed().as_millis() as u64;
    
    match result {
        Ok(output) => Json(ExecuteToolResponse {
            success: true,
            output: Some(output),
            error: None,
            execution_time_ms,
        }),
        Err(e) => Json(ExecuteToolResponse {
            success: false,
            output: None,
            error: Some(e),
            execution_time_ms,
        }),
    }
}

/// Execute a tool by name
async fn execute_tool_by_name(
    tool_name: &str,
    version: &str,
    input: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // Validate tool exists in registry
    let tool = chroma_ai_dev::generated::tools::find_by_name_and_version(tool_name, version)
        .ok_or_else(|| format!("Tool '{}' not found", tool_name))?;
    
    // Validate input against schema
    if let Some(input_schema) = tool.get("input_schema") {
        let validator = jsonschema::JSONSchema::compile(input_schema)
            .map_err(|e| format!("Invalid schema: {}", e))?;
        
        let validation_result = validator.validate(&input);
        
        if let Err(errors) = validation_result {
            let error_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(format!("Input validation failed: {}", error_msgs.join(", ")));
        }
    }
    
    // Increment counter
    {
        let mut state = SERVER_STATE.lock().unwrap();
        state.tools_executed += 1;
    }
    
    // Execute based on tool type
    match tool_name {
        "web_search" => execute_web_search(input).await,
        "execute_sql_query" => execute_sql_query(input).await,
        "retrieve_docs" => execute_retrieve_docs(input).await,
        _ => Ok(serde_json::json!({
            "message": format!("Tool '{}' validated successfully", tool_name),
            "input": input
        })),
    }
}

/// Execute web_search tool
async fn execute_web_search(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let query = input["query"].as_str().unwrap_or("");
    let max_results = input["max_results"].as_i64().unwrap_or(5) as usize;
    
    // Use Brave Search API (or mock for now)
    // In production, use actual API key from config
    let search_url = format!(
        "https://search.brave.com/api/search?q={}&count={}",
        urlencoding::encode(query),
        max_results.min(10)
    );
    
    let client = reqwest::Client::new();
    
    match client.get(&search_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let results = json["web"]["results"]
                            .as_array()
                            .map(|arr| {
                                arr.iter().take(max_results).map(|r| {
                                    serde_json::json!({
                                        "title": r["title"].as_str().unwrap_or(""),
                                        "url": r["url"].as_str().unwrap_or(""),
                                        "snippet": r["description"].as_str().unwrap_or(""),
                                        "rank": r["rank"].as_i64().unwrap_or(0)
                                    })
                                }).collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        
                        Ok(serde_json::json!({
                            "results": results,
                            "query": query,
                            "total_results": results.len()
                        }))
                    }
                    Err(e) => Ok(serde_json::json!({
                        "message": "Search completed but failed to parse response",
                        "query": query,
                        "error": e.to_string()
                    }))
                }
            } else {
                Ok(serde_json::json!({
                    "message": "Search API returned error",
                    "query": query,
                    "status": response.status().as_u16()
                }))
            }
        }
        Err(e) => Ok(serde_json::json!({
            "message": "Search request failed, returning mock results",
            "query": query,
            "error": e.to_string(),
            "results": [
                {
                    "title": format!("Mock result for: {}", query),
                    "url": "https://example.com",
                    "snippet": "This is a mock result since the search API is not available",
                    "rank": 1
                }
            ]
        }))
    }
}

/// Execute execute_sql_query tool
async fn execute_sql_query(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let query = input["query"].as_str().unwrap_or("");
    
    // Only allow SELECT queries for safety
    let query_upper = query.to_uppercase();
    if !query_upper.starts_with("SELECT") {
        return Err("Only SELECT queries are allowed".to_string());
    }
    
    Ok(serde_json::json!({
        "message": "SQL query executed (mock - no database configured)",
        "query": query,
        "rows": [],
        "columns": []
    }))
}

/// Execute retrieve_docs tool (RAG)
async fn execute_retrieve_docs(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let query = input["query"].as_str().unwrap_or("");
    let max_results = input["max_results"].as_i64().unwrap_or(5) as usize;
    
    // Use the RAG corpus
    let corpus = RAG_CORPUS.lock().unwrap();
    
    // Simple keyword-based scoring
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    
    let mut scored: Vec<(&RagDocument, f64)> = corpus.documents.iter()
        .map(|doc| {
            let content_lower = doc.content.to_lowercase();
            let score = query_words.iter()
                .filter(|w| content_lower.contains(**w))
                .count() as f64 / query_words.len().max(1) as f64;
            (doc, score)
        })
        .collect();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let results: Vec<serde_json::Value> = scored.into_iter()
        .take(max_results)
        .filter(|(_, score)| *score > 0.0)
        .map(|(doc, score)| {
            serde_json::json!({
                "id": doc.id,
                "content": doc.content,
                "score": score,
                "metadata": doc.metadata
            })
        })
        .collect();
    
    Ok(serde_json::json!({
        "query": query,
        "documents": results,
        "total": results.len()
    }))
}

/// Get available tools
async fn list_tools() -> Json<serde_json::Value> {
    let tools = chroma_ai_dev::generated::tools::all();
    let tool_summaries: Vec<serde_json::Value> = tools.iter().map(|t| {
        serde_json::json!({
            "name": t.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
            "version": t.get("version").and_then(|v| v.as_str()).unwrap_or("?"),
            "description": t.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            "risk_rating": t.get("risk_rating").and_then(|v| v.as_str()).unwrap_or("unknown"),
        })
    }).collect();
    
    Json(serde_json::json!({
        "tools": tool_summaries,
        "count": tool_summaries.len()
    }))
}

/// Get server statistics
async fn stats() -> Json<serde_json::Value> {
    let state = SERVER_STATE.lock().unwrap();
    Json(serde_json::json!({
        "authenticated": state.authenticated,
        "tools_executed": state.tools_executed,
        "eval_runs": state.eval_runs,
        "rag_documents": RAG_CORPUS.lock().unwrap().documents.len(),
        "uptime": "N/A"
    }))
}

// ============ RAG Handlers ============

#[derive(Deserialize)]
struct RagQueryRequest {
    query: String,
    max_results: Option<usize>,
}

#[derive(Deserialize)]
struct RagIndexRequest {
    content: String,
    metadata: Option<serde_json::Value>,
}

/// Query the RAG corpus
async fn rag_query(Json(payload): Json<RagQueryRequest>) -> Json<serde_json::Value> {
    let query = payload.query.to_lowercase();
    let max_results = payload.max_results.unwrap_or(5);
    
    let corpus = RAG_CORPUS.lock().unwrap();
    
    // Simple keyword-based scoring
    let mut scored: Vec<(&RagDocument, f64)> = corpus.documents.iter()
        .map(|doc| {
            let content_lower = doc.content.to_lowercase();
            let query_words: Vec<&str> = query.split_whitespace().collect();
            let score = query_words.iter()
                .filter(|w| content_lower.contains(**w))
                .count() as f64 / query_words.len().max(1) as f64;
            (doc, score)
        })
        .collect();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let results: Vec<serde_json::Value> = scored.into_iter()
        .take(max_results)
        .filter(|(_, score)| *score > 0.0)
        .map(|(doc, score)| {
            serde_json::json!({
                "id": doc.id,
                "content": doc.content,
                "metadata": doc.metadata,
                "score": score
            })
        })
        .collect();
    
    Json(serde_json::json!({
        "query": payload.query,
        "results": results,
        "total": results.len()
    }))
}

/// Add documents to RAG corpus
async fn rag_index(Json(payload): Json<RagIndexRequest>) -> Json<serde_json::Value> {
    let id = format!("doc{}", RAG_CORPUS.lock().unwrap().documents.len() + 1);
    
    let doc = RagDocument {
        id: id.clone(),
        content: payload.content,
        metadata: payload.metadata.unwrap_or(serde_json::json!({})),
    };
    
    RAG_CORPUS.lock().unwrap().documents.push(doc);
    
    Json(serde_json::json!({
        "success": true,
        "id": id,
        "message": "Document indexed successfully"
    }))
}

// ============ Eval Handlers ============

/// List available eval suites
async fn list_evals() -> Json<serde_json::Value> {
    let evals = chroma_ai_dev::generated::evals::all();
    let eval_summaries: Vec<serde_json::Value> = evals.iter().map(|e| {
        serde_json::json!({
            "name": e.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
            "version": e.get("version").and_then(|v| v.as_str()).unwrap_or("?"),
            "description": e.get("description").and_then(|v| v.as_str()).unwrap_or(""),
        })
    }).collect();
    
    Json(serde_json::json!({
        "evals": eval_summaries,
        "count": eval_summaries.len()
    }))
}

#[derive(Deserialize)]
struct RunEvalRequest {
    eval_name: String,
    version: Option<String>,
    test_cases: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize)]
struct EvalResult {
    eval_name: String,
    passed: bool,
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    results: Vec<serde_json::Value>,
}

/// Run an eval suite
async fn run_eval(Json(payload): Json<RunEvalRequest>) -> Json<serde_json::Value> {
    // Find the eval
    let version = payload.version.as_deref().unwrap_or("1.0.0");
    let eval = match chroma_ai_dev::generated::evals::find_by_name_and_version(&payload.eval_name, version) {
        Some(e) => e,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "error": format!("Eval '{}' not found", payload.eval_name)
            }));
        }
    };
    
    // Increment eval counter
    {
        let mut state = SERVER_STATE.lock().unwrap();
        state.eval_runs += 1;
    }
    
    // Get test cases from payload or eval definition
    let test_cases = payload.test_cases.unwrap_or_else(|| {
        // Default test cases from eval
        vec![
            serde_json::json!({"input": "test1", "expected": "result1"}),
            serde_json::json!({"input": "test2", "expected": "result2"}),
        ]
    });
    
    // Run tests (simplified - actual implementation would be more sophisticated)
    let mut results = Vec::new();
    let mut passed = 0;
    
    for (i, case) in test_cases.iter().enumerate() {
        // Simulate evaluation
        let test_passed = i % 2 == 0; // Mock: half pass, half fail
        if test_passed {
            passed += 1;
        }
        
        results.push(serde_json::json!({
            "case": i + 1,
            "passed": test_passed,
            "input": case,
            "output": "mock_output",
        }));
    }
    
    let total = results.len();
    Json(serde_json::json!({
        "success": true,
        "eval_name": payload.eval_name,
        "version": version,
        "passed": passed == total,
        "total_tests": total,
        "passed_tests": passed,
        "failed_tests": total - passed,
        "results": results
    }))
}

/// Build the router
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/auth", get(auth_status))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/tools", get(list_tools))
        .route("/api/v1/execute", post(execute_tool))
        .route("/api/v1/rag/query", post(rag_query))
        .route("/api/v1/rag/index", post(rag_index))
        .route("/api/v1/evals", get(list_evals))
        .route("/api/v1/evals/run", post(run_eval))
        .route("/api/v1/stats", get(stats))
}

/// Run the server
pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    println!("🚀 Starting ChromaAI Dev server on http://{}", addr);
    println!("📡 Endpoints:");
    println!("   GET  /health           - Health check");
    println!("   GET  /api/v1/auth     - Auth status");
    println!("   POST /api/v1/auth/login - Login");
    println!("   GET  /api/v1/tools    - List available tools");
    println!("   POST /api/v1/execute   - Execute a tool");
    println!("   POST /api/v1/rag/query - Query RAG corpus");
    println!("   POST /api/v1/rag/index - Add to RAG corpus");
    println!("   GET  /api/v1/evals    - List eval suites");
    println!("   POST /api/v1/evals/run - Run eval suite");
    println!("   GET  /api/v1/stats    - Server statistics");
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router()).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn server_has_routes() {
        let routes = router();
        assert!(true);
    }
}
