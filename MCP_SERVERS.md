---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft
---

# MCP_SERVERS.md - MCP Server Registry

This file defines Model Context Protocol (MCP) servers that ChromaAI Dev integrates with. Each server provides tools, resources, and prompts that extend ChromaAI Dev's capabilities.

**Purpose:** Source of truth for MCP server configurations, capabilities, security posture, and integration requirements.

---

## Server Registry

### github-mcp-server

Official GitHub MCP server providing repository operations.

```yaml schema mcp-server
name: github
version: "1.0"
description: GitHub operations via MCP (issues, PRs, code search, repos)
server_command: ["npx", "-y", "@modelcontextprotocol/server-github"]
server_type: stdio
environment_variables:
  GITHUB_TOKEN: "vault://secrets/github-token"
capabilities:
  - tools
  - resources
risk_rating: medium
allowed_environments: [dev, stage, prod]
timeout_seconds: 30
max_retries: 3
```

**Tools provided:**
- `create_or_update_file` - Create or update files in repository
- `search_repositories` - Search GitHub repositories
- `create_repository` - Create new repository
- `get_file_contents` - Read file from repository
- `push_files` - Push multiple files in single commit
- `create_issue` - Create GitHub issue
- `create_pull_request` - Create pull request
- `fork_repository` - Fork repository
- `create_branch` - Create new branch

**Resources:**
- Repository file contents
- Issue data
- Pull request information

**Security requirements:**
- `GITHUB_TOKEN` must have minimal scopes:
  - `repo:read` for read operations
  - `repo:write` for write operations (dev/stage only)
  - `issues:write` for issue creation
- All write operations audited with request_id
- Rate limiting enforced (5000 requests/hour per GitHub API limits)
- No force push or destructive operations allowed

**ACL requirements:**
- Validate token scopes before server startup
- Deny operations outside allowed repositories (if scope configured)
- Audit trail for all mutations
- Separate tokens for dev/stage/prod

**Usage example:**
```rust
let result = mcp_client.call_tool("github", "search_repositories", json!({
    "query": "language:rust stars:>1000 async"
})).await?;
```

---

### context7-mcp-server

Context7 documentation retrieval MCP server.

```yaml schema mcp-server
name: context7
version: "1.0"
description: Retrieve up-to-date library documentation and code examples
server_command: ["npx", "-y", "@context7/mcp-server"]
server_type: stdio
environment_variables: {}
capabilities:
  - tools
risk_rating: low
allowed_environments: [dev, stage, prod]
timeout_seconds: 60
max_retries: 2
```

**Tools provided:**
- `resolve-library-id` - Convert library name to Context7 ID
- `query-docs` - Query documentation with semantic search

**Use cases:**
- Retrieve current API documentation
- Find code examples and patterns
- Get setup/configuration instructions
- Resolve library version information

**Security requirements:**
- Read-only operations (no mutations)
- No authentication required (public docs)
- Rate limiting: Respect Context7 API limits
- Network egress allowed (requires external API access)

**Usage pattern:**
```rust
// 1. Resolve library
let lib_id = mcp_client.call_tool("context7", "resolve-library-id", json!({
    "libraryName": "tokio",
    "query": "async runtime for Rust"
})).await?;

// 2. Query docs
let docs = mcp_client.call_tool("context7", "query-docs", json!({
    "libraryId": lib_id,
    "query": "How to spawn async tasks?"
})).await?;
```

---

## Adding New MCP Servers

**Process:**

1. **Add schema block** to this file with all required fields
2. **Security review** if risk_rating is medium or high
3. **Test integration:**
   - Server starts successfully
   - Tools are discoverable
   - Tool calls work end-to-end
   - Error handling works
4. **Document in this file:**
   - Tools/resources/prompts provided
   - Security requirements
   - ACL requirements
   - Usage examples
5. **Update tests:** Add integration test in `tests/mcp_servers/`

**Required fields:**
- `name`: Unique server identifier
- `version`: Semver version
- `server_command`: Command to start server
- `capabilities`: List of capabilities (tools, resources, prompts)
- `risk_rating`: low, medium, high
- `allowed_environments`: Which envs can use this server

**Optional fields:**
- `environment_variables`: Required env vars
- `timeout_seconds`: Command timeout
- `max_retries`: Retry attempts on failure
- `config_file`: Path to server-specific config

---

## Server Lifecycle

**Startup:**
1. Validate environment variables exist
2. Start server process with stdio transport
3. Wait for server initialization message
4. Discover capabilities (tools/resources/prompts)
5. Validate discovered capabilities match schema
6. Ready to accept requests

**Shutdown:**
1. Send shutdown notification to server
2. Wait for graceful shutdown (max 5 seconds)
3. Kill process if not terminated
4. Clean up resources

**Health checking:**
- Periodic ping to ensure server responsive
- Restart on crash (up to max_retries)
- Circuit breaker on repeated failures
- Alert on server unavailability

---

## Security Considerations

**Sandboxing:**
- MCP servers run as separate processes
- No direct filesystem access unless explicitly granted
- Network access controlled by firewall rules
- Cannot access ChromaAI Dev's memory or state

**Secret management:**
- Environment variables from vault (never hardcoded)
- Secrets rotated periodically
- Separate secrets for dev/stage/prod
- Audit all secret access

**Audit trail:**
- Log all tool calls with request_id
- Log tool inputs (redacted if sensitive)
- Log tool outputs (redacted if sensitive)
- Log errors and retries
- Track cost per tool call (if applicable)

---

## Changelog

### 1.0 (2026-02-23)
- Initial version
- Added github-mcp-server
- Added context7-mcp-server

---

## Next Steps

- Add more MCP servers as integrations needed
- Define server capability negotiation
- Add server performance metrics
- Document server-specific optimization strategies
