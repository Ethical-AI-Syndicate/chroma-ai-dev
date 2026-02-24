# Migration Guide: prompt legacy-assistant v1.0.0 → v2.0.0

**Date:** 2026-02-24
**Type:** deprecation
**Affected Schemas:** prompt

---

## Summary

The `legacy-assistant` prompt template has been deprecated in favor of the new `default-assistant` prompt, which includes improved variable validation, better prompt engineering patterns, and support for additional model configurations.

---

## Breaking Changes

None. This is a deprecation, not a breaking change.

---

## Deprecations

### legacy-assistant

**Deprecated in:** v1.0.0
**Will be removed in:** v3.0.0

**Migration:**
Replace all references to `legacy-assistant` with `default-assistant`:

```rust
// Before
let prompt = render_prompt("legacy-assistant", &variables);

// After
let prompt = render_prompt("default-assistant", &variables);
```

The `default-assistant` prompt accepts the same variables:
- `user_query`: The user's input query
- `context`: Optional context from RAG retrieval
- `system_prompt`: Optional custom system prompt override

---

## New Features

### default-assistant

The new `default-assistant` prompt includes:
- Improved chain-of-thought reasoning instructions
- Better tool use guidelines
- Support for multi-step reasoning
- Enhanced safety instructions

---

## Timeline

- **2026-02-24**: v1.0.0 deprecated
- **2026-06-01**: v2.0.0 released (legacy-assistant still available)
- **2026-09-01**: v3.0.0 - legacy-assistant removed

---

## Questions?

- Open an issue: https://github.com/Ethical-AI-Syndicate/chroma-ai-dev/issues
