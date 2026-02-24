# Issue Tracking Ticket System Design

**Date:** 2026-02-24  
**Status:** Implemented (Phase 1b)  
**Purpose:** Plan an issue/ticket system for ongoing work, inspired by [Beads](https://github.com/steveyegge/beads) — git-backed, dependency-aware, agent-friendly.

---

## 1. Overview

ChromaAI Dev needs a **persistent, structured way to track work** that:

- Works for **AI agents** (JSON output, dependency-aware “ready” work, no interactive editors).
- Stays **git-friendly** (merge-safe IDs, optional file-based storage).
- Fits the **markdown-first** culture (docs/plans, AGENTS.md).
- Can evolve toward **server-backed** tickets later (audit, RBAC) without a rewrite.

This document compares options, defines a ticket schema and workflow, and recommends a path.

---

## 2. Beads Inspiration — What We Want

From [Beads](https://github.com/steveyegge/beads):

| Capability | Beads | Our goal |
|------------|--------|----------|
| **Storage** | Dolt (versioned SQL) | File-based or SQLite/Dolt; git-tracked or sync’d |
| **IDs** | Hash-based (`bd-a1b2`) to avoid merge collisions | Same idea: short stable IDs (e.g. `chr-a1b2` or UUID prefix) |
| **Dependencies** | `blocks`, `related`, `parent-child`, `discovered-from` | At least `blocks` / `blocked_by`; optional `relates_to`, `parent` |
| **Ready work** | `bd ready` = tasks with no open blockers | Equivalent: list issues with all blockers closed |
| **Hierarchy** | Epics: `bd-a3f8.1`, `bd-a3f8.1.1` | Optional: epic/task/subtask or flat with `parent_id` |
| **Agent UX** | `--json` on all commands; no `bd edit` (no $EDITOR) | JSON-first; updates via flags/API, not interactive edit |
| **Sync** | Dolt push/pull or git (JSONL export) | Git-tracked files or explicit sync |
| **Compaction** | Summarize/archive old closed tasks | Optional: archive or summarize closed after N days |

We do **not** need to reimplement Beads in Rust. We can:

- **Option A:** Adopt Beads (`bd`) as the project’s issue tracker (document in AGENTS.md, use `.beads/`).
- **Option B:** File-based tickets in-repo (e.g. `.chroma/issues/` or `docs/issues/`) with a small CLI/scripts.
- **Option C:** Custom Rust issue tracker (SQLite or Dolt, `chroma tickets` subcommands).

---

## 3. Ticket Schema (Canonical)

Regardless of storage, tickets should have a **stable schema** so agents and tools can rely on it.

### 3.1 Core Fields

```yaml
# Conceptual ticket schema (YAML for doc; storage can be JSON/markdown/DB)

id: chr-a1b2              # Short hash or UUID prefix; unique, merge-safe
title: "Add OIDC device flow"
type: task                 # task | bug | epic | story | meta
status: open               # open | in_progress | done | cancelled
priority: 1                # 0=critical, 1=high, 2=medium, 3=low
created_at: "2026-02-24T10:00:00Z"
updated_at: "2026-02-24T12:00:00Z"

# Optional
description: |
  Multi-line description.
assignee: null             # agent id or user; optional
blocked_by: [chr-x9f2]     # list of ticket IDs; all must be done for "ready"
relates_to: [chr-y1a3]     # optional; no ordering semantics
parent_id: null            # chr-epic1 for subtasks; optional

# Agent / audit (optional until server-backed)
discovered_from: null      # chr-abc if this was spawned from another ticket
source_plan: "docs/plans/2026-02-23-implementation-plan.md"  # optional
```

### 3.2 Relation Types

- **blocked_by** (required for “ready”): This ticket is not ready until all listed tickets are `done` (or `cancelled`).
- **relates_to**: Informational link; does not affect ready.
- **parent_id**: Hierarchical (epic → task → subtask); optional.
- **discovered_from**: For agent-created follow-up work; optional.

### 3.3 ID Format

- **Short hash** (Beads-style): e.g. `chr-` + 4–6 hex chars from UUID. Prefer 4 for &lt;500 tickets, scale up to avoid collisions.
- **Alternative**: `chr-` + first 8 chars of UUID. No collision risk in practice.
- Rule: **No sequential IDs** (they collide across branches).

---

## 4. Agent Workflow

### 4.1 Essential Operations (CLI or API)

- **create** — Create ticket (title, type, priority, optional description, optional parent_id / blocked_by).
- **update** — Set status, assignee, title, description, add/remove blocked_by, etc. (by flag; no interactive editor).
- **list** — Filter by status, priority, assignee; output table or JSON.
- **ready** — List tickets with `status in (open, in_progress)` and no open blockers (transitive optional).
- **show &lt;id&gt;** — Full ticket + optional audit/history; JSON and human-readable.
- **close** — Set status to `done` (or `cancelled`) with optional reason/comment.

All write operations should be **non-interactive** (flags or JSON body) so agents can drive them.

### 4.2 Commit Message Convention

Recommend appending ticket ID to commits that complete work:

```text
Fix auth validation bug (chr-a1b2)
Add retry logic for DB locks (chr-x9f2)
```

Enables “orphan detection”: commits that reference a ticket but the ticket wasn’t closed (optional doctor/lint step).

### 4.3 “Land the plane” Checklist (Session End)

When ending a session, agents should:

1. Create tickets for any remaining follow-up work.
2. Close or update status for completed work.
3. Push to remote (no “ready when you are” — actually push).
4. Optionally run a “doctor” that checks: open tickets with recent commits mentioning their ID but still open.

This mirrors Beads’ “Landing the Plane” in AGENT_INSTRUCTIONS.md.

---

## 5. Storage Options (Detailed)

### 5.1 Option A: Adopt Beads (`bd`)

- **Pros:** Mature, Dolt-backed, hash IDs, `bd ready`, JSON, MCP server, no in-repo implementation cost.
- **Cons:** External binary (Go); different prefix (`bd-` not `chr-`); less control over schema and future server integration.
- **Effort:** Document in AGENTS.md and CONTRIBUTING; add `bd init` to repo setup; use `bd` in “land the plane” and agent instructions.

**Recommendation:** Strong option if the team is fine with a separate tool and `bd-` IDs. Add to AGENTS.md: “Use `bd` for task tracking; run `bd ready --json` for next work.”

### 5.2 Option B: File-Based Tickets (Markdown + Index)

- **Layout:**  
  - `.chroma/issues/chr-a1b2.md` (one file per ticket) or  
  - Single `docs/issues/tickets.yaml` (or JSON/JSONL) with all tickets.
- **Per-file format:** YAML frontmatter + optional markdown body for description.
- **Index:** Generated or maintained: `.chroma/issues/index.json` (id → path, status, blocked_by) for fast `ready` without parsing every file.
- **Pros:** Git-tracked, markdown-first, no runtime deps, trivial to add to repo.
- **Cons:** `ready` and dependency walks need either a script or a small Rust CLI; merge conflicts possible if two branches touch same ticket (mitigate with one file per ticket and hash IDs).

**Schema in frontmatter example:**

```markdown
---
id: chr-a1b2
title: "Add OIDC device flow"
type: task
status: open
priority: 1
blocked_by: [chr-x9f2]
parent_id: null
created_at: "2026-02-24T10:00:00Z"
updated_at: "2026-02-24T10:00:00Z"
---

Optional long description in markdown below.
```

**Recommendation:** Good first step: minimal, fits markdown/schema culture, can be driven by scripts or a small `chroma tickets` subcommand later.

### 5.3 Option C: Custom Rust Tracker (SQLite or Dolt)

- **Storage:** SQLite in `.chroma/issues.db` or Dolt for versioning and sync.
- **CLI:** `chroma tickets create/list/ready/show/update/close` with `--json`.
- **Pros:** Single binary, full control, same ID prefix and schema, natural path to server-backed tickets (same schema, different backend).
- **Cons:** Implementation and maintenance cost; need migrations, backup/export story.

**Recommendation:** Consider after file-based is in use and limits hit (e.g. need for faster queries, compaction, or server sync). Schema in this doc can be the contract for both file-based and DB backends.

---

## 6. Recommended Phasing

### Phase 1: Lightweight adoption (immediate)

- **Choose one:**
  - **1a.** Adopt Beads: document in AGENTS.md, add `bd init` (or `bd init --quiet` for CI/agents) to setup, and use `bd ready` / `bd create` / `bd update` / `bd close` in agent workflow and “land the plane.”
  - **1b.** Or introduce file-based tickets: define `.chroma/issues/` layout and frontmatter schema (Section 3), add a small script or `chroma tickets` that: create (generate ID, write file), list (glob + parse), ready (filter by status + resolve blocked_by from index or files), show, update, close. Output JSON for agents.
- Add “Issue tracking” to AGENTS.md: where tickets live, how to create/update/close, commit message convention, and “land the plane” checklist.

### Phase 2: Consistency and hygiene

- Add a **doctor** (or CI job) that: checks for orphan commits (commit message contains ticket ID but ticket still open); optionally validates ticket frontmatter/index.
- Optionally: compaction or archive (e.g. move closed tickets older than 90 days to `docs/issues/archive/` or summarize and remove from index).

### Phase 3: Optional server-backed (later)

- If control plane gains an “issues” API: same ticket schema; server is source of truth; audit (actor_id, request_id) and RBAC; client uses same `create/list/ready/show/update/close` semantics over HTTP. File-based or Beads can remain for local-only use.

---

## 7. Integration with Existing Docs

- **docs/plans/** — Design and implementation plans (e.g. `2026-02-23-implementation-plan.md`) can reference tickets by ID: “Task 0.3 (chr-a1b2).”
- **AGENTS.md** — Add subsection: “Issue tracking: use `bd` (or `chroma tickets`) for task tracking; run `ready` for next work; close tickets when done; include ticket ID in commit messages; follow land-the-plane checklist.”
- **EVALS.md** — Optional: eval that creates/updates/closes tickets and checks `ready` output (contract test).

---

## 8. Summary

| Item | Recommendation |
|------|----------------|
| **Schema** | Section 3: id, title, type, status, priority, blocked_by, relates_to, parent_id, timestamps; hash-style IDs. |
| **Agent workflow** | create/update/list/ready/show/close; JSON output; no interactive edit; commit message convention; land-the-plane checklist. |
| **Phase 1** | Either adopt Beads (1a) or file-based `.chroma/issues/` + minimal CLI/scripts (1b). Document in AGENTS.md. |
| **Phase 2** | Doctor for orphans and optional compaction/archive. |
| **Phase 3** | Optional server-backed API reusing same schema. |

This gives a Beads-like issue tracking ticket system that fits ChromaAI Dev’s markdown-first, agent-friendly, and future server-authoritative direction.

---

## Appendix: Ticket Schema (For Validation/Codegen)

For validation or codegen, the ticket schema can be defined in a future `ISSUES.md` or `docs/schemas/ticket-schema.json`. Conceptual field spec:

| Field | Type | Required | Notes |
|-------|------|----------|--------|
| id | string | yes | Pattern `chr-[a-f0-9]{4,8}` |
| title | string | yes | |
| type | enum | no | task, bug, epic, story, meta (default task) |
| status | enum | no | open, in_progress, done, cancelled (default open) |
| priority | integer 0–3 | no | 0=critical, 2=medium default |
| created_at, updated_at | date-time | yes | |
| description, assignee | string | no | |
| blocked_by, relates_to | array of id | no | |
| parent_id, discovered_from, source_plan | string | no | |

---

## Changelog

- **2026-02-24:** Initial design: Beads-inspired goals, ticket schema, agent workflow, storage options (adopt bd / file-based / custom Rust), phased recommendation.
