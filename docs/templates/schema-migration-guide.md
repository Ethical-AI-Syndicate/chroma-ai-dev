# Schema Migration Guide Template

Use this template when a schema change introduces breaking behavior and requires consumer migration.

---

## Metadata

- **Schema Type:** `<tool|prompt|eval|agent-config|mcp-server|claude-config>`
- **Schema ID/Name:** `<id-or-name>`
- **From Version:** `<old-version>`
- **To Version:** `<new-version>`
- **Change Type:** `<major|minor|patch>`
- **Effective Date:** `<YYYY-MM-DD>`
- **Owner:** `<team-or-person>`

---

## Why This Change Exists

Describe the business, security, compliance, or reliability reason for the version change.

---

## Breaking Changes

List each incompatible change explicitly.

1. `<change-1>`
2. `<change-2>`

For each change, include impact scope:

- **Who is affected:** `<users/systems>`
- **Failure mode:** `<validation error/runtime behavior>`
- **Risk level:** `<low|medium|high|critical>`

---

## Before and After Examples

### Before (`<old-version>`)

```yaml
# old schema or payload example
```

### After (`<new-version>`)

```yaml
# new schema or payload example
```

---

## Required Migration Steps

1. Update schema references to `<new-version>`.
2. Update payload fields according to the breaking change list.
3. Run contract tests and schema validation.
4. Roll out in `<dev -> stage -> prod>` sequence.

---

## Validation Checklist

- [ ] `cargo build` passes (schema extraction + generation)
- [ ] `cargo test` passes
- [ ] Contract tests for affected schema pass
- [ ] Regression-blocking eval suites pass
- [ ] Audit log fields remain stable

---

## Rollback Plan

Document explicit rollback procedure and stop conditions.

1. `<rollback-step-1>`
2. `<rollback-step-2>`

---

## Communication Plan

- **Announcement date:** `<YYYY-MM-DD>`
- **Channels:** `<slack/email/changelog/release-notes>`
- **Decommission date for old version:** `<YYYY-MM-DD>`

---

## Changelog Entry Template

`breaking(<schema-area>): migrate <id-or-name> from <old-version> to <new-version> due to <reason>`
