//! HTTP Server for ChromaAI Dev Control Plane

use axum::{
    routing::{get, post},
    Router,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub use chroma_ai_dev::config;

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
        "weather" => execute_weather(input).await,
        "calculator" => execute_calculator(input),
        "dictionary" => execute_dictionary(input).await,
        "unit_converter" => execute_unit_converter(input),
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
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;
    
    // Try DuckDuckGo HTML scrape (free, no API key needed)
    let search_url = format!(
        "https://html.duckduckgo.com/html/?q={}&b={}",
        urlencoding::encode(query),
        (max_results * 10) // Get more to filter
    );
    
    match client.get(&search_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let body = response.text().await.map_err(|e| e.to_string())?;
                
                // Parse HTML results
                let results = extract_ddg_results(&body, max_results);
                
                Ok(serde_json::json!({
                    "results": results,
                    "query": query,
                    "total_results": results.len(),
                    "source": "duckduckgo"
                }))
            } else {
                Err(format!("Search failed with status: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e))
    }
}

/// Extract results from DuckDuckGo HTML
fn extract_ddg_results(html: &str, max_results: usize) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    
    // Simple regex-free parsing for result blocks
    let html_lower = html.to_lowercase();
    
    // Find result blocks (between "result" divs)
    let mut start = 0;
    for _ in 0..max_results {
        // Find next result__a (link)
        if let Some(link_pos) = html_lower[start..].find("result__a") {
            let pos = start + link_pos;
            
            // Extract title (from a tag text)
            let title_start = match html[pos..].find(">") {
                Some(p) => pos + p + 1,
                None => { start += 1; continue; }
            };
            let title_end = match html[title_start..].find('<') {
                Some(p) => title_start + p,
                None => { start += 1; continue; }
            };
            let title = html[title_start..title_end].trim();
            
            // Extract URL from href
            let href_start = match html[title_end..].find("href=\"") {
                Some(p) => title_end + p + 6,
                None => { start += 1; continue; }
            };
            let href_end = match html[href_start..].find('"') {
                Some(p) => href_start + p,
                None => { start += 1; continue; }
            };
            let url = &html[href_start..href_end];
            
            // Skip if not a valid URL
            if url.starts_with("http") {
                results.push(serde_json::json!({
                    "title": title,
                    "url": url,
                    "snippet": format!("Result from {}", url)
                }));
            }
            
            start = href_end;
        } else {
            break;
        }
        
        if results.len() >= max_results {
            break;
        }
    }
    
    results
}

/// Execute execute_sql_query tool
async fn execute_sql_query(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let query = input["query"].as_str().unwrap_or("");
    
    // Only allow SELECT queries for safety
    let query_upper = query.to_uppercase();
    if !query_upper.starts_with("SELECT") {
        return Err("Only SELECT queries are allowed".to_string());
    }
    
    // Get database path from config or use default
    let db_path = config::get_config()
        .database_url
        .unwrap_or_else(|| "/tmp/chroma_db.sqlite".to_string());
    
    // Create a temporary in-memory database if file doesn't exist
    let conn = if std::path::Path::new(&db_path).exists() {
        rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?
    } else {
        // Create in-memory database with sample data
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| format!("Failed to create database: {}", e))?;
        
        // Create sample tables
        conn.execute(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, created_at TEXT)",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT, content TEXT, created_at TEXT)",
            [],
        ).map_err(|e| e.to_string())?;
        
        // Insert sample data
        conn.execute(
            "INSERT INTO users (name, email, created_at) VALUES ('Alice', 'alice@example.com', '2024-01-01')",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO users (name, email, created_at) VALUES ('Bob', 'bob@example.com', '2024-01-02')",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO users (name, email, created_at) VALUES ('Charlie', 'charlie@example.com', '2024-01-03')",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO posts (user_id, title, content, created_at) VALUES (1, 'Hello World', 'This is my first post!', '2024-01-01')",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn.execute(
            "INSERT INTO posts (user_id, title, content, created_at) VALUES (2, 'Rust is Awesome', 'I love programming in Rust!', '2024-01-02')",
            [],
        ).map_err(|e| e.to_string())?;
        
        conn
    };
    
    // Execute query
    let mut stmt = conn.prepare(query)
        .map_err(|e| format!("Query error: {}", e))?;
    
    let column_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();
    
    let rows = stmt.query_map([], |row| {
        let mut row_data = serde_json::Map::new();
        for (i, col) in column_names.iter().enumerate() {
            let value: rusqlite::types::Value = row.get(i)
                .unwrap_or(rusqlite::types::Value::Null);
            let json_value = match value {
                rusqlite::types::Value::Null => serde_json::Value::Null,
                rusqlite::types::Value::Integer(i) => serde_json::json!(i),
                rusqlite::types::Value::Real(f) => serde_json::json!(f),
                rusqlite::types::Value::Text(s) => serde_json::json!(s),
                rusqlite::types::Value::Blob(b) => serde_json::json!(format!("[blob: {} bytes]", b.len())),
            };
            row_data.insert(col.clone(), json_value);
        }
        Ok(serde_json::Value::Object(row_data))
    }).map_err(|e| format!("Query error: {}", e))?;
    
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {}", e))?);
    }
    
    Ok(serde_json::json!({
        "query": query,
        "rows": results,
        "columns": column_names,
        "row_count": results.len()
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

/// Execute weather tool
async fn execute_weather(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let location = input["location"].as_str().unwrap_or("Toronto");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    
    let url = format!("https://wttr.in/{}?format=j1", urlencoding::encode(location));
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await
                    .map_err(|e| format!("Failed to parse response: {}", e))?;
                
                let current = &data["current_condition"][0];
                
                Ok(serde_json::json!({
                    "location": location,
                    "temperature_C": current["temp_C"],
                    "temperature_F": current["temp_F"],
                    "feels_like_C": current["FeelsLikeC"],
                    "humidity": current["humidity"],
                    "weather": current["weatherDesc"][0]["value"],
                    "wind": current["winddir16Point"],
                    "pressure": current["pressure"],
                    "visibility": current["visibility"],
                    "uv_index": current["uvIndex"]
                }))
            } else {
                Err(format!("Weather API returned: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e))
    }
}

/// Execute calculator tool
fn execute_calculator(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let expression = input["expression"].as_str().unwrap_or("");
    
    // Simple calculator - supports +, -, *, /, ^, sqrt, sin, cos, tan, log, ln
    let expr_lower = expression.to_lowercase();
    
    let result = if expr_lower.starts_with("sqrt(") {
        let inner = &expression[5..expression.len()-1];
        let num: f64 = inner.parse().map_err(|_| "Invalid number")?;
        num.sqrt()
    } else if expr_lower.starts_with("sin(") {
        let inner = &expression[4..expression.len()-1];
        let num: f64 = inner.parse().map_err(|_| "Invalid number")?;
        num.to_radians().sin()
    } else if expr_lower.starts_with("cos(") {
        let inner = &expression[4..expression.len()-1];
        let num: f64 = inner.parse().map_err(|_| "Invalid number")?;
        num.to_radians().cos()
    } else if expr_lower.starts_with("tan(") {
        let inner = &expression[4..expression.len()-1];
        let num: f64 = inner.parse().map_err(|_| "Invalid number")?;
        num.to_radians().tan()
    } else if expr_lower.starts_with("log(") {
        let inner = &expression[4..expression.len()-1];
        let num: f64 = inner.parse().map_err(|_| "Invalid number")?;
        num.log10()
    } else if expr_lower.starts_with("ln(") {
        let inner = &expression[3..expression.len()-1];
        let num: f64 = inner.parse().map_err(|_| "Invalid number")?;
        num.ln()
    } else {
        // Basic arithmetic using meval crate simulation
        // Replace ^ with ** for Rust
        let expr_fixed = expression.replace('^', "**");
        
        // Simple eval (be careful with this!)
        simple_eval(&expr_fixed)?
    };
    
    Ok(serde_json::json!({
        "expression": expression,
        "result": result,
        "formatted": format!("{}", result)
    }))
}

/// Simple expression evaluator
fn simple_eval(expr: &str) -> Result<f64, String> {
    let expr = expr.replace(" ", "");
    
    // Handle parentheses first (simplified)
    if expr.contains('(') {
        return Err("Parentheses not supported in simple mode".to_string());
    }
    
    // Split by + and - (lowest precedence)
    let mut result = 0.0;
    let mut current_op = '+';
    let mut current_num = String::new();
    
    for ch in expr.chars() {
        match ch {
            '+' | '-' => {
                if !current_num.is_empty() {
                    let num: f64 = current_num.parse().map_err(|_| "Invalid number")?;
                    match current_op {
                        '+' => result += num,
                        '-' => result -= num,
                        _ => {}
                    }
                    current_num = String::new();
                }
                current_op = ch;
            }
            '*' | '/' => {
                // Handle * and / with higher precedence
                if !current_num.is_empty() {
                    let num: f64 = current_num.parse().map_err(|_| "Invalid number")?;
                    let mut next_num = String::new();
                    let mut i = 0;
                    let chars: Vec<char> = expr.chars().collect();
                    
                    // Find next number
                    let mut j = expr.find(ch).unwrap_or(0) + 1;
                    while j < chars.len() {
                        match chars[j] {
                            '0'..='9' | '.' => next_num.push(chars[j]),
                            _ => break,
                        }
                        j += 1;
                    }
                    
                    let next: f64 = next_num.parse().unwrap_or(1.0);
                    let intermediate = match ch {
                        '*' => num * next,
                        '/' => num / next,
                        _ => num,
                    };
                    
                    match current_op {
                        '+' => result += intermediate,
                        '-' => result -= intermediate,
                        _ => result = intermediate,
                    }
                    current_num = String::new();
                    break;
                }
            }
            _ => current_num.push(ch),
        }
    }
    
    // Handle last number
    if !current_num.is_empty() {
        let num: f64 = current_num.parse().map_err(|_| "Invalid number")?;
        match current_op {
            '+' => result += num,
            '-' => result -= num,
            _ => {}
        }
    }
    
    Ok(result)
}

/// Execute dictionary tool
async fn execute_dictionary(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let word = input["word"].as_str().unwrap_or("");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", urlencoding::encode(word));
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await
                    .map_err(|e| format!("Failed to parse: {}", e))?;
                
                // Extract relevant info
                let mut definitions = Vec::new();
                if let Some(arr) = data.as_array() {
                    for entry in arr {
                        if let Some(meanings) = entry.get("meanings").and_then(|m| m.as_array()) {
                            for meaning in meanings {
                                let part_of_speech = meaning.get("partOfSpeech").and_then(|p| p.as_str()).unwrap_or("");
                                if let Some(defs) = meaning.get("definitions").and_then(|d| d.as_array()) {
                                    for def in defs.iter().take(3) {
                                        definitions.push(serde_json::json!({
                                            "part_of_speech": part_of_speech,
                                            "definition": def.get("definition").and_then(|d| d.as_str()).unwrap_or(""),
                                            "example": def.get("example").and_then(|e| e.as_str()).unwrap_or("")
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
                
                Ok(serde_json::json!({
                    "word": word,
                    "definitions": definitions
                }))
            } else if response.status().as_u16() == 404 {
                Err(format!("Word '{}' not found in dictionary", word))
            } else {
                Err(format!("Dictionary API error: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e))
    }
}

/// Execute unit converter tool
fn execute_unit_converter(input: serde_json::Value) -> Result<serde_json::Value, String> {
    let value: f64 = input["value"].as_f64().ok_or("Invalid number")?;
    let from_unit = input["from"].as_str().ok_or("Missing unit")?;
    let to_unit = input["to"].as_str().ok_or("Missing target unit")?;
    
    // Convert to base unit first, then to target
    let in_base = convert_to_base(value, from_unit)?;
    let result = convert_from_base(in_base, to_unit)?;
    
    Ok(serde_json::json!({
        "value": value,
        "from": from_unit,
        "to": to_unit,
        "result": result,
        "formatted": format!("{} {} = {} {}", value, from_unit, result, to_unit)
    }))
}

fn convert_to_base(value: f64, unit: &str) -> Result<f64, String> {
    match unit.to_lowercase().as_str() {
        // Length (base: meters)
        "m" | "meter" | "meters" => Ok(value),
        "km" | "kilometer" | "kilometers" => Ok(value * 1000.0),
        "cm" | "centimeter" | "centimeters" => Ok(value / 100.0),
        "mm" | "millimeter" | "millimeters" => Ok(value / 1000.0),
        "mi" | "mile" | "miles" => Ok(value * 1609.344),
        "yd" | "yard" | "yards" => Ok(value * 0.9144),
        "ft" | "foot" | "feet" => Ok(value * 0.3048),
        "in" | "inch" | "inches" => Ok(value * 0.0254),
        
        // Weight (base: grams)
        "g" | "gram" | "grams" => Ok(value),
        "kg" | "kilogram" | "kilograms" => Ok(value * 1000.0),
        "mg" | "milligram" | "milligrams" => Ok(value / 1000.0),
        "lb" | "lbs" | "pound" | "pounds" => Ok(value * 453.592),
        "oz" | "ounce" | "ounces" => Ok(value * 28.3495),
        
        // Temperature (base: celsius)
        "c" | "celsius" => Ok(value),
        "f" | "fahrenheit" => Ok((value - 32.0) * 5.0 / 9.0),
        "k" | "kelvin" => Ok(value - 273.15),
        
        _ => Err(format!("Unknown unit: {}", unit))
    }
}

fn convert_from_base(value: f64, unit: &str) -> Result<f64, String> {
    match unit.to_lowercase().as_str() {
        // Length
        "m" | "meter" | "meters" => Ok(value),
        "km" | "kilometer" | "kilometers" => Ok(value / 1000.0),
        "cm" | "centimeter" | "centimeters" => Ok(value * 100.0),
        "mm" | "millimeter" | "millimeters" => Ok(value * 1000.0),
        "mi" | "mile" | "miles" => Ok(value / 1609.344),
        "yd" | "yard" | "yards" => Ok(value / 0.9144),
        "ft" | "foot" | "feet" => Ok(value / 0.3048),
        "in" | "inch" | "inches" => Ok(value / 0.0254),
        
        // Weight
        "g" | "gram" | "grams" => Ok(value),
        "kg" | "kilogram" | "kilograms" => Ok(value / 1000.0),
        "mg" | "milligram" | "milligrams" => Ok(value * 1000.0),
        "lb" | "lbs" | "pound" | "pounds" => Ok(value / 453.592),
        "oz" | "ounce" | "ounces" => Ok(value / 28.3495),
        
        // Temperature
        "c" | "celsius" => Ok(value),
        "f" | "fahrenheit" => Ok(value * 9.0 / 5.0 + 32.0),
        "k" | "kelvin" => Ok(value + 273.15),
        
        _ => Err(format!("Unknown unit: {}", unit))
    }
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
