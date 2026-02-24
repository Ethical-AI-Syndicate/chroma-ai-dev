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
version: "1.0.0"
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
    error_pattern: "minLength|required|shorter than"

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
      query: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
      max_results: 5
    expect_error: true
    error_pattern: "maxLength|longer than"
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
version: "1.0.0"
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
    error_pattern: "pattern.*failed|Only SELECT|does not match"

  - name: update-operation-rejected
    description: UPDATE statement must be rejected by pattern validation
    input:
      query: "UPDATE users SET active = false WHERE id = 1"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT|does not match"

  - name: insert-operation-rejected
    description: INSERT statement must be rejected by pattern validation
    input:
      query: "INSERT INTO users (name) VALUES ('hacker')"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT|does not match"

  - name: drop-operation-rejected
    description: DROP statement must be rejected by pattern validation
    input:
      query: "DROP TABLE users"
      database: analytics_dev
    expect_error: true
    error_pattern: "pattern.*failed|Only SELECT|does not match"

  - name: invalid-database-rejected
    description: Database not in enum list must be rejected
    input:
      query: "SELECT * FROM logs"
      database: production_db
    expect_error: true
    error_pattern: "enum|invalid value|not one of"

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
version: "1.0.0"
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
    error_pattern: "minLength|required|shorter than"

  - name: excessive-top-k-rejected
    description: top_k exceeding maximum must be rejected
    input:
      query: "test"
      corpus_id: "docs-v1"
      top_k: 100
    expect_error: true
    error_pattern: "maximum"

  - name: invalid-corpus-id-format-rejected
    description: Empty corpus_id should fail schema validation
    input:
      query: "test"
      corpus_id: ""
    expect_error: true
    error_pattern: "minLength|required|shorter than"
```

**ACL enforcement:**
- Documents filtered based on user's ACL groups BEFORE returning
- User's ACL groups determined from session identity (actor_id)
- Documents without ACL metadata: default deny (unless policy overrides)
- Audit log records which documents were filtered out (count only, not IDs)

---

### http_request

Medium-risk HTTP client for controlled API calls.

```yaml schema tool
name: http_request
version: "1.0.0"
description: Performs HTTP requests to allowlisted endpoints with method and timeout controls
risk_rating: medium
allowed_environments: [dev, stage]
connector_binding: http_connector
timeout_seconds: 20
max_retries: 2

input_schema:
  type: object
  properties:
    method:
      type: string
      enum: [GET, POST, PUT, PATCH, DELETE]
      description: HTTP method to execute
    url:
      type: string
      format: uri
      pattern: "^https://"
      description: HTTPS URL target
    headers:
      type: object
      description: Optional request headers
      additionalProperties: {type: string}
    body:
      type: object
      description: Optional JSON body for write methods
      additionalProperties: true
    timeout_ms:
      type: integer
      minimum: 100
      maximum: 30000
      default: 5000
      description: Per-request timeout override
  required: [method, url]

output_schema:
  type: object
  properties:
    status:
      type: integer
    headers:
      type: object
      additionalProperties: {type: string}
    body:
      type: object
      additionalProperties: true
    response_time_ms:
      type: integer
  required: [status, response_time_ms]

error_behavior:
  timeout: retry_with_backoff
  network_error: retry_with_backoff
  rate_limit: fail_with_message

policy_tags:
  data_classification: internal
  retention_class: SHORT

contract_tests:
  - name: valid-get-request-schema
    description: Valid GET request payload should pass validation
    input:
      method: GET
      url: "https://api.example.com/health"
    expect_success: true

  - name: non-https-url-rejected
    description: Non-HTTPS URL should fail schema validation
    input:
      method: GET
      url: "http://api.example.com/health"
    expect_error: true
    error_pattern: "pattern|https"

  - name: timeout-too-high-rejected
    description: timeout_ms above max should fail validation
    input:
      method: GET
      url: "https://api.example.com/health"
      timeout_ms: 50000
    expect_error: true
    error_pattern: "maximum"
```

---

### read_file

Medium-risk tool for reading local files within approved workspace boundaries.

```yaml schema tool
name: read_file
version: "1.0.0"
description: Reads UTF-8 text files from allowlisted paths with size and line limits
risk_rating: medium
allowed_environments: [dev]
connector_binding: filesystem_connector
timeout_seconds: 10
max_retries: 0

input_schema:
  type: object
  properties:
    path:
      type: string
      minLength: 1
      maxLength: 500
      pattern: "^(src|docs|tests|config)/"
      description: Relative file path under approved directories
    max_bytes:
      type: integer
      minimum: 1
      maximum: 200000
      default: 50000
      description: Maximum bytes to read
    offset_line:
      type: integer
      minimum: 1
      maximum: 100000
      default: 1
      description: Line number offset for partial reads
  required: [path]

output_schema:
  type: object
  properties:
    content:
      type: string
    bytes_read:
      type: integer
    truncated:
      type: boolean
  required: [content, bytes_read, truncated]

error_behavior:
  timeout: fail_immediately
  acl_denial: fail_immediately

policy_tags:
  data_classification: confidential
  retention_class: SHORT

contract_tests:
  - name: valid-source-file-path
    description: Approved source path should pass schema checks
    input:
      path: "src/main.rs"
      max_bytes: 1000
    expect_success: true

  - name: traversal-path-rejected
    description: Parent directory traversal should be rejected by pattern
    input:
      path: "../secrets.txt"
    expect_error: true
    error_pattern: "pattern|does not match"

  - name: max-bytes-too-large-rejected
    description: max_bytes over limit should fail
    input:
      path: "docs/readme.md"
      max_bytes: 999999
    expect_error: true
    error_pattern: "maximum"
```

---

### write_file

High-risk tool for controlled file writes in development only.

```yaml schema tool
name: write_file
version: "1.0.0"
description: Writes UTF-8 content to approved development paths with explicit overwrite controls
risk_rating: high
allowed_environments: [dev]
connector_binding: filesystem_connector
timeout_seconds: 10
max_retries: 0
requires_confirmation: true

input_schema:
  type: object
  properties:
    path:
      type: string
      minLength: 1
      maxLength: 500
      pattern: "^(tmp|docs|tests)/"
      description: Relative output path under approved directories
    content:
      type: string
      minLength: 1
      maxLength: 100000
      description: UTF-8 file content to write
    overwrite:
      type: boolean
      default: false
      description: Whether existing file may be overwritten
  required: [path, content]

output_schema:
  type: object
  properties:
    path:
      type: string
    bytes_written:
      type: integer
    created:
      type: boolean
  required: [path, bytes_written, created]

error_behavior:
  timeout: fail_immediately
  acl_denial: fail_immediately

policy_tags:
  data_classification: confidential
  retention_class: NONE

contract_tests:
  - name: valid-write-request-schema
    description: Write request with allowed path should pass schema validation
    input:
      path: "tmp/output.txt"
      content: "hello world"
      overwrite: false
    expect_success: true

  - name: disallowed-path-rejected
    description: Write outside allowlisted directories must fail
    input:
      path: "src/main.rs"
      content: "not allowed"
    expect_error: true
    error_pattern: "pattern|does not match"

  - name: empty-content-rejected
    description: Empty content must fail minLength validation
    input:
      path: "tmp/output.txt"
      content: ""
    expect_error: true
    error_pattern: "minLength|shorter than"
```

---

### parse_json

Low-risk utility tool for JSON parsing and shape validation.

```yaml schema tool
name: parse_json
version: "1.0.0"
description: Parses JSON strings and returns normalized object output for downstream tool usage
risk_rating: low
allowed_environments: [dev, stage, prod]
connector_binding: local_transform
timeout_seconds: 5
max_retries: 0

input_schema:
  type: object
  properties:
    text:
      type: string
      minLength: 2
      maxLength: 200000
      description: JSON text payload
    strict:
      type: boolean
      default: true
      description: Whether duplicate keys and trailing commas are rejected
  required: [text]

output_schema:
  type: object
  properties:
    parsed:
      type: object
      additionalProperties: true
    is_valid:
      type: boolean
    key_count:
      type: integer
  required: [parsed, is_valid, key_count]

error_behavior:
  timeout: fail_immediately

policy_tags:
  data_classification: internal
  retention_class: SHORT

contract_tests:
  - name: valid-json-passes
    description: Valid JSON payload should pass schema validation
    input:
      text: '{"status":"ok","count":2}'
      strict: true
    expect_success: true

  - name: too-short-json-rejected
    description: Too-short JSON text should fail minLength
    input:
      text: "{"
    expect_error: true
    error_pattern: "minLength|shorter than"

  - name: malformed-json-shape-rejected
    description: Non-object text payload that violates minLength should fail
    input:
      text: "x"
      strict: true
    expect_error: true
    error_pattern: "minLength|shorter than"
```

---

### format_date

Low-risk utility for deterministic date and time formatting.

```yaml schema tool
name: format_date
version: "1.0.0"
description: Formats timestamps into specified date-time output formats and time zones
risk_rating: low
allowed_environments: [dev, stage, prod]
connector_binding: local_transform
timeout_seconds: 5
max_retries: 0

input_schema:
  type: object
  properties:
    timestamp:
      type: string
      format: date-time
      description: ISO-8601 input timestamp
    format:
      type: string
      enum: [rfc3339, iso_date, iso_datetime, unix_seconds]
      description: Output format identifier
    timezone:
      type: string
      pattern: "^(UTC|[A-Za-z_]+/[A-Za-z_]+)$"
      default: UTC
      description: IANA timezone or UTC
  required: [timestamp, format]

output_schema:
  type: object
  properties:
    formatted:
      type: string
    timezone:
      type: string
    unix_seconds:
      type: integer
  required: [formatted, timezone]

error_behavior:
  timeout: fail_immediately

policy_tags:
  data_classification: public
  retention_class: SHORT

contract_tests:
  - name: valid-format-request
    description: Valid timestamp and format should pass schema validation
    input:
      timestamp: "2026-02-23T10:30:00Z"
      format: rfc3339
      timezone: UTC
    expect_success: true

  - name: invalid-format-enum-rejected
    description: Unsupported output format should fail enum validation
    input:
      timestamp: "2026-02-23T10:30:00Z"
      format: custom
    expect_error: true
    error_pattern: "enum|not one of"

  - name: invalid-timezone-pattern-rejected
    description: Invalid timezone identifier should fail pattern validation
    input:
      timestamp: "2026-02-23T10:30:00Z"
      format: iso_date
      timezone: "../../etc/passwd"
    expect_error: true
    error_pattern: "pattern|does not match"
```

---

### list_files

Low-risk utility for listing files in approved directories.

```yaml schema tool
name: list_files
version: "1.0.0"
description: Lists files under approved workspace-relative directories with optional extension filtering
risk_rating: low
allowed_environments: [dev, stage]
connector_binding: filesystem_connector
timeout_seconds: 10
max_retries: 0

input_schema:
  type: object
  properties:
    directory:
      type: string
      minLength: 1
      maxLength: 500
      pattern: "^(src|docs|tests|config)/?$"
      description: Relative directory path to list
    recursive:
      type: boolean
      default: false
      description: Whether to list files recursively
    extension:
      type: string
      pattern: "^\\.[A-Za-z0-9]+$"
      description: Optional extension filter (e.g. .rs)
  required: [directory]

output_schema:
  type: object
  properties:
    files:
      type: array
      items: {type: string}
    count:
      type: integer
  required: [files, count]

error_behavior:
  timeout: fail_immediately
  acl_denial: fail_immediately

policy_tags:
  data_classification: internal
  retention_class: SHORT

contract_tests:
  - name: valid-directory-listing-schema
    description: Valid directory request should pass schema validation
    input:
      directory: "src"
      recursive: true
      extension: ".rs"
    expect_success: true

  - name: disallowed-directory-rejected
    description: Directory outside allowlist should fail pattern validation
    input:
      directory: "../"
    expect_error: true
    error_pattern: "pattern|does not match"

  - name: invalid-extension-rejected
    description: Extension without dot prefix should fail validation
    input:
      directory: "docs"
      extension: "md"
    expect_error: true
    error_pattern: "pattern|does not match"
```

---

### encode_base64

Low-risk utility for deterministic Base64 encoding and decoding.

```yaml schema tool
name: encode_base64
version: "1.0.0"
description: Encodes or decodes Base64 payloads for safe transport between tools
risk_rating: low
allowed_environments: [dev, stage, prod]
connector_binding: local_transform
timeout_seconds: 5
max_retries: 0

input_schema:
  type: object
  properties:
    mode:
      type: string
      enum: [encode, decode]
      description: Operation mode
    text:
      type: string
      minLength: 1
      maxLength: 200000
      description: Input text payload
  required: [mode, text]

output_schema:
  type: object
  properties:
    mode:
      type: string
    result:
      type: string
  required: [mode, result]

error_behavior:
  timeout: fail_immediately

policy_tags:
  data_classification: internal
  retention_class: SHORT

contract_tests:
  - name: valid-encode-request
    description: Valid encode request should pass schema checks
    input:
      mode: encode
      text: "hello"
    expect_success: true

  - name: invalid-mode-rejected
    description: Unsupported mode should fail enum validation
    input:
      mode: transform
      text: "hello"
    expect_error: true
    error_pattern: "enum|not one of"

  - name: empty-text-rejected
    description: Empty input text should fail minLength validation
    input:
      mode: decode
      text: ""
    expect_error: true
    error_pattern: "minLength|shorter than"
```

---

### agent_mail_register

Low-risk coordination tool for registering an agent mailbox identity.

```yaml schema tool
name: agent_mail_register
version: "1.0.0"
description: Registers an agent identity with the coordination mailbox service
risk_rating: low
allowed_environments: [dev, stage, prod]
connector_binding: mcp_agent_mail_connector
timeout_seconds: 10
max_retries: 2

input_schema:
  type: object
  properties:
    agent_id:
      type: string
      minLength: 3
      maxLength: 80
      pattern: "^[a-z][a-z0-9_-]*$"
      description: Unique agent identifier
    display_name:
      type: string
      minLength: 1
      maxLength: 120
      description: Human-readable agent name
    capabilities:
      type: array
      items: {type: string}
      minItems: 1
      description: Capability tags for routing
  required: [agent_id, display_name, capabilities]

output_schema:
  type: object
  properties:
    registered:
      type: boolean
    mailbox_id:
      type: string
    registered_at:
      type: string
      format: date-time
  required: [registered, mailbox_id, registered_at]

error_behavior:
  network_error: retry_with_backoff
  rate_limit: fail_with_message

policy_tags:
  data_classification: internal
  retention_class: STANDARD

contract_tests:
  - name: valid-registration-payload
    description: Valid registration payload should pass schema validation
    input:
      agent_id: planner_agent
      display_name: Planner Agent
      capabilities: [planning, decomposition]
    expect_success: true

  - name: invalid-agent-id-rejected
    description: Agent identifiers with spaces should fail pattern validation
    input:
      agent_id: "planner agent"
      display_name: Planner Agent
    expect_error: true
    error_pattern: "pattern|does not match"

  - name: missing-display-name-rejected
    description: Missing display name should fail required field validation
    input:
      agent_id: planner_agent
    expect_error: true
    error_pattern: "required"
```

---

### agent_mail_send_message

Medium-risk coordination tool for inter-agent messaging.

```yaml schema tool
name: agent_mail_send_message
version: "1.0.0"
description: Sends structured coordination messages to another registered agent
risk_rating: medium
allowed_environments: [dev, stage, prod]
connector_binding: mcp_agent_mail_connector
timeout_seconds: 15
max_retries: 2

input_schema:
  type: object
  properties:
    from_agent_id:
      type: string
      minLength: 3
      maxLength: 80
      pattern: "^[a-z][a-z0-9_-]*$"
      description: Sender agent identifier
    to_agent_id:
      type: string
      minLength: 3
      maxLength: 80
      pattern: "^[a-z][a-z0-9_-]*$"
      description: Recipient agent identifier
    subject:
      type: string
      minLength: 1
      maxLength: 200
      description: Message subject line
    body:
      type: string
      minLength: 1
      maxLength: 20000
      description: Message payload
    thread_id:
      type: string
      minLength: 1
      maxLength: 120
      description: Optional thread correlation id
  required: [from_agent_id, to_agent_id, subject, body]

output_schema:
  type: object
  properties:
    message_id:
      type: string
    delivery_status:
      type: string
      enum: [queued, delivered]
    queued_at:
      type: string
      format: date-time
  required: [message_id, delivery_status, queued_at]

error_behavior:
  network_error: retry_with_backoff
  rate_limit: retry_with_backoff

policy_tags:
  data_classification: internal
  retention_class: STANDARD

contract_tests:
  - name: valid-send-message-request
    description: Valid send message request should pass schema validation
    input:
      from_agent_id: planner_agent
      to_agent_id: implementer_agent
      subject: "Task assignment"
      body: "Please implement task 1 from plan"
      thread_id: thread-001
    expect_success: true

  - name: empty-body-rejected
    description: Empty message body must fail minLength validation
    input:
      from_agent_id: planner_agent
      to_agent_id: implementer_agent
      subject: "Task assignment"
      body: ""
    expect_error: true
    error_pattern: "minLength|shorter than"

  - name: invalid-recipient-pattern-rejected
    description: Recipient id with uppercase letters should fail pattern checks
    input:
      from_agent_id: planner_agent
      to_agent_id: ImplementerAgent
      subject: "Task assignment"
      body: "hello"
    expect_error: true
    error_pattern: "pattern|does not match"
```

---

### agent_mail_check_inbox

Low-risk coordination tool for reading queued agent messages.

```yaml schema tool
name: agent_mail_check_inbox
version: "1.0.0"
description: Retrieves inbox messages for an agent with optional unread filtering
risk_rating: low
allowed_environments: [dev, stage, prod]
connector_binding: mcp_agent_mail_connector
timeout_seconds: 10
max_retries: 2

input_schema:
  type: object
  properties:
    agent_id:
      type: string
      minLength: 3
      maxLength: 80
      pattern: "^[a-z][a-z0-9_-]*$"
      description: Agent mailbox to retrieve
    status:
      type: string
      enum: [unread, all]
      default: unread
      description: Message status filter
    max_messages:
      type: integer
      minimum: 1
      maximum: 100
      default: 20
      description: Maximum messages to return
  required: [agent_id]

output_schema:
  type: object
  properties:
    messages:
      type: array
      items:
        type: object
        properties:
          message_id: {type: string}
          from_agent_id: {type: string}
          subject: {type: string}
          body: {type: string}
          created_at:
            type: string
            format: date-time
        required: [message_id, from_agent_id, subject, body, created_at]
    unread_count:
      type: integer
  required: [messages, unread_count]

error_behavior:
  network_error: retry_with_backoff
  rate_limit: retry_with_backoff

policy_tags:
  data_classification: internal
  retention_class: SHORT

contract_tests:
  - name: valid-inbox-read-request
    description: Valid inbox read request should pass schema validation
    input:
      agent_id: implementer_agent
      status: unread
      max_messages: 25
    expect_success: true

  - name: excessive-max-messages-rejected
    description: max_messages above limit should fail maximum validation
    input:
      agent_id: implementer_agent
      max_messages: 1000
    expect_error: true
    error_pattern: "maximum"

  - name: invalid-status-rejected
    description: Unsupported status values should fail enum validation
    input:
      agent_id: implementer_agent
      status: pending
    expect_error: true
    error_pattern: "enum|not one of"
```

---

### agent_mail_reserve_file

Medium-risk coordination tool for temporary file ownership reservations.

```yaml schema tool
name: agent_mail_reserve_file
version: "1.0.0"
description: Creates a time-bounded file reservation to prevent write collisions
risk_rating: medium
allowed_environments: [dev, stage]
connector_binding: mcp_agent_mail_connector
timeout_seconds: 10
max_retries: 1

input_schema:
  type: object
  properties:
    agent_id:
      type: string
      minLength: 3
      maxLength: 80
      pattern: "^[a-z][a-z0-9_-]*$"
      description: Agent creating the reservation
    path:
      type: string
      minLength: 1
      maxLength: 500
      pattern: "^(src|docs|tests|config)/"
      description: Workspace-relative file path to reserve
    lease_seconds:
      type: integer
      minimum: 60
      maximum: 7200
      description: Reservation TTL in seconds
    mode:
      type: string
      enum: [read, write, exclusive]
      default: read
      description: Lease mode - read allows others to read, write allows modification, exclusive blocks all access
  required: [agent_id, path, lease_seconds]

output_schema:
  type: object
  properties:
    lease_id:
      type: string
    path:
      type: string
    expires_at:
      type: string
      format: date-time
  required: [lease_id, path, expires_at]

error_behavior:
  acl_denial: fail_immediately
  network_error: retry_with_backoff

policy_tags:
  data_classification: confidential
  retention_class: SHORT

contract_tests:
  - name: valid-file-reservation-request
    description: Valid file reservation payload should pass schema validation
    input:
      agent_id: implementer_agent
      path: "src/generated/tools.rs"
      lease_seconds: 300
    expect_success: true

  - name: traversal-reservation-path-rejected
    description: Traversal path should fail reservation path pattern validation
    input:
      agent_id: implementer_agent
      path: "../secrets.txt"
      lease_seconds: 300
    expect_error: true
    error_pattern: "pattern|does not match"

  - name: lease-too-short-rejected
    description: Lease values below minimum should fail schema validation
    input:
      agent_id: implementer_agent
      path: "docs/plan.md"
      lease_seconds: 10
    expect_error: true
    error_pattern: "minimum"
```

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
