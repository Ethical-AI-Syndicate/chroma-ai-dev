# Schema Authoring Guide

This guide defines the required process for adding or changing schemas in ChromaAI Dev.

## Scope

The following schema sources are validated and code-generated at build time:

- `TOOLS.md`
- `PROMPTS.md`
- `EVALS.md`
- `AGENTS.md`
- `MCP_SERVERS.md`
- `CLAUDE.md`

## Authoring Workflow

1. Add or modify a `yaml schema <type>` fenced block.
2. Increment `version` using semver rules.
3. Add or update contract/eval coverage for changed behavior.
4. Run:
   - `cargo build`
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt -- --check`
5. Ensure generated files under `src/generated/` are committed.

## Required Fields

All schemas must include:

- Stable schema ID (`name`, `id`, or `suite_id` by type)
- `version` in full semver form (`MAJOR.MINOR.PATCH`)
- `description`
- `policy_tags` where required by type

## Versioning Rules

- **Major:** breaking shape/contract changes
- **Minor:** backward-compatible additions
- **Patch:** non-breaking fixes and clarifications

Use `docs/templates/schema-migration-guide.md` for breaking changes.

## Validation Expectations

- Schema must pass JSON meta-schema validation.
- Cross-references must resolve (models, prompt IDs, etc.).
- Contract tests must prove allow/deny behavior for tool inputs.
- Security-sensitive schemas must include negative tests.

## Review Checklist

- [ ] Version bump is correct for change type
- [ ] Backward compatibility impact documented
- [ ] Migration guide added for breaking changes
- [ ] Runtime tests cover new or changed behavior
- [ ] Generated code has no drift
