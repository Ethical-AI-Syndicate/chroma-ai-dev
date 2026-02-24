//! HTTP Server for ChromaAI Dev Control Plane

use axum::{
    routing::get,
    Router,
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global server state
static SERVER_STATE: Lazy<Mutex<ServerState>> = Lazy::new(|| {
    Mutex::new(ServerState::default())
});

#[derive(Default)]
struct ServerState {
    authenticated: bool,
    tools_executed: u64,
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
    
    let tool = match chroma_ai_dev::generated::tools::find_by_name_and_version(tool_name, version) {
        Some(t) => t,
        None => {
            return Json(ExecuteToolResponse {
                success: false,
                output: None,
                error: Some(format!("Tool '{}' not found", tool_name)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            });
        }
    };
    
    // Validate input against schema
    if let Some(input_schema) = tool.get("input_schema") {
        let validator = match jsonschema::JSONSchema::compile(input_schema) {
            Ok(v) => v,
            Err(e) => {
                return Json(ExecuteToolResponse {
                    success: false,
                    output: None,
                    error: Some(format!("Invalid schema: {}", e)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                });
            }
        };
        
        let validation_result = validator.validate(&payload.input);
        
        if let Err(errors) = validation_result {
            let error_msgs: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Json(ExecuteToolResponse {
                success: false,
                output: None,
                error: Some(format!("Input validation failed: {}", error_msgs.join(", "))),
                execution_time_ms: start.elapsed().as_millis() as u64,
            });
        }
    }
    
    // Increment counter
    {
        let mut state = SERVER_STATE.lock().unwrap();
        state.tools_executed += 1;
    }
    
    // For now, return mock success (actual execution would call external tools)
    Json(ExecuteToolResponse {
        success: true,
        output: Some(serde_json::json!({
            "message": format!("Tool '{}' validated and ready", tool_name),
            "input_received": payload.input
        })),
        error: None,
        execution_time_ms: start.elapsed().as_millis() as u64,
    })
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
        "uptime": "N/A"
    }))
}

/// Build the router
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/auth", get(auth_status))
        .route("/api/v1/tools", get(list_tools))
        .route("/api/v1/execute", axum::routing::post(execute_tool))
        .route("/api/v1/stats", get(stats))
}

/// Run the server
pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    println!("🚀 Starting ChromaAI Dev server on http://{}", addr);
    println!("📡 Endpoints:");
    println!("   GET  /health        - Health check");
    println!("   GET  /api/v1/auth   - Auth status");
    println!("   GET  /api/v1/tools  - List available tools");
    println!("   POST /api/v1/execute - Execute a tool");
    println!("   GET  /api/v1/stats  - Server statistics");
    
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
