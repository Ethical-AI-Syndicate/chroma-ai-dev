# Advanced Agent Platform Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement server-authoritative advanced agent capabilities: built-in LSP services, parallel subagent orchestration, mailbox-style inter-agent coordination with leases, and explicit plan/build/review/incident modes.

**Architecture:** Implement as a modular monolith in Rust with strict boundaries (`modes`, `orchestrator`, `agent_mail`, `lsp_manager`, `control_plane`) so policy, budget, and audit checks are centralized and unavoidable. Use TDD-first for every runtime behavior and keep state transitions deterministic and observable.

**Tech Stack:** Rust, tokio, serde/serde_json, sqlx/sqlite (or rusqlite for M1), thiserror/anyhow, existing schema extraction/build.rs pipeline, GitHub Actions CI.

---

### Task 1: Add Schemas for Modes, Orchestration, and Agent Mail

**Files:**
- Modify: `AGENTS.md`
- Modify: `TOOLS.md`
- Modify: `EVALS.md`
- Test: `tests/schema_validation.rs`

**Step 1: Write the failing test**

Add tests in `tests/schema_validation.rs` asserting:
- At least one `agent-config` schema has mode controls.
- Tool schemas include mailbox/lease operations (`register_agent`, `send_message`, `claim_file_lease`).

**Step 2: Run test to verify it fails**

Run: `cargo test --test schema_validation`
Expected: FAIL because schemas are missing these definitions.

**Step 3: Add minimal schema blocks**

Add to markdown schema files:
- Agent mode config schema extension in `AGENTS.md`.
- Mailbox + lease tool schemas in `TOOLS.md`.
- Deterministic eval suite for mode policy and lease conflicts in `EVALS.md`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test schema_validation`
Expected: PASS.

**Step 5: Commit**

```bash
git add AGENTS.md TOOLS.md EVALS.md tests/schema_validation.rs src/generated/
git commit -m "feat(schemas): add mode and agent-mail schema definitions"
```

---

### Task 2: Implement Mode State Machine Runtime

**Files:**
- Create: `src/modes.rs`
- Modify: `src/lib.rs`
- Test: `tests/modes.rs`

**Step 1: Write the failing test**

Create `tests/modes.rs` tests for:
- Valid transitions (`plan -> build`, `build -> review`).
- Invalid transitions rejected.
- Incident mode requires reason and expiry.

**Step 2: Run test to verify it fails**

Run: `cargo test --test modes`
Expected: FAIL because `modes` module does not exist.

**Step 3: Write minimal implementation**

Implement:
- `AgentMode` enum.
- `ModeTransitionRequest` + `ModeTransitionError`.
- `transition_mode(current, request)`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test modes`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/modes.rs src/lib.rs tests/modes.rs
git commit -m "feat(modes): add agent mode transition state machine"
```

---

### Task 3: Implement Orchestration DAG Core

**Files:**
- Create: `src/orchestrator.rs`
- Modify: `src/lib.rs`
- Test: `tests/orchestrator.rs`

**Step 1: Write the failing test**

Create tests for:
- DAG dependency ordering.
- Parallel execution for independent nodes.
- Join waits for all prerequisites.

**Step 2: Run test to verify it fails**

Run: `cargo test --test orchestrator`
Expected: FAIL because orchestrator module is missing.

**Step 3: Write minimal implementation**

Implement:
- `TaskNode`, `TaskGraph`, `NodeStatus`.
- Topological scheduling and async node executor hooks.
- Deterministic join reducer invocation.

**Step 4: Run test to verify it passes**

Run: `cargo test --test orchestrator`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/orchestrator.rs src/lib.rs tests/orchestrator.rs
git commit -m "feat(orchestrator): add DAG scheduler and join behavior"
```

---

### Task 4: Implement Agent Mail + Lease Storage Layer

**Files:**
- Create: `src/agent_mail.rs`
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Test: `tests/agent_mail.rs`

**Step 1: Write the failing test**

Create tests for:
- Register agent identity.
- Send/fetch/ack messages by thread.
- Exclusive lease conflict behavior and TTL expiry transitions.

**Step 2: Run test to verify it fails**

Run: `cargo test --test agent_mail`
Expected: FAIL because module/storage is missing.

**Step 3: Write minimal implementation**

Implement SQLite-backed layer with:
- `register_agent`, `send_message`, `fetch_inbox`, `ack_message`.
- `claim_file_lease`, `release_file_lease`, conflict checks.

**Step 4: Run test to verify it passes**

Run: `cargo test --test agent_mail`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml src/agent_mail.rs src/lib.rs tests/agent_mail.rs
git commit -m "feat(agent-mail): add mailbox threads and file lease coordination"
```

---

### Task 5: Implement Built-in LSP Manager and Unified Facade

**Files:**
- Create: `src/lsp_manager.rs`
- Modify: `src/lib.rs`
- Test: `tests/lsp_manager.rs`

**Step 1: Write the failing test**

Create tests for:
- Registering required adapters (`rust`, `typescript`, `python`, `go`, `java`).
- Session lifecycle (start/stop/restart).
- Unified operation routing (diagnostics, hover, completion).

**Step 2: Run test to verify it fails**

Run: `cargo test --test lsp_manager`
Expected: FAIL because manager module is missing.

**Step 3: Write minimal implementation**

Implement:
- `LanguageKind` enum with five languages.
- `LspSessionManager` with lifecycle methods.
- Unified facade API with stubbed transport abstraction.

**Step 4: Run test to verify it passes**

Run: `cargo test --test lsp_manager`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/lsp_manager.rs src/lib.rs tests/lsp_manager.rs
git commit -m "feat(lsp): add built-in multi-language lsp manager facade"
```

---

### Task 6: Integrate Policy/Budget/Audit Enforcement at Control Plane Boundaries

**Files:**
- Create: `src/control_plane.rs`
- Modify: `src/modes.rs`
- Modify: `src/orchestrator.rs`
- Modify: `src/agent_mail.rs`
- Modify: `src/lsp_manager.rs`
- Test: `tests/control_plane.rs`

**Step 1: Write the failing test**

Create tests for:
- Deny unauthorized mode transitions.
- Deny tool/lease/LSP operations when policy forbids.
- Budget hard-stop triggers termination state.
- Audit events emitted with required identifiers.

**Step 2: Run test to verify it fails**

Run: `cargo test --test control_plane`
Expected: FAIL because central enforcement is missing.

**Step 3: Write minimal implementation**

Implement control-plane guard methods invoked from each module before action execution.

**Step 4: Run test to verify it passes**

Run: `cargo test --test control_plane`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/control_plane.rs src/modes.rs src/orchestrator.rs src/agent_mail.rs src/lsp_manager.rs tests/control_plane.rs
git commit -m "feat(control-plane): enforce policy budget and audit across advanced runtime"
```

---

### Task 7: Add Regression and Security Evals for Advanced Features

**Files:**
- Modify: `EVALS.md`
- Test: `tests/eval_runner.rs`

**Step 1: Write the failing test**

Extend `tests/eval_runner.rs` with cases for:
- mode-policy suite.
- lease-conflict suite.
- orchestrator-join determinism suite.

**Step 2: Run test to verify it fails**

Run: `cargo test --test eval_runner`
Expected: FAIL due to missing suites/cases.

**Step 3: Add minimal eval suites**

Add deterministic/llm_judge suites in `EVALS.md` matching test expectations.

**Step 4: Run test to verify it passes**

Run: `cargo test --test eval_runner`
Expected: PASS.

**Step 5: Commit**

```bash
git add EVALS.md src/generated/evals.rs tests/eval_runner.rs
git commit -m "feat(evals): add advanced feature regression suites"
```

---

### Task 8: Update CI and Docs for Advanced Platform

**Files:**
- Modify: `.github/workflows/schema-validation.yml`
- Modify: `docs/plans/2026-02-23-implementation-plan.md`
- Modify: `docs/schema-authoring-guide.md`

**Step 1: Write the failing test/check**

Add validation assertions in tests (or workflow YAML parse checks) for new test targets:
- `modes`, `orchestrator`, `agent_mail`, `lsp_manager`, `control_plane`.

**Step 2: Run test/check to verify it fails**

Run: `cargo test --test workflow_yaml`
Expected: FAIL until workflow updated.

**Step 3: Write minimal implementation**

Update CI workflow to run full advanced test matrix and keep generated code in sync.
Update docs and implementation status to reflect added capabilities.

**Step 4: Run checks to verify it passes**

Run:
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt -- --check`

Expected: PASS.

**Step 5: Commit**

```bash
git add .github/workflows/schema-validation.yml docs/plans/2026-02-23-implementation-plan.md docs/schema-authoring-guide.md
git commit -m "docs(ci): update workflows and plan status for advanced platform rollout"
```

---

## Final Verification Gate

Run exactly:

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

If any fail, fix before merge.

---

## Notes

- Keep commits frequent and scoped to one task.
- Do not skip failing-test-first sequence.
- Use generated schema updates from `cargo build` in each schema-touching task.
- Preserve thin-client rule: no authoritative policy decisions in CLI/TUI.
