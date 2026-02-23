---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft
---

# CLAUDE.md - Claude-Specific Configuration

This file serves dual purposes:
1. **Part A:** Project-specific instructions that extend/override global `~/.claude/CLAUDE.md`
2. **Part B:** Claude API integration specifications for ChromaAI Dev

---

## Part A: Project-Specific Instructions (Extends ~/.claude/CLAUDE.md)

### ChromaAI Dev Specific Rules

This section extends the global CLAUDE.md instructions with project-specific requirements for ChromaAI Dev development.

#### AUTHENTIK Integration (CRITICAL - NO EXCEPTIONS)

**Rule: AUTHENTIK SSO must be setup properly - no shortcuts or workarounds**

- Real OIDC device authorization flow (not mocked, not stubbed)
- Test token acquisition, refresh, and expiration handling
- Validate identity binding on every request (actor_id, session_id, device_id)
- Server-side token validation (never trust client-provided tokens)
- Break-glass elevation must work and be audited

**Validation requirements:**
```rust
// Example of what "irrefutable proof" looks like:

#[test]
async fn test_oidc_device_flow_end_to_end() {
    // 1. Initiate device flow
    let device_code_response = initiate_device_flow().await.unwrap();
    assert!(device_code_response.device_code.len() > 0);
    assert!(device_code_response.user_code.len() > 0);
    assert!(device_code_response.verification_uri.starts_with("https://"));

    // 2. Simulate user authorization (in test: auto-approve)
    approve_device_code(&device_code_response.device_code).await.unwrap();

    // 3. Poll for token
    let token_response = poll_for_token(&device_code_response.device_code).await.unwrap();
    assert!(token_response.access_token.len() > 0);
    assert!(token_response.refresh_token.is_some());

    // 4. Verify token is valid
    let user_info = get_user_info(&token_response.access_token).await.unwrap();
    assert_eq!(user_info.sub, "expected-user-id");

    // 5. Test token refresh
    let refreshed = refresh_token(&token_response.refresh_token.unwrap()).await.unwrap();
    assert!(refreshed.access_token != token_response.access_token);

    // 6. Test expired token handling
    let expired_token = create_expired_token();
    let result = get_user_info(&expired_token).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::TokenExpired));
}
```

**What counts as a shortcut (FORBIDDEN):**
- Mocking the OIDC provider responses
- Skipping token validation
- Hard-coding tokens or user IDs
- "TODO: implement real auth later"
- Using HTTP instead of HTTPS
- Disabling certificate validation

**What counts as proper setup (REQUIRED):**
- Real AUTHENTIK instance (can be dev instance, not production)
- Full OIDC device flow implementation
- Token refresh logic with exponential backoff
- Expired token detection and re-authentication
- Server-side token introspection/validation
- Audit logging of all auth events

#### Testing Standards (PRODUCTION URLs ONLY)

**Rule: STOP TESTING WITH LOCALHOST - this is a production system**

```rust
// BAD - Don't do this:
const API_URL: &str = "http://localhost:3000";

// GOOD - Do this:
const API_URL: &str = env!("CHROMA_API_URL"); // Must be set to prod-like URL
```

**What "production URLs" means:**
- HTTPS (not HTTP)
- Real domain names (not localhost, not 127.0.0.1)
- Proper TLS certificates (not self-signed, or pinned if self-signed)
- DNS resolution required
- Production-like network topology (firewall rules, proxies, etc.)

**Local development environment:**
- Use Docker Compose to run services with real hostnames
- Use mkcert or similar for local TLS
- Configure /etc/hosts if needed for domain resolution
- Never commit localhost URLs to code

**Visual validation for TUI:**
- Screenshots or automated visual regression tests
- Verify ANSI color codes render correctly
- Verify layout doesn't break with long strings
- Test terminal width changes (responsive)

#### Architecture Validation (Server-Authoritative)

**Thin-Client Principle Enforcement:**

Every feature must be validated against these questions:

1. **Who decides?** If the answer involves policy, artifacts, budgets, or audit → Server decides
2. **Who stores?** If the answer involves credentials, artifacts, or audit logs → Server stores
3. **Who validates?** Client validates for UX, Server validates for security (both!)

**Examples:**

| Feature | Client Responsibility | Server Responsibility |
|---------|----------------------|----------------------|
| Publish prompt | Validate template syntax, show preview | Validate policy, version, store artifact, audit log |
| Execute run | Validate tool schemas exist | Enforce policy, track budget, execute, audit log |
| Promote release | Show diff, request approval | Enforce RBAC, require approvals, validate evals, audit |
| Tool invocation | Validate input schema | Enforce ACL, execute sandboxed, audit |

**Validation test pattern:**
```rust
#[test]
async fn test_client_cannot_bypass_server_policy() {
    // Client sends request that would be denied by policy
    let request = CreateRunRequest {
        tools: vec!["forbidden_tool".to_string()],
        // ... other fields
    };

    // Server MUST reject, even if client thinks it's valid
    let response = control_plane_client.create_run(request).await;

    assert!(response.is_err());
    assert!(matches!(response.unwrap_err(), ApiError::PolicyDenial { .. }));

    // Verify audit log recorded the denial
    let audit_logs = query_audit_log(request_id).await.unwrap();
    assert_eq!(audit_logs[0].decision, PolicyDecision::Deny);
    assert!(audit_logs[0].reason.contains("forbidden_tool"));
}
```

#### Validation Requirements (Irrefutable Proof)

**Rule: HTTP status codes are NOT sufficient proof**

```rust
// BAD - Insufficient validation:
#[test]
async fn test_create_prompt() {
    let response = client.create_prompt(prompt_data).await;
    assert_eq!(response.status(), 200); // ❌ NOT ENOUGH!
}

// GOOD - Irrefutable proof:
#[test]
async fn test_create_prompt() {
    let prompt_data = PromptCreate { /* ... */ };

    // 1. Create prompt
    let response = client.create_prompt(prompt_data.clone()).await.unwrap();
    assert_eq!(response.status, 201);
    let prompt_id = response.prompt_id;

    // 2. Verify it exists in artifact store
    let retrieved = client.get_prompt(&prompt_id).await.unwrap();
    assert_eq!(retrieved.template, prompt_data.template);
    assert_eq!(retrieved.version, "1.0");

    // 3. Verify audit log entry
    let audit_log = query_audit_log_for_prompt(&prompt_id).await.unwrap();
    assert_eq!(audit_log.action, "prompt.create");
    assert_eq!(audit_log.actor_id, current_user_id());

    // 4. Verify immutability (cannot modify after creation)
    let modify_result = client.update_prompt(&prompt_id, new_template).await;
    assert!(modify_result.is_err());
    assert!(matches!(modify_result.unwrap_err(), ApiError::ImmutableArtifact));
}
```

**What "irrefutable proof" requires:**
- Verify the expected state change occurred (database, filesystem, etc.)
- Verify audit logs were written
- Verify side effects (notifications, cache invalidation, etc.)
- Verify negative cases (cannot bypass, cannot modify immutable, etc.)

**Complete test coverage includes:**
- Happy path (expected success case)
- Error cases (invalid input, missing permissions, etc.)
- Edge cases (boundary values, empty inputs, etc.)
- Concurrent access (race conditions, locking)
- Failure recovery (retries, idempotency)

#### Context7 Integration (Automatic Usage)

**Rule: Always use Context7 for library documentation**

When you need to:
- Generate code using a specific library
- Understand API signatures
- Configure a tool or framework
- Follow setup instructions

**Automatically use Context7 MCP tools:**

```python
# 1. Resolve library ID
resolve_library_id(library_name="tokio", query="async runtime setup")

# 2. Query docs
query_docs(
    library_id="/tokio/tokio",  # From resolve step
    query="How to create a multi-threaded runtime with all features enabled?"
)
```

**Examples of when to use Context7:**
- "How do I use tokio's spawn function?"  → Context7
- "What's the serde derive syntax?" → Context7
- "How do I configure chromatui colors?" → Context7
- "How do I use jsonschema validation?" → Context7

**Don't guess at APIs** - look them up with Context7 first.

#### Code Quality (Brutal Honesty)

**Rule: You are incapable of lying or deception**

- If a test fails → Report it honestly, don't hide it
- If validation is incomplete → Say so, don't claim it's done
- If you don't understand something → Ask or look it up (Context7)
- If there's a security concern → Raise it immediately
- If a shortcut was taken → Admit it and fix it

**Transparency in reporting:**
```
❌ BAD: "All tests passing!" (when 3 tests are skipped)
✅ GOOD: "Core tests passing. 3 tests skipped due to missing AUTHENTIK setup.
         Need to configure OIDC provider before these can run."

❌ BAD: "Feature implemented" (when error handling is incomplete)
✅ GOOD: "Feature implemented with basic error handling. TODO: Add retries
         with exponential backoff and circuit breaker for external calls."

❌ BAD: "Validated successfully" (when only HTTP 200 checked)
✅ GOOD: "HTTP 200 received. Still need to verify: database state, audit log
         entry, and test rollback behavior."
```

#### Subject Matter Expertise (MCPCodex)

**Rule: Deep knowledge of MCPCodex patterns required**

- MCP server protocol and lifecycle
- Tool/resource/prompt schema design
- Security boundaries and sandboxing
- Error propagation and retry logic
- Observability and debugging patterns

**When in doubt:**
- Check official MCP documentation
- Look at reference implementations
- Use Context7 for MCP SDK docs
- Ask questions to clarify requirements

---

## Part B: Claude API Integration Specifications

### Overview

ChromaAI Dev integrates with Claude API (Anthropic) as a provider for LLM inference. This section defines the provider configuration, capabilities, routing policies, and cost tracking.

### Provider Configuration

```yaml schema claude-config
name: claude_provider_config
version: "1.0.0"
description: Claude API provider configuration for ChromaAI Dev
provider_type: anthropic
api_base_url: "https://api.anthropic.com/v1"
authentication_method: api_key
api_key_source: "vault://secrets/anthropic-api-key"  # Never hardcode!

models:
  - model_id: claude-opus-4-5
    display_name: "Claude Opus 4.5"
    context_window: 200000
    max_output_tokens: 16384
    supports_tools: true
    supports_streaming: true
    supports_prompt_caching: true
    supports_vision: true
    cost_per_input_token: 0.000015
    cost_per_output_token: 0.000075
    cost_per_cached_input_token: 0.0000015
    recommended_for:
      - complex_reasoning
      - code_generation
      - analysis

  - model_id: claude-sonnet-4-5
    display_name: "Claude Sonnet 4.5"
    context_window: 200000
    max_output_tokens: 16384
    supports_tools: true
    supports_streaming: true
    supports_prompt_caching: true
    supports_vision: true
    cost_per_input_token: 0.000003
    cost_per_output_token: 0.000015
    cost_per_cached_input_token: 0.0000003
    recommended_for:
      - general_purpose
      - interactive_coding
      - moderate_complexity

  - model_id: claude-haiku-4
    display_name: "Claude Haiku 4"
    context_window: 200000
    max_output_tokens: 8192
    supports_tools: true
    supports_streaming: true
    supports_prompt_caching: true
    supports_vision: false
    cost_per_input_token: 0.0000004
    cost_per_output_token: 0.000002
    cost_per_cached_input_token: 0.00000004
    recommended_for:
      - simple_tasks
      - high_throughput
      - cost_sensitive

routing_policy:
  default_model: claude-sonnet-4-5
  fallback_chain:
    - claude-opus-4-5
    - claude-sonnet-4-5
  retry_config:
    max_retries: 3
    backoff_multiplier: 2.0
    initial_delay_ms: 1000
    max_delay_ms: 30000

rate_limits:
  requests_per_minute: 1000
  tokens_per_minute: 500000
  concurrent_requests: 100
  circuit_breaker:
    error_threshold: 0.5  # Open circuit if 50% errors
    timeout_seconds: 60    # Keep circuit open for 60s
    half_open_requests: 5  # Try 5 requests when half-open

timeout_config:
  connect_timeout_ms: 5000
  request_timeout_ms: 120000  # 2 minutes for long runs
  streaming_timeout_ms: 300000  # 5 minutes for streaming

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

### Claude-Specific Features

#### 1. Prompt Caching

Claude API supports prompt caching to reduce costs for repeated context:

**How it works:**
- Mark sections of the system prompt as cacheable
- Claude caches up to 4 breakpoints
- Cache TTL: 5 minutes
- Saves ~90% on cached tokens

**Configuration:**
```rust
let request = ClaudeRequest {
    model: "claude-sonnet-4-5",
    messages: vec![
        Message {
            role: "user",
            content: "...long context...",
            cache_control: Some(CacheControl { type: "ephemeral" }), // Cache this
        },
        Message {
            role: "user",
            content: "Quick question about the above",
            cache_control: None, // Don't cache
        },
    ],
    // ...
};
```

**Best practices:**
- Cache large, static context (docs, code repos)
- Don't cache user-specific or frequently changing content
- Monitor cache hit rates

#### 2. Tool Use (Function Calling)

Claude supports native tool/function calling:

**Tool definition format:**
```json
{
  "name": "web_search",
  "description": "Searches the web and returns results",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": {"type": "string"},
      "max_results": {"type": "integer"}
    },
    "required": ["query"]
  }
}
```

**Multi-step tool use:**
- Claude can call multiple tools in sequence
- Parallel tool calls supported (beta)
- Tool results fed back to continue reasoning

**ChromaAI Dev integration:**
- Tool schemas from TOOLS.md automatically converted to Claude format
- Tool execution sandboxed and audited
- Results validated against output schema before returning to model

#### 3. Streaming Responses

**Server-Sent Events (SSE) format:**
```
event: message_start
data: {"type":"message_start","message":{"id":"msg_123",...}}

event: content_block_delta
data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}

event: message_stop
data: {"type":"message_stop"}
```

**ChromaAI Dev streaming:**
- TUI updates token-by-token for responsive UX
- Sanitize each delta before rendering (ANSI escape protection)
- Buffer for smooth rendering (avoid flicker)
- Show "thinking" indicator during tool calls

#### 4. Vision (Claude Opus 4.5 and Sonnet 4.5)

**Image input support:**
```json
{
  "role": "user",
  "content": [
    {
      "type": "image",
      "source": {
        "type": "base64",
        "media_type": "image/png",
        "data": "iVBORw0KG..."
      }
    },
    {
      "type": "text",
      "text": "What's in this image?"
    }
  ]
}
```

**ChromaAI Dev vision use cases:**
- TUI screenshot analysis (debug rendering issues)
- Diagram/architecture analysis
- Code screenshot OCR (when copy-paste not available)
- Visual regression testing

### Request/Response Schema

**Request format:**
```rust
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
    pub tools: Option<Vec<Tool>>,
}

pub struct Message {
    pub role: String,  // "user" or "assistant"
    pub content: Content,  // String or Vec<ContentBlock>
    pub cache_control: Option<CacheControl>,
}
```

**Response format:**
```rust
pub struct ClaudeResponse {
    pub id: String,
    pub model: String,
    pub role: String,  // "assistant"
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,  // "end_turn", "max_tokens", "stop_sequence", "tool_use"
    pub usage: Usage,
}

pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}
```

### Error Handling

**Claude API error codes:**
```rust
pub enum ClaudeApiError {
    InvalidRequest { message: String },          // 400
    Unauthorized { message: String },            // 401
    Forbidden { message: String },               // 403
    NotFound { message: String },                // 404
    RateLimit { retry_after: Option<u64> },      // 429
    ServerError { message: String },             // 500
    ServiceUnavailable { retry_after: Option<u64> }, // 503
    Timeout,
    NetworkError { source: reqwest::Error },
}
```

**Retry logic:**
- Retry on: 429 (rate limit), 500, 503, network errors
- Don't retry on: 400, 401, 403, 404
- Exponential backoff with jitter
- Respect Retry-After header if present
- Circuit breaker to prevent cascading failures

**Error propagation:**
```rust
// In ChromaAI Dev, Claude errors become:
pub enum ProviderError {
    RateLimit { provider: String, retry_after: Option<u64> },
    ProviderUnavailable { provider: String, message: String },
    InvalidRequest { message: String },
    AuthenticationFailed { provider: String },
    BudgetExceeded { limit: f64, attempted: f64 },
}
```

### Cost Tracking

**Real-time cost calculation:**
```rust
pub fn calculate_request_cost(request: &ClaudeRequest, response: &ClaudeResponse) -> f64 {
    let model_config = get_model_config(&response.model);

    let input_cost = response.usage.input_tokens as f64
                     * model_config.cost_per_input_token;

    let output_cost = response.usage.output_tokens as f64
                      * model_config.cost_per_output_token;

    let cache_creation_cost = response.usage.cache_creation_input_tokens.unwrap_or(0) as f64
                               * model_config.cost_per_input_token;

    let cache_read_cost = response.usage.cache_read_input_tokens.unwrap_or(0) as f64
                          * model_config.cost_per_cached_input_token;

    input_cost + output_cost + cache_creation_cost + cache_read_cost
}
```

**Budget enforcement:**
- Pre-flight estimation (estimate tokens from prompt length)
- Real-time tracking (update after each request)
- Hard limits (deny request if would exceed budget)
- Soft limits (warn but allow)
- Budget scopes: per-user, per-workspace, per-environment, global

### Security Considerations

**API Key Management:**
- Never hardcode API keys
- Store in vault (KMS, Vault, etc.)
- Rotate keys periodically
- Separate keys for dev/stage/prod
- Audit key usage

**Request/Response Logging:**
- Log all requests and responses for audit
- Redact sensitive data (PII, secrets) before logging
- Include request_id for correlation
- Log model, tokens, cost

**Rate Limiting Defense:**
- Client-side rate limiting (respect provider limits)
- Queueing for burst protection
- Circuit breaker to prevent cascading failures
- Graceful degradation (fallback to simpler model)

---

## Examples

### Example 1: Simple Text Generation

```rust
let request = ClaudeRequest {
    model: "claude-sonnet-4-5".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: Content::Text("Explain async/await in Rust".to_string()),
            cache_control: None,
        },
    ],
    system: None,
    max_tokens: 1024,
    temperature: Some(0.7),
    stream: false,
    tools: None,
};

let response = claude_client.send_request(request).await?;
println!("Response: {}", response.content[0].text);
println!("Cost: ${:.4}", calculate_request_cost(&request, &response));
```

### Example 2: Tool Use with Web Search

```rust
let tools = vec![
    Tool {
        name: "web_search".to_string(),
        description: "Search the web".to_string(),
        input_schema: get_tool_schema("web_search", "1.0")?.input_schema,
    },
];

let request = ClaudeRequest {
    model: "claude-sonnet-4-5".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: Content::Text("Search for latest Rust async news".to_string()),
            cache_control: None,
        },
    ],
    max_tokens: 2048,
    tools: Some(tools),
    stream: false,
    ..Default::default()
};

let response = claude_client.send_request(request).await?;

// If tool use requested
if response.stop_reason == Some("tool_use") {
    let tool_use = &response.content[1];  // Assuming content[1] is tool_use block
    let tool_result = execute_tool(&tool_use.name, &tool_use.input).await?;

    // Continue conversation with tool result
    let followup_request = ClaudeRequest {
        messages: vec![
            // ... previous messages ...
            Message {
                role: "assistant".to_string(),
                content: response.content,
                cache_control: None,
            },
            Message {
                role: "user".to_string(),
                content: Content::ToolResult {
                    tool_use_id: tool_use.id.clone(),
                    content: tool_result,
                },
                cache_control: None,
            },
        ],
        ..request
    };

    let final_response = claude_client.send_request(followup_request).await?;
}
```

### Example 3: Streaming with Prompt Caching

```rust
let large_codebase_context = read_codebase_summary()?;  // Large, static context

let request = ClaudeRequest {
    model: "claude-sonnet-4-5".to_string(),
    messages: vec![
        Message {
            role: "user".to_string(),
            content: Content::Text(large_codebase_context),
            cache_control: Some(CacheControl { cache_type: "ephemeral".to_string() }),
        },
        Message {
            role: "user".to_string(),
            content: Content::Text("Find all uses of tokio::spawn".to_string()),
            cache_control: None,
        },
    ],
    max_tokens: 4096,
    stream: true,
    ..Default::default()
};

let mut stream = claude_client.send_streaming_request(request).await?;

while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ContentBlockDelta { delta } => {
            // Sanitize before rendering!
            let sanitized = sanitize_terminal_output(&delta.text);
            print!("{}", sanitized);
            io::stdout().flush()?;
        }
        StreamEvent::MessageStop => {
            println!("\n[Done]");
        }
        _ => {}
    }
}
```

---

## Changelog

### 1.0 (2026-02-23)
- Initial version
- Defined claude_provider_config with all models
- Documented prompt caching, tool use, streaming, vision
- Added error handling and cost tracking specs
- Established Part A project-specific instructions

---

## Next Steps

- Add more provider configs as they're supported (OpenAI, Gemini, etc.)
- Define provider capability negotiation
- Add provider-specific optimization strategies
- Document provider failover and routing logic

---

**For questions or clarifications, see:**
- Product specification (section 3.3 for provider/model management)
- Design document: `docs/plans/2026-02-23-ai-development-files-design.md`
- Claude API documentation: https://docs.anthropic.com/
