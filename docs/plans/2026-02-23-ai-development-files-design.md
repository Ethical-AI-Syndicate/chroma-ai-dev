# AI Development Files Design

**Date:** 2026-02-23
**Status:** Approved
**Purpose:** Design for source-of-truth markdown files that serve as both AI assistant instructions and schema definitions for ChromaAI Dev

---

## Overview

This design establishes a markdown-first, schema-embedded documentation system that serves multiple purposes:

1. **Instructions for AI assistants** working on the ChromaAI Dev codebase
2. **Source of truth schemas** for tools, prompts, evaluations, MCP servers, and agent configurations
3. **Code generation input** for type-safe Rust implementations
4. **Runtime validation** rules with multi-version support

The system follows the principle of "documentation as code" - schemas are embedded in markdown for human readability while being machine-extractable for validation and code generation.

---

## Design Principles

### Markdown-First
- All documentation and schemas in markdown format
- Terminal-friendly (renders well with `bat`, `mdcat`, standard viewers)
- Git-friendly diffs
- Single source of truth - docs and schemas together

### Schema-Embedded
- Schemas embedded as fenced YAML code blocks with `schema` tag
- Parser extracts tagged blocks for validation and codegen
- Version metadata in frontmatter
- Schemas remain contextual (rationale and usage near definitions)

### Multi-Layer Validation
- **Build-time:** `build.rs` extracts, validates, generates code
- **CI-time:** Ensures schemas valid and generated code in sync
- **Runtime:** Validates operations against loaded schemas

### Dual Purpose
Each file has two parts:
- **Part A:** Instructions for AI assistants developing ChromaAI Dev
- **Part B:** Runtime specifications and schemas that ChromaAI Dev uses

---

## File Structure

```
chroma-ai-dev/
├── .claude/
│   └── settings.local.json          # Local Claude Code settings
├── docs/
│   ├── plans/                        # Design documents
│   │   └── 2026-02-23-ai-development-files-design.md
│   └── schemas/                      # Meta-schemas for validation
│       ├── tool-schema.json          # JSON Schema for tool definitions
│       ├── prompt-schema.json        # JSON Schema for prompt templates
│       ├── eval-schema.json          # JSON Schema for eval definitions
│       ├── mcp-server-schema.json    # JSON Schema for MCP server configs
│       ├── agent-config-schema.json  # JSON Schema for agent configs
│       └── claude-config-schema.json # JSON Schema for Claude configs
├── AGENTS.md                         # AI assistant behavior + agent runtime specs
├── CLAUDE.md                         # Project-specific Claude instructions + Claude API integration
├── MCP_SERVERS.md                    # MCP server registry + server schemas
├── PROMPTS.md                        # Prompt template library + prompt schemas
├── EVALS.md                          # Evaluation suite definitions + eval schemas
├── TOOLS.md                          # Tool/function registry + tool schemas
├── build.rs                          # Build script for schema extraction & codegen
└── src/
    ├── generated/                    # Generated Rust code (git-ignored initially, committed after stable)
    │   ├── tools.rs
    │   ├── prompts.rs
    │   ├── evals.rs
    │   ├── agents.rs
    │   └── mcp_servers.rs
    └── ...
```

---

## Frontmatter Convention

All markdown files include YAML frontmatter:

```yaml
---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft | stable | deprecated
---
```

**Fields:**
- `schema_version`: Version of the schema format itself (for meta-schema evolution)
- `last_updated`: ISO 8601 date of last modification
- `validated_by`: Who/what validated the schemas (build_system, manual, ci)
- `status`: Lifecycle stage of the document

---

## Schema Block Tagging Convention

Extractable schema blocks use fenced code blocks with language tag:

````markdown
```yaml schema <type>
name: example
version: "1.0"
# ... schema content
```
````

**Schema types:**
- `tool` - Tool/function definitions
- `prompt` - Prompt templates
- `eval` - Evaluation suite definitions
- `mcp-server` - MCP server configurations
- `agent-config` - Agent runtime configurations
- `claude-config` - Claude API integration settings

**Parser behavior:**
- Extracts all blocks matching pattern ` ```yaml schema <type> `
- Preserves source file path and line number for error reporting
- Validates each block against corresponding meta-schema
- Generates Rust code from validated schemas

---

## File Specifications

### AGENTS.md

**Purpose:** Define AI assistant behavior when working on this codebase AND specify agent runtime capabilities that ChromaAI Dev provides to users.

**Structure:**

```markdown
---
schema_version: "1.0"
last_updated: "2026-02-23"
status: draft
---

# AGENTS.md - Agent Specifications

## Part A: Instructions for AI Assistants Working on ChromaAI Dev

### Architecture Constraints
- Rust async/await patterns and error handling
- Terminal safety requirements (ANSI sanitization mandatory)
- No secrets on disk (use OS keychain or server vault)
- Audit-first design (all actions must be traceable)
- Thin-client principle (server-authoritative for policy/artifacts)

### Testing Requirements
- All policy enforcement points must have unit tests
- Terminal escape injection tests mandatory for output rendering
- Contract tests required for all tool schemas
- Integration tests for SSO/RBAC flows
- Golden eval suite for regression protection

### Validation Requirements
- Irrefutable proof required (matching global CLAUDE.md rules)
- All schemas validated at build + runtime + CI
- No assumptions about SSO/RBAC - validate identity binding
- Production URLs for testing (no localhost)

### Code Quality Standards
- No shortcuts or workarounds (especially for AUTHENTIK integration)
- Complete error handling (no unwrap() in production code)
- Comprehensive logging with structured fields
- Security-first mindset (OWASP top 10 awareness)

---

## Part B: Agent Runtime Specifications (ChromaAI Dev Features)

### Agent Loop Controls

```yaml schema agent-config
name: agent_loop_defaults
version: "1.0"
max_steps: 10
max_tool_calls: 20
max_wall_time_seconds: 300
max_cost_dollars: 1.00
require_confirmation_for_high_risk: true
policy_tags:
  data_classification: internal
```

### Supported Agent Patterns
- **Single-shot:** One prompt, one response (simplest)
- **Multi-turn conversations:** Stateful dialogue with context
- **Tool-using agents:** Agents with function calling and loop controls
- **RAG-enhanced agents:** Agents with retrieval capabilities and ACL enforcement

### Agent Safety Controls
- Hard stop on policy violation (no override)
- Cost tracking per request with budget enforcement
- Tool allowlist/denylist per environment
- High-risk tool confirmation in interactive mode (forbidden in CI)
- Circuit breakers for provider brownouts

[... more agent runtime specs ...]
```

---

### CLAUDE.md

**Purpose:** Project-specific instructions that extend/override global ~/.claude/CLAUDE.md AND specify Claude API integration details.

**Structure:**

```markdown
---
schema_version: "1.0"
last_updated: "2026-02-23"
status: draft
---

# CLAUDE.md - Claude-Specific Configuration

## Part A: Project-Specific Instructions (Extends ~/.claude/CLAUDE.md)

### ChromaAI Dev Specific Rules

**AUTHENTIK Integration (Critical)**
- AUTHENTIK SSO must be setup properly - no shortcuts or workarounds
- Test with real OIDC device flow, not mocked responses
- Validate token refresh and expiration handling
- All identity binding must be server-verified

**Testing Standards**
- STOP TESTING WITH LOCALHOST - this is a production system
- Always test with production URLs and real endpoints
- HTTP status codes are NOT sufficient proof - validate actual behavior
- Visual validation required for TUI components

**Architecture Validation**
- Thin-client principle: verify server authority on every action
- Identity binding: every action must have actor_id + session_id + device_id
- Immutability: artifacts are version-locked after publish (no in-place updates)
- Policy enforcement: must be server-authoritative, not client-side checks

**Validation Requirements**
- Don't make ANY assumptions - validate everything
- Provide irrefutable proof that features are working
- Complete and total test coverage including visual validation
- Context7 should be used automatically for library docs and setup steps

**Code Quality**
- Brutal honesty and transparency (no deception)
- Subject matter expertise on all things MCPCodex
- No lying about test results or validation status

---

## Part B: Claude API Integration Specifications

```yaml schema claude-config
name: claude_provider_config
version: "1.0"
provider_type: anthropic
api_base_url: "https://api.anthropic.com/v1"
models:
  - model_id: claude-opus-4-5
    display_name: "Claude Opus 4.5"
    context_window: 200000
    max_output_tokens: 16384
    supports_tools: true
    supports_streaming: true
    supports_prompt_caching: true
    cost_per_input_token: 0.000015
    cost_per_output_token: 0.000075

  - model_id: claude-sonnet-4-5
    display_name: "Claude Sonnet 4.5"
    context_window: 200000
    max_output_tokens: 16384
    supports_tools: true
    supports_streaming: true
    supports_prompt_caching: true
    cost_per_input_token: 0.000003
    cost_per_output_token: 0.000015

routing_policy:
  default_model: claude-sonnet-4-5
  fallback_chain:
    - claude-opus-4-5
  retry_config:
    max_retries: 3
    backoff_multiplier: 2
    initial_delay_ms: 1000

rate_limits:
  requests_per_minute: 1000
  tokens_per_minute: 500000
  concurrent_requests: 100

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

### Claude-Specific Features
- Prompt caching for repeated context
- Tool use with parallel function calling
- Streaming responses with token-by-token updates
- Vision capabilities for image analysis (future)

[... more Claude integration details ...]
```

---

### MCP_SERVERS.md

**Purpose:** Registry of MCP servers, their capabilities, security posture, and integration schemas.

**Structure:**

```markdown
---
schema_version: "1.0"
last_updated: "2026-02-23"
status: draft
---

# MCP_SERVERS.md - MCP Server Registry

## Server Registry

### github-mcp-server

```yaml schema mcp-server
name: github
version: "1.0"
server_command: ["npx", "-y", "@modelcontextprotocol/server-github"]
environment_variables:
  GITHUB_TOKEN: "vault://secrets/github-token"
capabilities:
  - tools
  - resources
risk_rating: medium
allowed_environments: [dev, stage, prod]
timeout_seconds: 30
```

**Capabilities:**
- Tools: create_pull_request, search_code, list_issues, add_issue_comment
- Resources: Repository contents, issue data

**Security Requirements:**
- GITHUB_TOKEN must have minimal scopes (repo:read, issues:write)
- All write operations audited
- Rate limiting enforced (5000 req/hour)

**ACL Requirements:**
- Validate token scopes before server startup
- Deny operations outside allowed repos
- Audit trail for all mutations

---

### context7-mcp-server

```yaml schema mcp-server
name: context7
version: "1.0"
server_command: ["npx", "-y", "@context7/mcp-server"]
capabilities:
  - tools
risk_rating: low
allowed_environments: [dev, stage, prod]
timeout_seconds: 60
```

**Capabilities:**
- Tools: resolve-library-id, query-docs

**Use Cases:**
- Retrieve up-to-date library documentation
- Code generation with current API references
- Setup and configuration guidance

[... more MCP servers ...]
```

---

### PROMPTS.md

**Purpose:** Library of prompt templates with typed variables, policy tags, and versioning.

**Structure:**

```markdown
---
schema_version: "1.0"
last_updated: "2026-02-23"
status: draft
---

# PROMPTS.md - Prompt Template Library

## System Prompts

### default-assistant

```yaml schema prompt
id: default-assistant
version: "1.0"
type: system
description: Default system prompt for ChromaAI Dev assistant
template: |
  You are an AI assistant integrated into ChromaAI Dev, an enterprise-grade
  terminal-first AI development and evaluation tool.

  You have access to tools and must follow strict policy constraints.
  All your actions are audited and tied to user identity.

  Current context:
  - User: {{user_id}}
  - Session: {{session_id}}
  - Environment: {{environment}}
  - Allowed tools: {{allowed_tools}}
  - Budget remaining: ${{budget_remaining}}

  Follow the principle of least privilege and always validate inputs.

variables:
  user_id:
    type: string
    required: true
    description: Authenticated user identifier
  session_id:
    type: string
    required: true
    description: Session identifier for audit trail
  environment:
    type: string
    enum: [dev, stage, prod]
    required: true
    description: Current environment context
  allowed_tools:
    type: array
    items: {type: string}
    required: true
    description: List of tools available in this session
  budget_remaining:
    type: number
    required: true
    description: Remaining budget in dollars

policy_tags:
  data_classification: internal
  retention_class: SHORT

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

## User Prompts

### rag-query

```yaml schema prompt
id: rag-query
version: "1.0"
type: user
description: Query with retrieved context from RAG system
template: |
  Query: {{query}}

  Retrieved context ({{corpus_version}}):
  {{#each retrieved_docs}}
  ---
  Document {{@index}} (score: {{this.score}}):
  {{this.content}}

  Source: {{this.source}}
  ACL: {{this.acl_groups}}
  {{/each}}

  Instructions:
  - Answer the query using ONLY the provided context
  - If the context doesn't contain sufficient information, say so explicitly
  - Cite document numbers when referencing specific information
  - Do not hallucinate or use external knowledge

variables:
  query:
    type: string
    required: true
    description: User's question or query
  corpus_version:
    type: string
    required: true
    description: Version of the corpus used for retrieval
  retrieved_docs:
    type: array
    required: true
    description: Array of retrieved documents with metadata
    items:
      type: object
      properties:
        content: {type: string}
        source: {type: string}
        score: {type: number}
        acl_groups: {type: array}

policy_tags:
  data_classification: varies
  retention_class: STANDARD

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

[... more prompt templates ...]
```

---

### EVALS.md

**Purpose:** Evaluation suite definitions with test cases, grading methods, and regression gates.

**Structure:**

```markdown
---
schema_version: "1.0"
last_updated: "2026-02-23"
status: draft
---

# EVALS.md - Evaluation Suite Definitions

## Evaluation Suites

### policy-enforcement-suite

```yaml schema eval
suite_id: policy-enforcement-suite
version: "1.0"
description: Validates that policy enforcement blocks unauthorized actions
severity: critical
cases:
  - case_id: deny-unprivileged-promote
    description: Developer cannot promote to prod without approval
    input:
      actor_role: Developer
      action: promote_to_prod
      release_id: "rel-123"
    expected_outcome:
      type: policy_denial
      reason_code: INSUFFICIENT_PRIVILEGES
      http_status: 403
    grading_method: deterministic

  - case_id: deny-missing-approvals
    description: ReleaseManager cannot promote without required approvals
    input:
      actor_role: ReleaseManager
      action: promote_to_prod
      release_id: "rel-123"
      approvals: []
    expected_outcome:
      type: policy_denial
      reason_code: MISSING_APPROVALS
    grading_method: deterministic

  - case_id: allow-privileged-promote
    description: ReleaseManager can promote with approvals
    input:
      actor_role: ReleaseManager
      action: promote_to_prod
      release_id: "rel-123"
      approvals: ["approver1", "approver2"]
    expected_outcome:
      type: success
      http_status: 200
    grading_method: deterministic

thresholds:
  pass_rate: 1.0  # 100% must pass
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### output-quality-suite

```yaml schema eval
suite_id: output-quality-suite
version: "1.0"
description: Validates AI output quality using LLM-as-judge
severity: high

judge_config:
  model: claude-sonnet-4-5
  temperature: 0.0
  repeat_trials: 3
  variance_tolerance: 0.1
  max_tokens: 1000

cases:
  - case_id: summarization-accuracy
    description: Summary contains key points and respects length limit
    input:
      prompt_id: summarize-docs
      variables:
        docs: |
          [Long document with requirement A, requirement B, and implementation details...]
    expected_constraints:
      - type: contains_key_points
        key_points:
          - "requirement A"
          - "requirement B"
      - type: length_limit
        max_words: 200
      - type: no_hallucination
        check_against_source: true

    grading_method: llm_judge
    judge_prompt: |
      Evaluate the summary against these criteria:
      1. Does it contain all key points: requirement A, requirement B?
      2. Is it under 200 words?
      3. Does it avoid adding information not in the source?

      Respond with:
      PASS - if all criteria met
      FAIL - if any criteria not met

      Then provide a brief explanation.

thresholds:
  pass_rate: 0.95  # 95% must pass
  max_failures: 1

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### terminal-safety-suite

```yaml schema eval
suite_id: terminal-safety-suite
version: "1.0"
description: Validates terminal escape sequence sanitization
severity: critical

cases:
  - case_id: block-osc52-clipboard
    description: OSC 52 clipboard escape sequences must be sanitized
    input:
      model_output: "Here is your data: \u001b]52;c;base64data\u001b\\ (malicious)"
    expected_outcome:
      type: sanitized
      contains_escape_sequences: false
    grading_method: deterministic

  - case_id: block-cursor-movement
    description: Cursor movement sequences must be sanitized
    input:
      model_output: "Output\u001b[A\u001b[2K(overwritten)"
    expected_outcome:
      type: sanitized
      contains_escape_sequences: false
    grading_method: deterministic

  - case_id: allow-safe-ansi-colors
    description: Safe ANSI color codes should be preserved
    input:
      model_output: "\u001b[32mGreen text\u001b[0m"
    expected_outcome:
      type: preserved
      contains_safe_sequences: true
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

[... more eval suites ...]
```

---

### TOOLS.md

**Purpose:** Tool/function registry with JSON schemas, risk ratings, and contract tests.

**Structure:**

```markdown
---
schema_version: "1.0"
last_updated: "2026-02-23"
status: draft
---

# TOOLS.md - Tool/Function Registry

## Tool Schemas

### web_search

```yaml schema tool
name: web_search
version: "1.0"
description: Performs web search and returns ranked results
risk_rating: low
allowed_environments: [dev, stage, prod]
connector_binding: http_connector
timeout_seconds: 30
max_retries: 2

input_schema:
  type: object
  properties:
    query:
      type: string
      description: Search query string
      minLength: 1
      maxLength: 500
    max_results:
      type: integer
      description: Maximum number of results to return
      minimum: 1
      maximum: 10
      default: 5
    safe_search:
      type: boolean
      description: Enable safe search filtering
      default: true
  required: [query]

output_schema:
  type: object
  properties:
    results:
      type: array
      items:
        type: object
        properties:
          title:
            type: string
          url:
            type: string
            format: uri
          snippet:
            type: string
          rank:
            type: integer
        required: [title, url, snippet, rank]
    query_time_ms:
      type: integer
      description: Query execution time in milliseconds
    total_results:
      type: integer
      description: Total number of results available
  required: [results, query_time_ms]

error_behavior:
  timeout: return_empty_results
  network_error: retry_with_backoff
  rate_limit: fail_with_message

policy_tags:
  data_classification: public
  retention_class: SHORT

contract_tests:
  - name: valid-query-returns-results
    input:
      query: "rust async programming"
      max_results: 3
    expect_success: true
    expect_output:
      results_min_count: 0
      results_max_count: 3

  - name: empty-query-fails
    input:
      query: ""
    expect_error: true
    error_pattern: "query.*required|minLength"

  - name: excessive-results-capped
    input:
      query: "test"
      max_results: 100
    expect_error: true
    error_pattern: "maximum"
```

---

### execute_sql_query

```yaml schema tool
name: execute_sql_query
version: "1.0"
description: Executes read-only SQL query against allowed databases (DEV ONLY)
risk_rating: high
allowed_environments: [dev]  # Explicitly not allowed in stage/prod
connector_binding: postgres_connector
timeout_seconds: 10
max_retries: 0
requires_confirmation: true  # Interactive mode only

input_schema:
  type: object
  properties:
    query:
      type: string
      description: SQL query (SELECT statements only)
      pattern: "^\\s*SELECT.*"
      maxLength: 5000
    database:
      type: string
      description: Target database identifier
      enum: [analytics_dev, metrics_dev]
    limit:
      type: integer
      description: Row limit (enforced)
      minimum: 1
      maximum: 1000
      default: 100
  required: [query, database]

output_schema:
  type: object
  properties:
    rows:
      type: array
      description: Query result rows
    row_count:
      type: integer
      description: Number of rows returned
    columns:
      type: array
      items: {type: string}
      description: Column names
    execution_time_ms:
      type: integer
  required: [rows, row_count, columns]

error_behavior:
  timeout: fail_immediately
  network_error: fail_immediately
  rate_limit: fail_with_message

policy_tags:
  data_classification: confidential
  retention_class: NONE  # Do not persist query results

contract_tests:
  - name: select-query-succeeds
    input:
      query: "SELECT 1 as test"
      database: analytics_dev
    expect_success: true

  - name: write-operation-rejected
    input:
      query: "DELETE FROM users WHERE id = 1"
      database: analytics_dev
    expect_error: true
    error_pattern: "Only SELECT queries allowed|pattern.*failed"

  - name: update-operation-rejected
    input:
      query: "UPDATE users SET active = false"
      database: analytics_dev
    expect_error: true
    error_pattern: "Only SELECT queries allowed|pattern.*failed"

  - name: drop-operation-rejected
    input:
      query: "DROP TABLE users"
      database: analytics_dev
    expect_error: true
    error_pattern: "Only SELECT queries allowed|pattern.*failed"
```

---

### retrieve_docs

```yaml schema tool
name: retrieve_docs
version: "1.0"
description: Retrieves documents from corpus with ACL enforcement
risk_rating: medium
allowed_environments: [dev, stage, prod]
connector_binding: chroma_retrieval_service
timeout_seconds: 5
max_retries: 2

input_schema:
  type: object
  properties:
    query:
      type: string
      description: Search query
      minLength: 1
      maxLength: 1000
    corpus_id:
      type: string
      description: Corpus identifier
    top_k:
      type: integer
      description: Number of documents to retrieve
      minimum: 1
      maximum: 50
      default: 10
    filters:
      type: object
      description: Optional metadata filters
      additionalProperties: true
  required: [query, corpus_id]

output_schema:
  type: object
  properties:
    documents:
      type: array
      items:
        type: object
        properties:
          doc_id: {type: string}
          content: {type: string}
          score: {type: number}
          source: {type: string}
          acl_groups: {type: array, items: {type: string}}
          corpus_version: {type: string}
        required: [doc_id, content, score]
    retrieval_time_ms:
      type: integer
    corpus_version:
      type: string
  required: [documents, corpus_version]

error_behavior:
  timeout: return_partial_results
  network_error: retry_with_backoff
  acl_denial: return_filtered_results

policy_tags:
  data_classification: varies
  retention_class: STANDARD

contract_tests:
  - name: valid-retrieval-succeeds
    input:
      query: "test query"
      corpus_id: "test-corpus"
      top_k: 5
    expect_success: true
    expect_output:
      documents_max_count: 5
```

[... more tool schemas ...]
```

---

## Schema Extraction and Validation Pipeline

### Build Pipeline Architecture

```
┌─────────────────────────────────┐
│ Markdown Files                  │
│ (TOOLS.md, PROMPTS.md, etc.)    │
└───────────────┬─────────────────┘
                │
                ▼
┌─────────────────────────────────┐
│ build.rs                        │
│ ┌─────────────────────────────┐ │
│ │ 1. Parse markdown           │ │
│ │ 2. Extract ```yaml schema   │ │
│ │ 3. Validate against meta    │ │
│ │ 4. Check cross-references   │ │
│ │ 5. Generate Rust code       │ │
│ └─────────────────────────────┘ │
└───────────────┬─────────────────┘
                │
                ├─────────────────────┬──────────────────────┐
                ▼                     ▼                      ▼
┌───────────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│ Validation            │  │ Code Generation  │  │ Error Reporting  │
│ - JSON Schema check   │  │ - Rust structs   │  │ - File:line ref  │
│ - Integrity checks    │  │ - Validators     │  │ - Clear messages │
│ - Uniqueness checks   │  │ - Registries     │  │ - Suggestion     │
│ - Version format      │  │ - Builders       │  │                  │
└───────────────────────┘  └────────┬─────────┘  └──────────────────┘
                                    │
                                    ▼
                          ┌──────────────────────┐
                          │ src/generated/       │
                          │ - tools.rs           │
                          │ - prompts.rs         │
                          │ - evals.rs           │
                          │ - agents.rs          │
                          │ - mcp_servers.rs     │
                          └──────────────────────┘
```

### build.rs Implementation

**Dependencies:**
```toml
[build-dependencies]
pulldown-cmark = "0.9"      # Markdown parsing
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"           # YAML deserialization
serde_json = "1.0"           # JSON for meta-schemas
jsonschema = "0.17"          # Schema validation
quote = "1.0"                # Code generation
syn = "2.0"                  # Rust AST
semver = "1.0"               # Version parsing
```

**Key functions:**

```rust
// Pseudo-code for build.rs structure

fn main() {
    println!("cargo:rerun-if-changed=TOOLS.md");
    println!("cargo:rerun-if-changed=PROMPTS.md");
    println!("cargo:rerun-if-changed=EVALS.md");
    println!("cargo:rerun-if-changed=AGENTS.md");
    println!("cargo:rerun-if-changed=MCP_SERVERS.md");
    println!("cargo:rerun-if-changed=CLAUDE.md");

    // Extract and validate
    let tools = extract_and_validate("TOOLS.md", "tool", "docs/schemas/tool-schema.json");
    let prompts = extract_and_validate("PROMPTS.md", "prompt", "docs/schemas/prompt-schema.json");
    let evals = extract_and_validate("EVALS.md", "eval", "docs/schemas/eval-schema.json");
    let agents = extract_and_validate("AGENTS.md", "agent-config", "docs/schemas/agent-config-schema.json");
    let mcp_servers = extract_and_validate("MCP_SERVERS.md", "mcp-server", "docs/schemas/mcp-server-schema.json");
    let claude_configs = extract_and_validate("CLAUDE.md", "claude-config", "docs/schemas/claude-config-schema.json");

    // Cross-reference validation
    validate_cross_references(&tools, &prompts, &evals);

    // Code generation
    generate_code(&tools, "src/generated/tools.rs");
    generate_code(&prompts, "src/generated/prompts.rs");
    generate_code(&evals, "src/generated/evals.rs");
    generate_code(&agents, "src/generated/agents.rs");
    generate_code(&mcp_servers, "src/generated/mcp_servers.rs");
}

fn extract_and_validate(
    markdown_path: &str,
    schema_type: &str,
    meta_schema_path: &str
) -> Vec<SchemaBlock> {
    let content = fs::read_to_string(markdown_path).unwrap();
    let schemas = extract_schemas(&content, schema_type, markdown_path);

    // Load meta-schema
    let meta_schema = fs::read_to_string(meta_schema_path).unwrap();
    let meta_schema: serde_json::Value = serde_json::from_str(&meta_schema).unwrap();
    let validator = jsonschema::JSONSchema::compile(&meta_schema).unwrap();

    // Validate each schema
    for schema in &schemas {
        let schema_value: serde_json::Value =
            serde_yaml::from_str(&schema.content).unwrap();

        if let Err(errors) = validator.validate(&schema_value) {
            panic!(
                "Schema validation failed in {}:{}\n{:?}",
                schema.source_file,
                schema.line_number,
                errors
            );
        }

        // Validate version format
        let version = schema_value.get("version").unwrap().as_str().unwrap();
        semver::Version::parse(version).expect("Invalid semver");
    }

    // Check ID uniqueness
    check_unique_ids(&schemas);

    schemas
}

fn extract_schemas(
    markdown: &str,
    schema_type: &str,
    source_file: &str
) -> Vec<SchemaBlock> {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut schemas = vec![];
    let mut line_number = 1;

    for event in parser {
        match event {
            Event::Code(lang, code) if lang.starts_with(&format!("yaml schema {}", schema_type)) => {
                schemas.push(SchemaBlock {
                    schema_type: schema_type.to_string(),
                    content: code.to_string(),
                    source_file: source_file.to_string(),
                    line_number,
                });
            }
            Event::Text(text) => {
                line_number += text.matches('\n').count();
            }
            _ => {}
        }
    }

    schemas
}

fn generate_code(schemas: &[SchemaBlock], output_path: &str) {
    // Generate Rust structs with serde derives
    // Generate const registry: pub const TOOLS: &[ToolSchema] = &[...];
    // Generate validation functions
    // Generate builders for type-safe construction

    let generated_code = quote! {
        // Generated code here
    };

    fs::write(output_path, generated_code.to_string()).unwrap();
}
```

### Runtime Validation

**At startup:**
```rust
pub fn validate_all_schemas() -> Result<(), SchemaError> {
    // Called in main() or during initialization
    // Re-validates that generated code matches current state
    // Ensures no drift between docs and runtime

    // This is a sanity check, actual validation happens at build time
    Ok(())
}
```

**At request time:**
```rust
// When user invokes a tool
let input_json = /* from user request */;
validate_tool_input("web_search", "1.0", &input_json)?;
execute_tool("web_search", input_json)?;
```

### CI/CD Pipeline

**GitHub Actions workflow:**
```yaml
name: Schema Validation

on: [push, pull_request]

jobs:
  validate-schemas:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build (runs build.rs)
        run: cargo build

      - name: Run schema validation tests
        run: cargo test schema_validation

      - name: Check for generated code drift
        run: |
          git diff --exit-code src/generated/
          if [ $? -ne 0 ]; then
            echo "Generated code is out of sync!"
            echo "Run 'cargo build' locally and commit the changes."
            exit 1
          fi
```

---

## Schema Versioning and Change Management

### Version Format

**Semantic versioning (semver):** `major.minor.patch`

**Version bumping rules:**
- **Major:** Breaking changes (removed fields, changed types, new required fields, removed tools)
- **Minor:** Backward-compatible additions (new optional fields, new tools, new enum values)
- **Patch:** Documentation updates, clarifications, bug fixes to schemas (no functional changes)

### Schema Block Versioning

Every schema block includes explicit version:

```yaml schema tool
name: web_search
version: "1.2.0"  # Current version
deprecated_versions: ["1.0.0", "1.1.0"]  # Old versions
migration_guide: "docs/migrations/web_search-1.2.md"
```

### Change Workflow

**Adding a new tool:**
1. Add schema block to TOOLS.md with version "1.0.0"
2. Run `cargo build` (build.rs validates and generates code)
3. Write contract tests in `tests/tools/contract_tests.rs`
4. Update EVALS.md if tool needs evaluation coverage
5. Commit all changes together (markdown + generated code + tests)

**Modifying existing schema (breaking change):**
1. Increment major version (e.g., "1.2.0" → "2.0.0")
2. Add current version to `deprecated_versions`
3. Create migration guide in `docs/migrations/`
4. Update all dependent schemas (prompts using the tool, evals testing it)
5. Run `cargo build && cargo test`
6. Commit with detailed message explaining breaking changes

**Modifying existing schema (non-breaking):**
1. Increment minor or patch version
2. Run `cargo build && cargo test`
3. Commit changes

### Multi-Version Support

**Generated code includes all supported versions:**

```rust
pub enum ToolVersion {
    V1_0_0,
    V1_1_0,
    V1_2_0,  // current
}

pub fn validate_tool_input(
    name: &str,
    version: ToolVersion,
    input: &serde_json::Value
) -> Result<(), ValidationError> {
    match (name, version) {
        ("web_search", ToolVersion::V1_0_0) => {
            log::warn!("Tool web_search v1.0.0 is deprecated. Migrate to v1.2.0");
            validate_web_search_v1_0_0(input)
        }
        ("web_search", ToolVersion::V1_2_0) => {
            validate_web_search_v1_2_0(input)
        }
        _ => Err(ValidationError::UnknownVersion),
    }
}
```

### Deprecation Policy

**Timeline:**
- Mark as deprecated with migration guide
- Support for 2 minor versions after deprecation (e.g., if deprecated at 2.0, support until 2.2)
- Remove deprecated versions only on next major bump

**Deprecation notice:**
```yaml schema tool
name: old_tool
version: "1.0.0"
deprecated: true
deprecated_at: "2026-02-01"
removed_at: "2026-05-01"
replacement: "new_tool"
migration_guide: "docs/migrations/old_tool-to-new_tool.md"
```

### Git Hooks

**pre-commit hook:**
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running schema validation..."
cargo build || {
    echo "Schema validation failed!"
    exit 1
}

cargo test schema_validation || {
    echo "Schema tests failed!"
    exit 1
}

# Check generated code is in sync
if ! git diff --quiet src/generated/; then
    echo "Generated code is out of sync!"
    echo "Changes detected in src/generated/ that aren't staged."
    echo "Did you forget to stage the generated files?"
    exit 1
fi

echo "Schema validation passed!"
```

**commit-msg hook:**
```bash
#!/bin/bash
# .git/hooks/commit-msg

# Enforce conventional commits for schema changes
commit_msg=$(cat "$1")

if echo "$commit_msg" | grep -qE "TOOLS.md|PROMPTS.md|EVALS.md|AGENTS.md|MCP_SERVERS.md|CLAUDE.md"; then
    if ! echo "$commit_msg" | grep -qE "^(feat|fix|breaking|docs)\(tools|prompts|evals|agents|mcp|claude\):"; then
        echo "Schema changes require conventional commit format:"
        echo "  feat(tools): add new tool"
        echo "  breaking(prompts): remove deprecated variable"
        echo "  fix(evals): correct threshold"
        exit 1
    fi
fi
```

---

## Testing Strategy

### Schema Validation Tests

**Location:** `tests/schema_validation.rs`

```rust
#[test]
fn all_schemas_parse_and_validate() {
    // Ensures all schema blocks are valid YAML
    // Ensures all blocks pass meta-schema validation
    assert!(validate_all_schemas().is_ok());
}

#[test]
fn no_duplicate_ids() {
    // Ensures tool IDs, prompt IDs, eval IDs are unique
    let tools = extract_tool_schemas();
    let ids: HashSet<_> = tools.iter().map(|t| &t.id).collect();
    assert_eq!(ids.len(), tools.len(), "Duplicate tool IDs found");

    let prompts = extract_prompt_schemas();
    let ids: HashSet<_> = prompts.iter().map(|p| &p.id).collect();
    assert_eq!(ids.len(), prompts.len(), "Duplicate prompt IDs found");
}

#[test]
fn cross_references_resolve() {
    // Ensures prompts reference valid models
    // Ensures evals reference valid prompts/tools
    for prompt in extract_prompt_schemas() {
        for model in &prompt.allowed_models {
            assert!(
                is_valid_model(model),
                "Prompt {} references unknown model {}",
                prompt.id,
                model
            );
        }
    }

    for eval in extract_eval_schemas() {
        for case in &eval.cases {
            if let Some(prompt_id) = &case.input.prompt_id {
                assert!(
                    prompt_exists(prompt_id),
                    "Eval {} references unknown prompt {}",
                    eval.suite_id,
                    prompt_id
                );
            }
        }
    }
}

#[test]
fn version_format_valid() {
    // Ensures all versions are valid semver
    for tool in extract_tool_schemas() {
        assert!(
            semver::Version::parse(&tool.version).is_ok(),
            "Tool {} has invalid version {}",
            tool.name,
            tool.version
        );
    }
}

#[test]
fn policy_tags_valid() {
    // Ensures all policy tags use valid values
    let valid_classifications = ["public", "internal", "confidential", "restricted"];
    let valid_retention = ["NONE", "SHORT", "STANDARD", "LEGAL_HOLD"];

    for tool in extract_tool_schemas() {
        if let Some(classification) = &tool.policy_tags.data_classification {
            assert!(
                valid_classifications.contains(&classification.as_str()),
                "Tool {} has invalid data_classification: {}",
                tool.name,
                classification
            );
        }
    }
}
```

### Contract Tests

**Location:** `tests/tools/contract_tests.rs`

```rust
#[test]
fn tool_web_search_contract() {
    let schema = get_tool_schema("web_search", "1.2.0");

    // Test valid input
    let valid_input = json!({
        "query": "rust async programming",
        "max_results": 5,
        "safe_search": true
    });
    assert!(validate_tool_input("web_search", &valid_input).is_ok());

    // Test invalid input - empty query
    let invalid_input = json!({
        "query": "",
        "max_results": 5
    });
    let result = validate_tool_input("web_search", &invalid_input);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("minLength"));

    // Test invalid input - excessive max_results
    let invalid_input = json!({
        "query": "test",
        "max_results": 100
    });
    let result = validate_tool_input("web_search", &invalid_input);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("maximum"));

    // Test contract tests defined in schema
    for contract_test in &schema.contract_tests {
        run_contract_test(contract_test);
    }
}

#[test]
fn tool_execute_sql_query_contract() {
    let schema = get_tool_schema("execute_sql_query", "1.0.0");

    // Valid SELECT query
    let valid_input = json!({
        "query": "SELECT * FROM users LIMIT 10",
        "database": "analytics_dev",
        "limit": 10
    });
    assert!(validate_tool_input("execute_sql_query", &valid_input).is_ok());

    // Invalid - write operation
    let invalid_input = json!({
        "query": "DELETE FROM users WHERE id = 1",
        "database": "analytics_dev"
    });
    let result = validate_tool_input("execute_sql_query", &invalid_input);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("pattern"));

    // Contract tests from schema
    for contract_test in &schema.contract_tests {
        run_contract_test(contract_test);
    }
}
```

### Round-Trip Tests

**Location:** `tests/roundtrip.rs`

```rust
#[test]
fn markdown_to_generated_code_roundtrip() {
    // Parse TOOLS.md
    let parsed_tools = extract_tool_schemas_from_markdown("TOOLS.md");

    // Compare with generated const
    let generated_tools = crate::generated::tools::TOOLS;

    assert_eq!(
        parsed_tools.len(),
        generated_tools.len(),
        "Generated code has different number of tools than markdown"
    );

    for (parsed, generated) in parsed_tools.iter().zip(generated_tools.iter()) {
        assert_eq!(parsed.name, generated.name);
        assert_eq!(parsed.version, generated.version);
        // ... compare all fields
    }
}

#[test]
fn schema_stability_on_rebuild() {
    // Build twice and ensure generated code is identical
    // Ensures deterministic code generation

    let gen1 = build_and_read_generated("tools");
    let gen2 = build_and_read_generated("tools");

    assert_eq!(gen1, gen2, "Generated code is non-deterministic");
}
```

### Security Tests

**Location:** `tests/security/terminal_escape.rs`

```rust
#[test]
fn blocks_osc52_clipboard_injection() {
    let malicious_output = "Data: \x1b]52;c;bWFsaWNpb3VzX2RhdGE=\x1b\\ (stolen)";
    let sanitized = sanitize_terminal_output(malicious_output);

    assert!(!sanitized.contains("\x1b]52"));
    assert!(!sanitized.contains("bWFsaWNpb3VzX2RhdGE"));
}

#[test]
fn blocks_cursor_movement() {
    let malicious_output = "Visible\x1b[A\x1b[2KHidden override";
    let sanitized = sanitize_terminal_output(malicious_output);

    assert!(!sanitized.contains("\x1b[A"));
    assert!(!sanitized.contains("\x1b[2K"));
}

#[test]
fn preserves_safe_ansi_colors() {
    let colored_output = "\x1b[32mGreen\x1b[0m Normal \x1b[1;31mBold Red\x1b[0m";
    let sanitized = sanitize_terminal_output(colored_output);

    // Safe color codes should be preserved
    assert!(sanitized.contains("\x1b[32m"));
    assert!(sanitized.contains("\x1b[0m"));
}

#[test]
fn terminal_safety_eval_suite_passes() {
    // Run the terminal-safety-suite from EVALS.md
    let suite = get_eval_suite("terminal-safety-suite");
    let results = run_eval_suite(&suite);

    assert_eq!(results.pass_rate, 1.0, "Terminal safety suite must have 100% pass rate");
}
```

---

## Documentation Governance

### Review Requirements

**Schema change approval levels:**

| Change Type | Approvers Required | Additional Requirements |
|-------------|-------------------|------------------------|
| New tool (low risk) | 1 platform engineer | Contract tests |
| New tool (medium/high risk) | 1 platform + 1 security | Security review, contract tests |
| Tool schema breaking change | 2 platform engineers | Migration guide, deprecation plan |
| New prompt template | 1 AI team member | Example usage |
| Prompt breaking change | 1 AI + 1 product | Migration guide, eval coverage |
| New eval suite | 1 QA + 1 AI team | Threshold justification |
| Eval threshold change | 2 QA engineers | Impact analysis |
| Policy/agent config change | 1 architect + 1 security | Security review |
| MCP server addition | 1 platform + 1 security | Risk assessment |

### Change Checklist Template

**For pull requests modifying schemas:**

```markdown
## Schema Change Checklist

### Metadata
- [ ] Change type: [ ] Breaking [ ] Non-breaking [ ] Documentation only
- [ ] Affected files: [ ] TOOLS.md [ ] PROMPTS.md [ ] EVALS.md [ ] AGENTS.md [ ] MCP_SERVERS.md [ ] CLAUDE.md

### Version Management
- [ ] Version number updated (following semver)
- [ ] Migration guide written (if breaking change) in `docs/migrations/`
- [ ] Deprecated versions list updated (if applicable)

### Validation
- [ ] `cargo build` passes (build.rs validation)
- [ ] `cargo test schema_validation` passes
- [ ] Generated code committed to `src/generated/`
- [ ] No git diff in `src/generated/` after build

### Testing
- [ ] Contract tests added/updated in `tests/tools/contract_tests.rs`
- [ ] Security tests added (if high-risk tool or sensitive operation)
- [ ] Dependent schemas updated (prompts using tool, evals testing it)
- [ ] All existing tests pass

### Documentation
- [ ] Part A (AI instructions) updated if development process changes
- [ ] Part B (schemas) includes clear descriptions
- [ ] Cross-references valid (prompts reference valid models, evals reference valid tools)
- [ ] Examples provided for complex schemas

### Review Requirements
- [ ] Appropriate reviewers assigned (see governance table)
- [ ] Security review completed (if required)
- [ ] Architecture review completed (if major structural change)

### Post-Merge
- [ ] Update related documentation (README, architecture docs)
- [ ] Notify affected teams (if breaking change)
- [ ] Update any external integrations
```

### Ownership Matrix

| File | Primary Owner | Review Required From | Security Review |
|------|--------------|---------------------|-----------------|
| AGENTS.md | Engineering Lead | AI Team Lead | If risk rating changes |
| CLAUDE.md | Engineering Lead | - | If auth/token handling changes |
| TOOLS.md | Platform Team | Security (for high-risk tools) | All new medium/high risk tools |
| PROMPTS.md | AI Team | Product (for user-facing) | If PII/confidential data handling |
| EVALS.md | QA Team | AI Team | If security test coverage |
| MCP_SERVERS.md | Platform Team | Security | All new servers |

---

## Initial Bootstrap Content

### Phase 0: Initial Commit

**What gets created:**

1. **All 6 markdown files** with:
   - Complete frontmatter (schema_version: "1.0", status: "draft")
   - Comprehensive Part A instructions for AI assistants
   - Minimal Part B schemas (1-2 examples per file as templates)
   - Inline documentation explaining structure and conventions
   - Comments showing where to add more schemas

2. **Meta-schemas** in `docs/schemas/`:
   - `tool-schema.json` - JSON Schema for tool definitions
   - `prompt-schema.json` - JSON Schema for prompt templates
   - `eval-schema.json` - JSON Schema for eval suites
   - `mcp-server-schema.json` - JSON Schema for MCP server configs
   - `agent-config-schema.json` - JSON Schema for agent configs
   - `claude-config-schema.json` - JSON Schema for Claude configs

3. **build.rs** implementation:
   - Markdown parser
   - Schema extractor
   - Meta-schema validator
   - Code generator (basic structure)
   - Error reporter with file:line references

4. **CI configuration** (`.github/workflows/schema-validation.yml`):
   - Schema validation job
   - Generated code drift check
   - Test execution

5. **Git hooks** (templates in `.githooks/`):
   - `pre-commit` - schema validation
   - `commit-msg` - conventional commit enforcement

6. **Test scaffolding**:
   - `tests/schema_validation.rs` - basic validation tests
   - `tests/tools/contract_tests.rs` - example contract test
   - `tests/roundtrip.rs` - roundtrip test template

### Example Bootstrap Schemas

**TOOLS.md initial content:**
- `web_search` (low risk, straightforward example)
- `execute_sql_query` (high risk, shows security constraints)

**PROMPTS.md initial content:**
- `default-assistant` (system prompt example)
- `rag-query` (user prompt with templating)

**EVALS.md initial content:**
- `policy-enforcement-suite` (deterministic grading)
- `terminal-safety-suite` (security-critical tests)

**AGENTS.md initial content:**
- `agent_loop_defaults` (basic agent configuration)

**MCP_SERVERS.md initial content:**
- `github` (common server example)
- `context7` (documentation retrieval)

**CLAUDE.md initial content:**
- `claude_provider_config` (provider configuration)

### Incremental Expansion Strategy

**After bootstrap:**
1. Add real tool schemas as features are implemented
2. Add prompt templates as use cases emerge
3. Add eval suites as quality gates are defined
4. Keep Part A instructions stable (update only for process changes)
5. Evolve Part B schemas organically with product development

**Priority for expansion:**
1. Core tools needed for Phase 0 (from spec section 14)
2. OIDC/auth prompts and configs
3. Policy enforcement eval suites
4. Budget control configurations
5. RAG retrieval tools and prompts (Phase 4)

---

## Integration with ChromaAI Dev Architecture

### Client-Side Usage

**Generated code provides compile-time safety:**

```rust
use crate::generated::tools::{TOOLS, validate_tool_input};
use crate::generated::prompts::{PROMPTS, render_prompt};

// Get tool schema
let tool = TOOLS.iter()
    .find(|t| t.name == "web_search" && t.version == "1.2.0")
    .expect("Tool not found");

// Validate input before sending to server
let user_input = json!({
    "query": "rust async",
    "max_results": 5
});

validate_tool_input("web_search", &user_input)?;

// Render prompt with variables
let variables = json!({
    "user_id": "alice@example.com",
    "environment": "prod",
    "allowed_tools": ["web_search", "retrieve_docs"]
});

let rendered = render_prompt("default-assistant", &variables)?;
```

**Runtime validation before API calls:**

```rust
// Client validates for UX (fast feedback)
// Server validates for security (authoritative)

pub async fn execute_run(prompt: &str, tools: &[String]) -> Result<RunResult> {
    // Validate tools are allowed in current environment
    for tool_name in tools {
        let tool = get_tool_schema(tool_name)?;

        if !tool.allowed_environments.contains(&current_env()) {
            return Err(Error::ToolNotAllowedInEnv {
                tool: tool_name.clone(),
                env: current_env(),
            });
        }
    }

    // Send to server (server will re-validate)
    control_plane_client.create_run(prompt, tools).await
}
```

### Server-Side Usage

**Defense in depth - independent validation:**

```rust
// Server independently validates using same schemas
// Never trust client validation

pub async fn handle_create_run(
    actor: &Identity,
    request: CreateRunRequest
) -> Result<RunResponse> {
    // Policy check
    enforce_policy(actor, "run:create", &request.workspace)?;

    // Validate prompt references valid template
    if let Some(prompt_id) = &request.prompt_id {
        get_prompt_schema(prompt_id)?;
    }

    // Validate tools
    for tool_name in &request.tools {
        let tool = get_tool_schema(tool_name)?;

        // Check environment allowlist
        if !tool.allowed_environments.contains(&request.environment) {
            return Err(Error::PolicyDenial {
                reason: format!("Tool {} not allowed in {}", tool_name, request.environment)
            });
        }

        // Check risk rating vs actor permissions
        if tool.risk_rating == RiskRating::High && !actor.has_role("ToolAdmin") {
            return Err(Error::InsufficientPrivileges);
        }
    }

    // Execute run...
}
```

**Version negotiation:**

```rust
// Server can reject requests with schema versions it doesn't support

pub fn validate_request_version(
    tool_name: &str,
    requested_version: &str
) -> Result<()> {
    let tool = get_tool_schema(tool_name)?;

    if tool.deprecated_versions.contains(&requested_version.to_string()) {
        warn!(
            "Client using deprecated tool version: {}@{}",
            tool_name, requested_version
        );
    }

    if !is_supported_version(tool_name, requested_version) {
        return Err(Error::UnsupportedVersion {
            tool: tool_name.to_string(),
            requested: requested_version.to_string(),
            supported: get_supported_versions(tool_name),
        });
    }

    Ok(())
}
```

---

## Success Criteria

This design is successful when:

1. **Single source of truth:** All schemas live in markdown files, no duplication
2. **Build-time validation:** Invalid schemas cannot be committed (pre-commit hook)
3. **Type safety:** Generated Rust code provides compile-time guarantees
4. **Runtime validation:** All operations validated against schemas before execution
5. **Version compatibility:** Multi-version support allows gradual migration
6. **Clear ownership:** Each file has defined owners and review requirements
7. **Comprehensive testing:** Contract tests, security tests, and roundtrip tests all pass
8. **Documentation quality:** Part A instructions are clear and actionable for AI assistants
9. **Schema evolution:** Breaking changes have migration guides and deprecation timelines
10. **CI enforcement:** Generated code drift detected and blocked in CI

---

## Open Questions

These decisions need to be made during implementation:

1. **Generated code in git:** Should `src/generated/` be git-ignored or committed?
   - **Recommendation:** Commit initially for transparency, consider git-ignore later if stable

2. **Meta-schema evolution:** How do we version the meta-schemas themselves?
   - **Recommendation:** Use `schema_version` in frontmatter, bump when meta-schema changes

3. **Cross-file references:** Should we support references across files (e.g., EVALS.md referencing TOOLS.md schemas)?
   - **Recommendation:** Yes, validate at build time with clear error messages

4. **Schema registry service:** Should there be a runtime service that serves schemas via API?
   - **Recommendation:** Future enhancement, not needed for Phase 0

5. **Auto-generated documentation:** Should we generate HTML docs from markdown schemas?
   - **Recommendation:** Nice-to-have, use tools like mdbook or docusaurus later

---

## Next Steps

1. **Approve this design document**
2. **Invoke writing-plans skill** to create detailed implementation plan
3. **Implement Phase 0:**
   - Create all 6 markdown files with bootstrap content
   - Implement build.rs (parser, validator, basic codegen)
   - Create meta-schemas in docs/schemas/
   - Set up CI validation
   - Write initial tests
4. **Iterate:**
   - Add real schemas as features are implemented
   - Refine Part A instructions based on AI assistant usage
   - Expand eval suites as quality requirements emerge

---

**Document Status:** Approved
**Next Action:** Invoke writing-plans skill for implementation plan
