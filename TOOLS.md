---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft
---

# TOOLS.md - Tool/Function Registry

This file defines tool/function schemas with JSON Schema for input/output validation, risk ratings, environment restrictions, and contract tests.

**Purpose:** Source of truth for tool definitions that ChromaAI Dev agents can use.

---

## Tool Schemas

### web_search

Low-risk tool for web search operations.

```yaml schema tool
name: web_search
version: "1.0"
description: Performs web search and returns ranked results with snippets
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
            description: Result title
          url:
            type: string
            format: uri
            description: Result URL
          snippet:
            type: string
            description: Text snippet/preview
          rank:
            type: integer
            description: Result ranking (1-indexed)
        required: [title, url, snippet, rank]
    query_time_ms:
      type: integer
      description: Query execution time in milliseconds
    total_results:
      type: integer
      description: Total number of results available (may be > max_results)
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
    description: Valid query with reasonable max_results returns success
    input:
      query: "rust async programming"
      max_results: 3
      safe_search: true
    expect_success: true
    expect_output:
      results_min_count: 0
      results_max_count: 3
      has_query_time: true

  - name: empty-query-rejected
    description: Empty query string must be rejected by schema validation
    input:
      query: ""
      max_results: 5
    expect_error: true
    error_pattern: "minLength|required"

  - name: excessive-max-results-rejected
    description: max_results exceeding maximum must be rejected
    input:
      query: "test"
      max_results: 100
    expect_error: true
    error_pattern: "maximum"

  - name: negative-max-results-rejected
    description: Negative max_results must be rejected
    input:
      query: "test"
      max_results: -1
    expect_error: true
    error_pattern: "minimum"

  - name: query-too-long-rejected
    description: Query exceeding maxLength must be rejected
    input:
      query: "x".repeat(501)
      max_results: 5
    expect_error: true
    error_pattern: "maxLength"
```

**Usage example:**
```rust
let input = json!({
    "query": "tokio async runtime",
    "max_results": 5
});

validate_tool_input("web_search", "1.0", &input)?;
let result = execute_tool("web_search", input).await?;
```

---

### execute_sql_query

High-risk tool for executing SQL queries (DEV ENVIRONMENT ONLY).

```yaml schema tool
name: execute_sql_query
version: "1.0"
description: Executes read-only SQL query against allowed databases (SELECT only, dev environment only)
risk_rating: high
allowed_environments: [dev]  # Explicitly NOT allowed in stage or prod
connector_binding: postgres_connector
timeout_seconds: 10
max_retries: 0  # No retries for high-risk operations
requires_confirmation: true  # Interactive mode must confirm before execution

input_schema:
  type: object
  properties:
    query:
      type: string
      description: SQL query (SELECT statements only - enforced by pattern)
      pattern: "^\\s*SELECT.*"
      minLength: 1
      maxLength: 5000
    database:
      type: string
      description: Target database identifier (must be in allowed list)
      enum: [analytics_dev, metrics_dev]
    limit:
      type: integer
      description: Row limit (enforced server-side)
      minimum: 1
      maximum: 1000
      default: 100
  required: [query, database]

output_schema:
  type: object
  properties:
    rows:
      type: array
      description: Query result rows as JSON objects
    row_count:
      type: integer
      description: Number of rows returned
    columns:
      type: array
      items: {type: string}
      description: Column names in result set
    execution_time_ms:
      type: integer
      description: Query execution time
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
    description: Simple SELECT query should succeed
    input:
      query: "SELECT 1 as test"
      database: analytics_dev
    expect_success: true

  - name: delete-operation-rejected
    description: DELETE statement must be rejected by pattern validation
    input:
      query: "DELETE FROM users WHERE id = 1"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT"

  - name: update-operation-rejected
    description: UPDATE statement must be rejected by pattern validation
    input:
      query: "UPDATE users SET active = false WHERE id = 1"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT"

  - name: insert-operation-rejected
    description: INSERT statement must be rejected by pattern validation
    input:
      query: "INSERT INTO users (name) VALUES ('hacker')"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT"

  - name: drop-operation-rejected
    description: DROP statement must be rejected by pattern validation
    input:
      query: "DROP TABLE users"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT"

  - name: invalid-database-rejected
    description: Database not in enum list must be rejected
    input:
      query: "SELECT * FROM logs"
      database: production_db
    expect_error: true
    error_pattern: "enum|invalid value"

  - name: excessive-limit-rejected
    description: Limit exceeding maximum must be rejected
    input:
      query: "SELECT * FROM users"
      database: analytics_dev
      limit: 10000
    expect_error: true
    error_pattern: "maximum"
```

**Security notes:**
- **Pattern validation alone is NOT sufficient** - server must also validate
- Queries should be parsed and validated server-side (prevent SQL injection)
- Use parameterized queries if variables needed (future enhancement)
- Consider query cost estimation before execution
- Log all queries with request_id for audit
- Results should NOT be persisted (retention_class: NONE)

---

### retrieve_docs

Medium-risk tool for RAG document retrieval with ACL enforcement.

```yaml schema tool
name: retrieve_docs
version: "1.0"
description: Retrieves documents from corpus with ACL enforcement and relevance scoring
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
      description: Search query for semantic/hybrid retrieval
      minLength: 1
      maxLength: 1000
    corpus_id:
      type: string
      description: Corpus identifier to search within
      minLength: 1
    top_k:
      type: integer
      description: Number of documents to retrieve
      minimum: 1
      maximum: 50
      default: 10
    filters:
      type: object
      description: Optional metadata filters (key-value pairs)
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
          doc_id:
            type: string
          content:
            type: string
          score:
            type: number
            description: Relevance score (0-1, higher is more relevant)
          source:
            type: string
            description: Source identifier (file path, URL, etc.)
          acl_groups:
            type: array
            items: {type: string}
            description: ACL groups that have access (for audit)
          corpus_version:
            type: string
            description: Version of corpus when document was indexed
        required: [doc_id, content, score]
    retrieval_time_ms:
      type: integer
      description: Retrieval operation time
    corpus_version:
      type: string
      description: Current corpus version
  required: [documents, corpus_version]

error_behavior:
  timeout: return_partial_results
  network_error: retry_with_backoff
  acl_denial: return_filtered_results

policy_tags:
  data_classification: varies  # Depends on corpus content
  retention_class: STANDARD

contract_tests:
  - name: valid-retrieval-succeeds
    description: Valid query with valid corpus_id returns results
    input:
      query: "async programming patterns"
      corpus_id: "docs-v1"
      top_k: 5
    expect_success: true
    expect_output:
      documents_max_count: 5
      has_corpus_version: true

  - name: empty-query-rejected
    description: Empty query string must be rejected
    input:
      query: ""
      corpus_id: "docs-v1"
    expect_error: true
    error_pattern: "minLength|required"

  - name: excessive-top-k-rejected
    description: top_k exceeding maximum must be rejected
    input:
      query: "test"
      corpus_id: "docs-v1"
      top_k: 100
    expect_error: true
    error_pattern: "maximum"

  - name: invalid-corpus-id-handled
    description: Invalid corpus_id should return clear error
    input:
      query: "test"
      corpus_id: "nonexistent-corpus"
    expect_error: true
    error_pattern: "corpus.*not found|invalid corpus"
```

**ACL enforcement:**
- Documents filtered based on user's ACL groups BEFORE returning
- User's ACL groups determined from session identity (actor_id)
- Documents without ACL metadata: default deny (unless policy overrides)
- Audit log records which documents were filtered out (count only, not IDs)

---

## Tool Execution Flow

```
1. User/Agent requests tool execution
   ↓
2. Client validates input against input_schema
   - If invalid → reject immediately with clear error
   ↓
3. Client checks environment allowlist
   - If tool not allowed in current env → reject
   ↓
4. Client checks risk rating + confirmation requirement
   - If high-risk + interactive → prompt for confirmation
   - If high-risk + CI/headless → reject
   ↓
5. Client sends request to server with request_id
   ↓
6. Server re-validates (defense in depth)
   - Input schema
   - Environment allowlist
   - RBAC (does actor have permission?)
   - Budget (is there sufficient budget?)
   ↓
7. Server executes tool via connector
   - Timeout enforced
   - Retries per error_behavior
   - Results captured
   ↓
8. Server validates output against output_schema
   - If invalid → log error, return sanitized error to client
   ↓
9. Server logs to audit trail
   - Tool name, version, input (redacted), output (redacted)
   - Execution time, retries, errors
   - request_id, actor_id, session_id
   ↓
10. Server returns result to client
   ↓
11. Client sanitizes output before rendering
    - ANSI escape sequences removed/filtered
    - Render to TUI
```

---

## Adding New Tools

**Process:**

1. **Define input/output schemas** using JSON Schema
2. **Set risk rating:** low (read-only, public data), medium (read with ACL), high (write, destructive, sensitive)
3. **Set allowed_environments:** Be conservative - start with `[dev]` and expand
4. **Write contract tests:** Cover happy path, error cases, edge cases
5. **Implement connector:** Server-side tool execution logic
6. **Test end-to-end:** Client → Server → Connector → Response
7. **Add to this file** with schema block
8. **Validate:** Run `cargo build` and `cargo test`
9. **Security review:** Required for medium/high risk tools

**Risk rating guidelines:**

| Risk | Criteria | Examples |
|------|----------|----------|
| Low | Read-only, public data, no side effects | web_search, format_date, parse_json |
| Medium | Read with ACL, moderate resource usage | retrieve_docs, read_file, list_files |
| High | Write operations, destructive, sensitive data access | execute_sql_query, write_file, delete_file |

**Contract test requirements:**
- At least 3 tests per tool (happy path, invalid input, edge case)
- All contract tests must pass before tool can be used
- Contract tests run on every build

---

## Tool Versioning

**When to bump version:**
- **Major (1.0 → 2.0):** Breaking changes to input/output schema
- **Minor (1.0 → 1.1):** New optional parameters, backward-compatible additions
- **Patch (1.0.0 → 1.0.1):** Bug fixes, documentation updates

**Deprecation:**
- Mark old version as deprecated
- Provide migration guide
- Support old version for 2 minor releases
- Remove only on next major version

---

## Changelog

### 1.0 (2026-02-23)
- Initial version
- Added web_search (low risk)
- Added execute_sql_query (high risk, dev only)
- Added retrieve_docs (medium risk, ACL-enforced)

---

## Next Steps

- Add more tools as use cases emerge:
  - File operations (read_file, write_file, list_files)
  - Data transformations (parse_json, format_date, encode/decode)
  - HTTP operations (http_request with configurable methods)
  - Code analysis (lint_code, run_tests)
- Define tool composition patterns (pipelines)
- Add tool cost estimation
- Add tool performance metrics
