# Advanced Agent Platform Design (LSP + Orchestration + Modes)

**Date:** 2026-02-23
**Status:** Proposed (user-approved direction)
**Scope:** M1 full advanced stack

---

## 1. Goals

Build an enterprise-safe, server-authoritative agent platform with:

1. Built-in LSP services for `Rust`, `TypeScript`, `Python`, `Go`, and `Java`.
2. Parallel subagent orchestration with explicit dependencies and deterministic joins.
3. Durable inter-agent communication modeled after mailbox/thread patterns with advisory file leases.
4. Explicit agent modes (`plan`, `build`, `review`, `incident`) with policy and budget enforcement.
5. Complete auditability for every action (`actor_id`, `session_id`, `device_id`, `request_id`, `decision_id`).

---

## 2. Non-Goals (M1)

1. Long-term autonomous memory beyond run/session scope.
2. Full distributed microservice decomposition (M1 is modular monolith).
3. Non-terminal GUI workflows beyond existing web or CLI/TUI pathways.

---

## 3. Architecture Decision

### 3.1 Chosen Approach

**Server-authoritative modular monolith** in Rust, with clean internal boundaries:

- `control_plane` (runs, policy, budget, audit)
- `orchestrator` (DAG scheduler, fan-out/fan-in)
- `agent_mail` (mailbox, threads, search, leases)
- `lsp_manager` (language server lifecycle + unified facade)
- `modes` (plan/build/review/incident state machine)

This preserves the thin-client principle and reduces distributed failure complexity while retaining future extraction paths.

### 3.2 Why Not Alternatives

- Full service mesh now adds ops complexity and weakens delivery speed for M1.
- Plugin-first core introduces larger security/versioning surface too early.

---

## 4. Core Components

## 4.1 Control Plane

Responsibilities:

- Run creation/termination and status lifecycle.
- Policy checks for each transition/tool/LSP action.
- Budget accounting and hard stops.
- Immutable audit event emission.

Core constraints:

- Client-side checks are UX-only; server checks are authoritative.
- No tool execution, LSP operation, or orchestration transition bypasses policy.

## 4.2 Mode Engine

Modes:

- `plan`: analysis and planning only, no mutating tools.
- `build`: implementation allowed per policy/tool allowlist.
- `review`: read-only + evaluators + static checks.
- `incident`: break-glass envelope with explicit expiry and reason.

State transitions are explicit and audited.

## 4.3 Orchestrator

Model:

- Workload DAG with `TaskNode` and dependency edges.
- Node-level budget caps and tool/LSP/mode constraints.
- Parallel scheduling for independent nodes.
- Join reducers for deterministic fan-in.

Failure handling:

- Retry policy per node type.
- Cancellation propagation to dependent nodes.
- Circuit breaker behavior when budget/policy violations occur.

## 4.4 Agent Mail (mcp_agent_mail-like)

Capabilities:

- Register logical agent identities.
- Send/fetch/ack messages.
- Threaded conversation history.
- Searchable archive (SQLite + FTS).
- Advisory file lease system with TTL and exclusivity.

Persistence:

- SQLite authoritative index and query path.
- Optional markdown mirror artifacts for human audit readability.

## 4.5 Built-in LSP Manager

M1 required language adapters:

- Rust: `rust-analyzer`
- TypeScript: `typescript-language-server`
- Python: `pyright`
- Go: `gopls`
- Java: `jdtls`

Responsibilities:

- Launch/monitor/restart language servers per workspace.
- Capability negotiation and health checks.
- Unified API for diagnostics, hover, definitions, references, completion, code actions.
- Workspace boundary enforcement and operation allowlisting.

---

## 5. Data Model (M1)

## 5.1 Run and Orchestration

- `RunSpec`: mode, limits, policy tags, model/tool allowlists.
- `RunState`: pending/running/blocked/terminated/completed.
- `TaskNode`: objective, deps, budget slice, allowed capabilities, expected outputs.
- `TaskResult`: pass/fail, artifacts, metrics, audit refs.

## 5.2 Communication and Coordination

- `AgentIdentity`: agent_id, display_name, program/model metadata.
- `Message`: thread_id, from/to, subject, markdown body, importance, ack_required.
- `Lease`: lease_id, agent_id, path_pattern, exclusive, ttl, status.

## 5.3 LSP

- `LspSession`: language, workspace, process handle, capabilities, health.
- `LspRequestLog`: operation, target, latency, result, decision_id.

---

## 6. Security and Compliance Design

Mandatory controls:

1. Terminal output sanitization before render.
2. No secrets on disk; keychain/vault only.
3. Full identity and decision binding on every server action.
4. Break-glass sessions require reason, approver, expiry, and elevated audit tags.
5. Policy and budget hard-stop semantics (no soft bypass).

---

## 7. API Surface (M1)

Representative server endpoints/services:

- Run control: create/list/get/cancel runs.
- Mode transition: request and validate mode changes.
- Orchestration: submit DAG, inspect node status, fetch artifacts.
- Agent mail: register, send, fetch inbox, acknowledge, search thread, lease claim/release.
- LSP facade: diagnostics, hover, completion, symbol navigation, code actions.

All endpoint handlers must emit audit events and enforce policy/budget checks.

---

## 8. Rollout Plan

1. Land foundational schemas + runtime modules for modes, orchestration, mail, and LSP sessions.
2. Implement deterministic integration tests before feature enablement.
3. Enable features behind explicit M1 flags.
4. Run regression suites (policy, terminal safety, orchestration correctness, mailbox lease conflicts).
5. Promote to default-on in prod only after policy/audit verification gates pass.

---

## 9. Acceptance Criteria

M1 is complete when all are true:

1. Five built-in LSP adapters start and respond through unified facade.
2. Parallel DAG workloads execute with deterministic joins and audit trace.
3. Inter-agent mailbox, threading, and lease conflict handling are fully functional.
4. Mode transitions enforce policy and budget constraints server-authoritatively.
5. Test suites pass for policy, terminal safety, orchestration, mailbox/leases, and LSP contracts.
6. CI is green on `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check`.
