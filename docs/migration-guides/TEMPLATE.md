# Migration Guide: {schema_name} v{old_version} → v{new_version}

**Date:** {date}
**Type:** {breaking|feature|deprecation}
**Affected Schemas:** {schema_type}

---

## Summary

{brief description of what changed and why}

---

## Breaking Changes

### {change_1_title}

**What changed:**
{description of the change}

**Before:**
```yaml
{old_example}
```

**After:**
```yaml
{new_example}
```

**Migration:**
```rust
// Code migration example
{code_example}
```

---

### {change_2_title}
{...}

---

## Deprecations

### {deprecated_field}

**Deprecated in:** v{new_version}
**Will be removed in:** v{planned_removal_version}

**Migration:**
```rust
{migration_code}
```

---

## New Features

### {feature_name}

{description of new feature}

---

## Timeline

- **{date}**: v{new_version} released
- **{removal_date}**: Deprecated fields will be removed
- **{end_of_support}**: v{old_version} no longer supported

---

## Rollback Plan

If you encounter issues after upgrading:

1. {step_1}
2. {step_2}
3. {step_3}

---

## Questions?

- Open an issue: {repo_url}/issues
- Discussion: {repo_url}/discussions
- Slack: #{slack_channel}
