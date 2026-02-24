---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft
---

# AGENTS.md - Agent Specifications

This file serves dual purposes:
1. **Part A:** Instructions for AI assistants working on the ChromaAI Dev codebase
2. **Part B:** Agent runtime specifications that ChromaAI Dev provides to end users

---

## Part A: Instructions for AI Assistants Working on ChromaAI Dev

### Project Context

ChromaAI Dev is an **enterprise-grade terminal-first AI development, evaluation, and release tool** built with Rust + chromatui. This is not a toy project or prototype - it must meet strict enterprise requirements for:

- Security (SSO/RBAC, audit trails, no secrets on disk)
- Reliability (thin-client architecture, server-authoritative policy)
- Compliance (data retention, regional constraints, immutable artifacts)
- Observability (structured logging, forensic reconstruction, traceability)

### Architecture Constraints

**Thin-Client Principle**
- Client MUST NOT make authoritative decisions about:
  - Policy enforcement (server decides)
  - Artifact publishing (server validates and stores)
  - Budget enforcement (server tracks and blocks)
  - Audit logging (server is append-only source of truth)
- Client validates for UX (fast feedback), server validates for security (authoritative)

**Rust Async/Await Patterns**
- Use `tokio` for async runtime
- All I/O operations must be async
- Proper error handling with `anyhow::Result` or `thiserror`
- No `unwrap()` or `expect()` in production code paths
- Use `?` operator for error propagation

**Terminal Safety (CRITICAL)**
- All model/tool output MUST be sanitized before rendering to terminal
- Strip malicious ANSI escape sequences:
  - OSC 52 (clipboard manipulation)
  - Cursor movement sequences that could overwrite output
  - Screen clearing sequences
- Preserve safe ANSI color codes (basic 16 colors)
- See `terminal-safety-suite` in EVALS.md for requirements
- Failure to sanitize = security vulnerability

**No Secrets on Disk**
- Never store long-lived secrets in local files
- Use OS keychain integration (keyring crate) where available
- Server-side secrets via vault integration (KMS/Vault)
- Encrypt local drafts/caches with user-specific key
- Log redaction for sensitive data

**Audit-First Design**
- Every action must be traceable to:
  - `actor_id` (who)
  - `session_id` (which session)
  - `device_id` (from where)
  - `request_id` (unique identifier for forensics)
- Structured logging with stable field names (JSON)
- Redaction rules applied before persistence
- See observability requirements in product spec

### Testing Requirements

**Policy Enforcement Tests (MANDATORY)**
- Every policy enforcement point MUST have tests
- Test both allow and deny cases
- Validate decision IDs are logged
- Examples in `policy-enforcement-suite` (EVALS.md)
- 100% pass rate required (regression blocking)

**Terminal Escape Injection Tests (MANDATORY)**
- Test suite: `terminal-safety-suite` in EVALS.md
- All test cases MUST pass (regression blocking)
- Add tests for any new output rendering code
- Verify both blocking malicious sequences AND preserving safe ones

**Contract Tests for Tool Schemas**
- Every tool schema MUST have contract tests
- Test valid inputs (expect success)
- Test invalid inputs (expect specific errors)
- Test boundary conditions (max lengths, min values, etc.)
- Run contract tests defined in TOOLS.md

**Integration Tests for SSO/RBAC**
- OIDC device authorization flow (headless terminal)
- Token refresh and expiration handling
- Role-based access control (all roles from spec)
- Break-glass sessions with time-bounds
- Identity binding on every request

**Security Tests**
- OWASP Top 10 awareness
- SQL injection (if any DB queries)
- Command injection (if any shell execution)
- Path traversal (if any file operations)
- XSS/terminal escape injection
- Secret leakage in logs/errors

### Validation Requirements

These requirements align with the global CLAUDE.md instructions and are NON-NEGOTIABLE:

**No Assumptions - Validate Everything**
- Don't assume SSO is configured correctly - test it
- Don't assume policy enforcement works - prove it
- Don't assume secrets are redacted - verify it
- Don't assume budget limits work - demonstrate them

**Irrefutable Proof Required**
- HTTP status codes are NOT sufficient proof
- Must verify actual behavior and side effects
- Visual validation for TUI components (screenshots if possible)
- Complete test coverage including edge cases
- Demonstrate with production URLs, not localhost

**AUTHENTIK Integration (CRITICAL)**
- AUTHENTIK SSO must be setup properly - no shortcuts or workarounds
- Test with real OIDC device flow, not mocked responses
- Validate token refresh works correctly
- Validate token expiration handling
- All identity binding must be server-verified
- Break-glass must work and be audited

**Production URLs Only**
- STOP TESTING WITH LOCALHOST - this is a production system
- Always test with production URLs and real endpoints
- Local dev environment must simulate production architecture
- No "it works on my machine" - prove it works in production config

**Context7 Integration**
- Always use Context7 MCP tools for:
  - Library documentation (e.g., tokio, serde, chromatui)
  - Setup and configuration steps
  - API references for dependencies
- Don't guess at API usage - look it up with Context7
- Resolve library ID first, then query docs

### Code Quality Standards

**No Shortcuts or Workarounds**
- Especially for AUTHENTIK integration - must be production-ready
- No TODOs that should be done now
- No commented-out code "for later"
- No "good enough for now" - it's good enough or it's not done

**Complete Error Handling**
- No `unwrap()` or `panic!()` in production code
- Use `Result<T, E>` for fallible operations
- Use `thiserror` for custom error types
- Provide helpful error messages with context
- Log errors with structured fields

**Comprehensive Logging**
- Structured JSON logs with stable field names
- Include request_id, actor_id, session_id in all log entries
- Log policy decisions with decision_id
- Redact sensitive data before logging
- Use log levels appropriately (trace, debug, info, warn, error)

**Security-First Mindset**
- Assume all input is malicious until validated
- Validate at system boundaries (user input, external APIs, file reads)
- Trust internal code and framework guarantees
- Don't add unnecessary validation for internal functions
- Defense in depth: client validates for UX, server validates for security

**Brutal Honesty and Transparency**
- You are incapable of lying or deception
- If a test fails, report it - don't hide it
- If validation is incomplete, say so - don't claim it's done
- If you don't know something, look it up (Context7) or ask
- Provide evidence for all claims

**Subject Matter Expertise**
- Deep knowledge of enterprise security patterns
- Understanding of audit and compliance requirements
- Familiarity with OWASP Top 10
- Knowledge of terminal escape sequences and sanitization
- Rust async patterns and best practices

### Development Workflow

**Before Writing Code**
1. Read relevant sections of product spec (provided at start)
2. Understand architecture constraints (thin-client, server-authoritative)
3. Check TOOLS.md, PROMPTS.md, EVALS.md for existing schemas
4. Plan approach with security and audit in mind

**While Writing Code**
1. Use Context7 for library documentation
2. Write tests FIRST for critical paths (TDD where appropriate)
3. Implement with security and error handling
4. Add structured logging
5. Sanitize outputs before rendering

**After Writing Code**
1. Run `cargo build` (validates schemas)
2. Run `cargo test` (all tests must pass)
3. Run `cargo clippy` (no warnings allowed)
4. Run `cargo fmt` (code must be formatted)
5. Manual testing with production-like config
6. Visual validation for TUI changes
7. Update schemas in markdown files if needed

### Common Pitfalls to Avoid

**"It's just a prototype"**
- Wrong. This is production-grade code from day one.
- No shortcuts, no workarounds, no "we'll fix it later"

**"Tests can wait"**
- Wrong. Tests for critical paths (policy, security, terminal safety) are mandatory.
- Write tests as you implement features.

**"Localhost testing is fine"**
- Wrong. Test with production URLs and real configurations.
- Local dev must mirror production architecture.

**"HTTP 200 means it worked"**
- Wrong. Verify actual behavior and side effects.
- Check database state, audit logs, filesystem, etc.

**"I'll add error handling later"**
- Wrong. Error handling is part of the implementation.
- No unwrap() in production code paths.

**"Users won't do that"**
- Wrong. Assume all input is malicious.
- Validate at boundaries, sanitize outputs.

---

## Part B: Agent Runtime Specifications (ChromaAI Dev Features)

### Overview

ChromaAI Dev provides controlled agent execution environments with strict policy enforcement, budget controls, and audit trails. This section defines the runtime specifications for agents that run within ChromaAI Dev.

### Agent Loop Controls

Default configuration for agent loops (can be overridden per workspace/environment):

```yaml schema agent-config
name: agent_loop_defaults
version: "1.0.0"
description: Default agent loop control parameters
max_steps: 10
max_tool_calls: 20
max_wall_time_seconds: 300
max_cost_dollars: 1.00
require_confirmation_for_high_risk: true
policy_tags:
  data_classification: internal
  retention_class: SHORT
```

**Field descriptions:**

- `max_steps`: Maximum number of reasoning steps (turns) in agent loop
- `max_tool_calls`: Maximum total tool invocations across all steps
- `max_wall_time_seconds`: Hard timeout for entire agent execution
- `max_cost_dollars`: Maximum spend (LLM + tools) for this execution
- `require_confirmation_for_high_risk`: If true, prompt user before calling high-risk tools (interactive mode only)

**Enforcement:**
- Exceeded limits trigger hard stop (not graceful degradation)
- Audit log records: actual steps, tool calls, wall time, cost
- If stopped by limit, run status = "terminated" with reason code

### Agent Patterns Supported

**1. Single-Shot (Simplest)**
- One prompt input → one response output
- No tool calls, no iteration
- Lowest latency and cost
- Use case: Simple transformations, formatting, summarization

**2. Multi-Turn Conversation**
- Stateful dialogue with preserved context
- No tool calls (or optional)
- Session-scoped state management
- Use case: Interactive coding assistant, Q&A

**3. Tool-Using Agent with Loop Controls**
- Agentic behavior with function calling
- Iterates until task complete or limits reached
- Loop controls enforced (see agent_loop_defaults)
- Use case: Code search, file operations, complex queries

**4. RAG-Enhanced Agent**
- Retrieval step before generation
- ACL enforcement on retrieved documents
- Corpus version binding for reproducibility
- Use case: Documentation Q&A, knowledge base queries

### Agent Safety Controls

**Policy Enforcement (Server-Authoritative)**
- Every agent execution checked against workspace policy
- Allowlist/denylist for models, tools, corpora
- RBAC: roles determine what actions are permitted
- ABAC: attributes (env, data classification, etc.) affect decisions
- Policy decision logged with decision_id

**Budget Enforcement (Real-Time)**
- Pre-flight cost estimation before execution
- Hard limits: per-request, per-user, per-workspace, per-env
- Circuit breakers for provider errors/rate limits
- Budget exceeded = request denied (not queued)

**Tool Safety**
- Risk ratings: low, medium, high
- High-risk tools:
  - Require elevated permissions (role checks)
  - Require confirmation in interactive mode
  - Forbidden in CI/automated runs
  - Extra audit logging
- Tool allowlists per environment (dev vs stage vs prod)
- Timeout and retry policies per tool

**Output Safety (Terminal Escape Prevention)**
- All model and tool outputs sanitized before rendering
- ANSI escape sequence filtering (see terminal-safety-suite in EVALS.md)
- Protects against:
  - Clipboard injection (OSC 52)
  - Cursor movement attacks
  - Screen manipulation
- Safe color codes preserved for UX

**Audit Trail (Immutable)**
- Every agent execution logged with:
  - request_id (unique)
  - actor_id, session_id, device_id
  - prompt (or reference), variables (or reference)
  - model, parameters (temperature, etc.)
  - tool calls: inputs, outputs, errors
  - policy decisions: decision_id, allow/deny, reason
  - cost breakdown: tokens, dollars
  - timing: start, end, duration per step
- Redaction applied before persistence (PII, secrets)
- Retention class determines storage duration

### Agent Configuration Schema

The following schema defines agent-specific configuration that can be set per workspace or environment:

```yaml schema agent-config
name: mode_single_shot_defaults
version: "1.0.0"
description: Default limits for single-shot mode with no iterative tool loop
max_steps: 1
max_tool_calls: 2
max_wall_time_seconds: 45
max_cost_dollars: 0.05
require_confirmation_for_high_risk: true
allowed_models:
  - claude-sonnet-4-5
allowed_tools:
  - web_search
  - retrieve_docs
forbidden_tools:
  - execute_sql_query
  - write_file
policy_tags:
  data_classification: internal
  retention_class: SHORT
```

```yaml schema agent-config
name: mode_multi_turn_defaults
version: "1.0.0"
description: Default limits for conversational multi-turn mode with bounded iteration
max_steps: 12
max_tool_calls: 24
max_wall_time_seconds: 420
max_cost_dollars: 1.50
session_timeout_seconds: 1800
require_confirmation_for_high_risk: true
allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
allowed_tools:
  - web_search
  - retrieve_docs
  - read_file
forbidden_tools:
  - execute_sql_query
policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

```yaml schema agent-config
name: orchestration_parallel_defaults
version: "1.0.0"
description: Baseline controls for parallel orchestration runs with agent-mail coordination
max_steps: 30
max_tool_calls: 80
max_wall_time_seconds: 900
max_cost_dollars: 3.00
max_concurrent_agents: 4
dependency_strategy: fail_fast
failure_handling: abort_all
retry_policy:
  max_retries: 2
  backoff_seconds: 5
require_confirmation_for_high_risk: true
allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
allowed_tools:
  - web_search
  - retrieve_docs
  - agent_mail_register
  - agent_mail_send_message
  - agent_mail_check_inbox
  - agent_mail_reserve_file
forbidden_tools:
  - execute_sql_query
policy_tags:
  data_classification: confidential
  retention_class: STANDARD
```

```yaml schema agent-config
name: workspace_agent_config_example
version: "1.0.0"
description: Example workspace-specific agent configuration
workspace_id: "workspace-123"
environment: "prod"
max_steps: 5
max_tool_calls: 10
max_wall_time_seconds: 180
max_cost_dollars: 0.50
require_confirmation_for_high_risk: true
allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
allowed_tools:
  - web_search
  - retrieve_docs
forbidden_tools:
  - execute_sql_query
  - write_file
default_system_prompt_id: "default-assistant"
policy_tags:
  data_classification: confidential
  retention_class: STANDARD
```

**Inheritance:**
- Workspace config overrides global defaults
- Environment config (dev/stage/prod) overrides workspace defaults
- Individual run can specify parameters but cannot exceed configured maximums

### Agent Execution Flow

```
1. User initiates run (CLI, TUI, API)
   ↓
2. Client validates request (fast feedback)
   - Check tool schemas exist
   - Check prompt template valid
   - Basic parameter validation
   ↓
3. Client sends request to Control Plane
   - Includes: actor_id, session_id, device_id
   - Includes: prompt, tools, model, parameters
   ↓
4. Server validates and enforces policy
   - RBAC: does actor have permission?
   - ABAC: do attributes allow this action?
   - Budget: is there sufficient budget?
   - Allowlists: are model/tools permitted in this env?
   ↓
5. Server creates run record (audit trail)
   - Generates request_id
   - Logs initial state
   ↓
6. Agent loop executes (server-side or via gateway)
   - Step 1: Generate response
   - If tool calls → validate, execute, record
   - Step 2: Continue with tool results
   - ... repeat until done or limits reached
   ↓
7. Server finalizes run record
   - Final status (completed, terminated, failed)
   - Cost accounting
   - Token usage
   - Policy decisions applied
   ↓
8. Client receives response (streaming or batch)
   - Sanitize outputs before rendering
   - Display tool calls, policy decisions
   - Show cost breakdown
```

### Break-Glass Mode

For incident response, elevated permissions can be granted temporarily:

```yaml schema agent-config
name: break_glass_config
version: "1.0.0"
description: Break-glass elevated permissions for incident response
break_glass: true
reason_code: "incident-2024-02-23-auth-bypass"
elevation_expires_at: "2026-02-23T14:00:00Z"
elevated_by: "incident-commander@example.com"
max_steps: 50
max_tool_calls: 100
max_wall_time_seconds: 3600
max_cost_dollars: 10.00
allowed_tools:
  - "*"  # All tools allowed
override_policy_checks:
  - tool_allowlist
  - environment_restrictions
audit_tag: "BREAK_GLASS"
policy_tags:
  data_classification: restricted
  retention_class: LEGAL_HOLD
```

**Break-glass requirements:**
- Explicit reason code (required)
- Time-bound elevation (expires_at required)
- Elevated by (incident commander role required)
- All actions tagged with BREAK_GLASS
- Alerts triggered on usage
- Extra audit retention

### Future Agent Capabilities (Planned)

**Multi-Agent Orchestration**
- Hierarchical agent delegation
- Parallel agent execution with result aggregation
- Inter-agent communication protocols

**Advanced RAG**
- Hybrid search (vector + keyword)
- Re-ranking with cross-encoders
- Multi-hop retrieval
- Query rewriting and expansion

**Agent Memory**
- Session-scoped working memory
- Long-term knowledge base integration
- Episodic memory for context recall

**Proactive Agents**
- Scheduled agent runs (cron-like)
- Event-driven triggers
- Monitoring and alerting workflows

---

## Schema Authoring Guidelines

When adding or modifying agent-config schemas:

1. **Version properly:** Follow semver (major.minor.patch)
2. **Validate limits:** Ensure max values are reasonable and safe
3. **Test inheritance:** Verify workspace/env/run-level overrides work correctly
4. **Document overrides:** Explain when to override defaults
5. **Security review:** High-risk configurations require security team approval
6. **Audit considerations:** Ensure all config changes are logged

---

## Examples

### Example 1: Strict Production Agent Config

Minimal permissions for production environment:

```yaml schema agent-config
name: prod_strict_config
version: "1.0.0"
workspace_id: "critical-app"
environment: "prod"
max_steps: 3
max_tool_calls: 5
max_wall_time_seconds: 60
max_cost_dollars: 0.10
require_confirmation_for_high_risk: true  # Forbidden in prod anyway
allowed_models:
  - claude-sonnet-4-5  # Sonnet only, no Opus
allowed_tools:
  - web_search  # Only read-only tools
  - retrieve_docs
forbidden_tools:
  - execute_sql_query
  - write_file
  - "*_write"  # Glob pattern
policy_tags:
  data_classification: confidential
  retention_class: STANDARD
```

### Example 2: Permissive Dev Environment Config

Generous limits for development/testing:

```yaml schema agent-config
name: dev_permissive_config
version: "1.0.0"
workspace_id: "experimental"
environment: "dev"
max_steps: 50
max_tool_calls: 100
max_wall_time_seconds: 1800
max_cost_dollars: 5.00
require_confirmation_for_high_risk: false  # Auto-approve in dev
allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
allowed_tools:
  - "*"  # All tools allowed
forbidden_tools: []
policy_tags:
  data_classification: internal
  retention_class: SHORT
```

---

## Changelog

### 1.0 (2026-02-23)
- Initial version
- Defined agent_loop_defaults
- Defined break_glass_config
- Added workspace_agent_config_example
- Established Part A instructions for AI assistants

---

## Next Steps

- Add more agent config examples as use cases emerge
- Define specialized agent patterns (code review agent, security audit agent, etc.)
- Implement agent orchestration schemas (multi-agent workflows)
- Add memory and state management schemas

---

**For questions or clarifications, see:**
- Product specification (section 3.10, 4.4 for agent requirements)
- Design document: `docs/plans/2026-02-23-ai-development-files-design.md`
- Implementation plan: `docs/plans/2026-02-23-implementation-plan.md`
