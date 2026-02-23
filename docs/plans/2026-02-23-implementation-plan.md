# AI Development Files - Implementation Plan

**Date:** 2026-02-23
**Status:** In Progress (Phases 0-5 implemented, final expansion items pending)
**Design Doc:** [2026-02-23-ai-development-files-design.md](./2026-02-23-ai-development-files-design.md)

---

## Implementation Phases

### Phase 0: Bootstrap and Foundation (Completed)

**Goal:** Create initial markdown files, basic validation infrastructure, and tests.

**Tasks:**

#### Task 0.1: Create Bootstrap Markdown Files ✓
- [x] Create `AGENTS.md` with Part A instructions and example agent-config schema
- [x] Create `CLAUDE.md` with Part A instructions and Claude provider config
- [x] Create `MCP_SERVERS.md` with github and context7 examples
- [x] Create `PROMPTS.md` with default-assistant and rag-query examples
- [x] Create `EVALS.md` with policy-enforcement-suite and terminal-safety-suite
- [x] Create `TOOLS.md` with web_search and execute_sql_query examples

**Deliverables:**
- 6 markdown files with complete frontmatter
- Part A: Comprehensive AI assistant instructions
- Part B: 1-2 example schemas per file
- Inline documentation and comments

---

#### Task 0.2: Create Meta-Schemas

**Goal:** Define JSON Schema validators for each schema type.

**Files to create:**
- `docs/schemas/tool-schema.json`
- `docs/schemas/prompt-schema.json`
- `docs/schemas/eval-schema.json`
- `docs/schemas/mcp-server-schema.json`
- `docs/schemas/agent-config-schema.json`
- `docs/schemas/claude-config-schema.json`

**Key constraints for meta-schemas:**
- All schemas require: `name`/`id`, `version` (semver format)
- Optional: `deprecated_versions`, `migration_guide`, `policy_tags`
- Type-specific required fields based on design doc

**Validation rules:**
- Version must match semver pattern: `^\d+\.\d+\.\d+$`
- Policy tags must use valid enums (data_classification, retention_class)
- Cross-references must be validated (e.g., prompts reference valid models)

**Dependencies:** None

**Estimated complexity:** Medium

---

#### Task 0.3: Implement build.rs Parser and Validator

**Goal:** Extract and validate schema blocks from markdown files.

**Implementation steps:**

1. **Setup build dependencies in Cargo.toml:**
```toml
[build-dependencies]
pulldown-cmark = "0.9"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
jsonschema = "0.17"
semver = "1.0"
```

2. **Implement extraction logic:**
   - Parse markdown with pulldown-cmark
   - Extract fenced code blocks matching `yaml schema <type>`
   - Capture source file path and line number for errors

3. **Implement validation logic:**
   - Load meta-schema from `docs/schemas/`
   - Compile JSON Schema validator
   - Validate each extracted block
   - Check version format (semver)
   - Check ID uniqueness within type
   - Validate cross-references

4. **Error reporting:**
   - Clear error messages with file:line references
   - Suggestions for common mistakes
   - Exit with non-zero code on validation failure

**Key functions:**
```rust
fn extract_schemas(markdown: &str, schema_type: &str, source_file: &str) -> Vec<SchemaBlock>
fn validate_against_meta_schema(schema: &SchemaBlock, meta_schema_path: &str) -> Result<()>
fn check_unique_ids(schemas: &[SchemaBlock]) -> Result<()>
fn validate_cross_references(tools: &[Schema], prompts: &[Schema], evals: &[Schema]) -> Result<()>
```

**Dependencies:** Task 0.2 (meta-schemas must exist)

**Estimated complexity:** High

---

#### Task 0.4: Implement Code Generation (Basic)

**Goal:** Generate Rust types and registries from validated schemas.

**Implementation steps:**

1. **Generate struct definitions:**
   - Create serde-compatible structs for each schema type
   - Include all fields from YAML schemas
   - Add derives: `Debug, Clone, Serialize, Deserialize`

2. **Generate const registries:**
   - `pub const TOOLS: &[ToolSchema] = &[...]`
   - `pub const PROMPTS: &[PromptSchema] = &[...]`
   - `pub const EVALS: &[EvalSchema] = &[...]`
   - `pub const AGENTS: &[AgentConfig] = &[...]`
   - `pub const MCP_SERVERS: &[McpServerSchema] = &[...]`

3. **Generate validation functions:**
   - `validate_tool_input(name: &str, input: &serde_json::Value) -> Result<()>`
   - `render_prompt(id: &str, variables: &serde_json::Value) -> Result<String>`

4. **Output files:**
   - `src/generated/tools.rs`
   - `src/generated/prompts.rs`
   - `src/generated/evals.rs`
   - `src/generated/agents.rs`
   - `src/generated/mcp_servers.rs`
   - `src/generated/mod.rs` (re-exports)

**Code generation approach:**
- Use `quote!` macro for generating Rust tokens
- Write generated code to files with warning header
- Format with rustfmt if available

**Dependencies:** Task 0.3 (validated schemas)

**Estimated complexity:** High

---

#### Task 0.5: Create Test Infrastructure

**Goal:** Comprehensive test coverage for schema validation and code generation.

**Test files to create:**

1. **`tests/schema_validation.rs`:**
   - `test_all_schemas_parse_and_validate()`
   - `test_no_duplicate_ids()`
   - `test_cross_references_resolve()`
   - `test_version_format_valid()`
   - `test_policy_tags_valid()`

2. **`tests/tools/contract_tests.rs`:**
   - `test_tool_web_search_contract()`
   - `test_tool_execute_sql_query_contract()`
   - Helper: `run_contract_test(test: &ContractTest)`

3. **`tests/roundtrip.rs`:**
   - `test_markdown_to_generated_code_roundtrip()`
   - `test_schema_stability_on_rebuild()`

4. **`tests/security/terminal_escape.rs`:**
   - `test_blocks_osc52_clipboard_injection()`
   - `test_blocks_cursor_movement()`
   - `test_preserves_safe_ansi_colors()`
   - `test_terminal_safety_eval_suite_passes()`

**Test helpers:**
```rust
fn extract_tool_schemas() -> Vec<ToolSchema>
fn get_tool_schema(name: &str, version: &str) -> ToolSchema
fn validate_tool_input(name: &str, input: &serde_json::Value) -> Result<()>
```

**Dependencies:** Task 0.3, 0.4 (build.rs and generated code)

**Estimated complexity:** Medium

---

#### Task 0.6: Setup CI/CD Pipeline

**Goal:** Automated validation on every commit.

**Create `.github/workflows/schema-validation.yml`:**

```yaml
name: Schema Validation

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  validate-schemas:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Build (runs build.rs)
        run: cargo build

      - name: Run schema validation tests
        run: cargo test schema_validation

      - name: Run all tests
        run: cargo test

      - name: Check for generated code drift
        run: |
          if ! git diff --quiet src/generated/; then
            echo "❌ Generated code is out of sync!"
            echo "Run 'cargo build' locally and commit the changes."
            git diff src/generated/
            exit 1
          fi

      - name: Lint
        run: cargo clippy -- -D warnings

      - name: Format check
        run: cargo fmt -- --check
```

**Dependencies:** Task 0.3, 0.4, 0.5

**Estimated complexity:** Low

---

#### Task 0.7: Setup Git Hooks

**Goal:** Prevent invalid schemas from being committed.

**Create `.githooks/pre-commit`:**

```bash
#!/bin/bash
set -e

echo "🔍 Running pre-commit schema validation..."

# Run build (which validates schemas)
echo "  Building (validates schemas)..."
cargo build || {
    echo "❌ Schema validation failed!"
    exit 1
}

# Run schema tests
echo "  Running schema validation tests..."
cargo test schema_validation || {
    echo "❌ Schema tests failed!"
    exit 1
}

# Check generated code is staged
if ! git diff --quiet HEAD src/generated/; then
    if ! git diff --cached --quiet src/generated/; then
        echo "✅ Generated code changes are staged"
    else
        echo "⚠️  Generated code has changes but they're not staged!"
        echo "   Run: git add src/generated/"
        exit 1
    fi
fi

echo "✅ Pre-commit validation passed!"
```

**Create `.githooks/commit-msg`:**

```bash
#!/bin/bash

commit_msg_file="$1"
commit_msg=$(cat "$commit_msg_file")

# Check if schema files are modified
schema_files_changed=$(git diff --cached --name-only | grep -E "(TOOLS|PROMPTS|EVALS|AGENTS|MCP_SERVERS|CLAUDE)\.md" || true)

if [ -n "$schema_files_changed" ]; then
    # Enforce conventional commit format for schema changes
    if ! echo "$commit_msg" | grep -qE "^(feat|fix|breaking|docs)\((tools|prompts|evals|agents|mcp|claude)\):"; then
        echo "❌ Schema changes require conventional commit format:"
        echo ""
        echo "Examples:"
        echo "  feat(tools): add web_search tool"
        echo "  breaking(prompts): remove deprecated variable from rag-query"
        echo "  fix(evals): correct threshold in policy-enforcement-suite"
        echo ""
        exit 1
    fi
fi
```

**Installation script `.githooks/install.sh`:**

```bash
#!/bin/bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
chmod +x .githooks/commit-msg
echo "✅ Git hooks installed!"
```

**Dependencies:** Task 0.3, 0.4

**Estimated complexity:** Low

---

#### Task 0.8: Create Initial Cargo.toml

**Goal:** Setup Rust project structure.

**Create `Cargo.toml`:**

```toml
[package]
name = "chroma-ai-dev"
version = "0.1.0"
edition = "2021"
authors = ["ChromaAI Dev Team"]
description = "Terminal-first AI development, evaluation, and release tool"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/chroma-ai-dev"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"

[build-dependencies]
pulldown-cmark = "0.9"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
jsonschema = "0.17"
quote = "1.0"
syn = "2.0"
semver = "1.0"

[dev-dependencies]
```

**Create basic src structure:**
- `src/main.rs` - Entry point (minimal for now)
- `src/lib.rs` - Library crate
- `src/generated/mod.rs` - Generated code module (placeholder)

**Dependencies:** None

**Estimated complexity:** Low

---

### Phase 0 Acceptance Criteria

- [ ] All 6 markdown files created with complete Part A + Part B examples
- [ ] All meta-schemas created in `docs/schemas/`
- [ ] `build.rs` extracts and validates schemas successfully
- [ ] `build.rs` generates Rust code in `src/generated/`
- [ ] `cargo build` completes without errors
- [ ] All tests pass: `cargo test`
- [ ] CI pipeline runs successfully
- [ ] Git hooks prevent invalid commits
- [ ] No schema validation errors
- [ ] Generated code is deterministic (rebuild produces same output)

---

## Phase 1: Tool Schema Implementation and Runtime Validation

**Goal:** Implement full tool schema validation and execution infrastructure.

### Task 1.1: Implement Tool Input Validation Runtime

**What:**
- JSON Schema validator for tool inputs
- Error messages with field-level details
- Support for all JSON Schema features (pattern, min/max, enum, etc.)

**Implementation:**
```rust
pub fn validate_tool_input(
    tool_name: &str,
    version: &str,
    input: &serde_json::Value
) -> Result<(), ValidationError> {
    let tool = get_tool_schema(tool_name, version)?;

    // Compile JSON Schema
    let compiled = jsonschema::JSONSchema::compile(&tool.input_schema)?;

    // Validate
    if let Err(errors) = compiled.validate(input) {
        return Err(ValidationError::SchemaViolation {
            tool: tool_name.to_string(),
            errors: errors.collect()
        });
    }

    Ok(())
}
```

**Dependencies:** Phase 0 complete

**Estimated complexity:** Medium

---

### Task 1.2: Implement Contract Test Runner

**What:**
- Execute contract tests defined in TOOLS.md
- Report pass/fail with detailed output
- Integrate into test suite

**Implementation:**
```rust
pub fn run_contract_test(test: &ContractTest) -> TestResult {
    let result = validate_tool_input(&test.tool_name, &test.input);

    match (result, test.expect_success) {
        (Ok(_), true) => TestResult::Pass,
        (Err(e), false) if test.error_pattern.is_match(&e.to_string()) => TestResult::Pass,
        _ => TestResult::Fail(/* details */)
    }
}
```

**Dependencies:** Task 1.1

**Estimated complexity:** Medium

---

### Task 1.3: Add More Tool Schemas

**What:**
- Add 5-10 additional tool schemas to TOOLS.md
- Cover common patterns: HTTP calls, file operations, data transformations
- Include both low-risk and high-risk examples

**Examples:**
- `http_request` (configurable HTTP client)
- `read_file` (with path restrictions)
- `write_file` (high-risk, dev-only)
- `parse_json` (transformation)
- `format_date` (utility)

**Dependencies:** Task 1.1, 1.2

**Estimated complexity:** Medium

---

## Phase 2: Prompt Template Engine

**Goal:** Implement prompt template rendering with variable substitution.

### Task 2.1: Implement Template Renderer

**What:**
- Handlebars or similar templating engine
- Variable substitution with type checking
- Support for conditionals and loops

**Implementation:**
```rust
pub fn render_prompt(
    prompt_id: &str,
    variables: &serde_json::Value
) -> Result<String, RenderError> {
    let prompt = get_prompt_schema(prompt_id)?;

    // Validate variables against schema
    validate_prompt_variables(prompt, variables)?;

    // Render template
    let handlebars = Handlebars::new();
    handlebars.render_template(&prompt.template, variables)
        .map_err(Into::into)
}
```

**Dependencies:** Phase 0 complete

**Estimated complexity:** Medium

---

### Task 2.2: Add Prompt Variable Validation

**What:**
- Validate variables match prompt schema
- Type checking (string, number, boolean, array, object)
- Required field checking
- Enum value validation

**Dependencies:** Task 2.1

**Estimated complexity:** Low

---

### Task 2.3: Add More Prompt Templates

**What:**
- Add 5-10 common prompt patterns
- System prompts for different use cases
- User prompts with various complexities

**Examples:**
- `code-review-assistant`
- `summarize-with-constraints`
- `extract-entities`
- `chain-of-thought-reasoning`

**Dependencies:** Task 2.1, 2.2

**Estimated complexity:** Low

---

## Phase 3: Evaluation Suite Runner

**Goal:** Implement evaluation execution and regression gating.

### Task 3.1: Implement Deterministic Eval Runner

**What:**
- Execute eval cases with deterministic grading
- Compare outputs against expected values
- Calculate pass rates and thresholds

**Implementation:**
```rust
pub async fn run_eval_suite(suite: &EvalSuite) -> EvalRunResult {
    let mut results = vec![];

    for case in &suite.cases {
        let result = match case.grading_method {
            GradingMethod::Deterministic => run_deterministic_case(case),
            GradingMethod::LlmJudge => run_llm_judge_case(case, &suite.judge_config),
        };
        results.push(result);
    }

    calculate_suite_result(results, &suite.thresholds)
}
```

**Dependencies:** Phase 0, Phase 1

**Estimated complexity:** High

---

### Task 3.2: Implement LLM-as-Judge Eval Runner

**What:**
- Execute eval cases with LLM grading
- Support repeat trials and variance tolerance
- Parse judge responses (PASS/FAIL)

**Dependencies:** Task 3.1

**Estimated complexity:** High

---

### Task 3.3: Implement Regression Gating

**What:**
- Block on failing regression-critical evals
- Generate regression reports
- Store eval results as immutable artifacts

**Dependencies:** Task 3.1, 3.2

**Estimated complexity:** Medium

---

## Phase 4: Version Management and Multi-Version Support

**Goal:** Support multiple schema versions simultaneously.

### Task 4.1: Implement Version Resolution

**What:**
- Resolve "latest" to actual version
- Validate version compatibility
- Warning on deprecated version usage

**Dependencies:** Phase 0 complete

**Estimated complexity:** Medium

---

### Task 4.2: Generate Multi-Version Validators

**What:**
- Code generation for all supported versions
- Version-specific validation functions
- Deprecation warnings in generated code

**Dependencies:** Task 4.1

**Estimated complexity:** High

---

### Task 4.3: Create Migration Guide Templates

**What:**
- Standard template for migration guides
- Examples of breaking changes
- Automated detection of breaking changes

**Dependencies:** Task 4.1

**Estimated complexity:** Low

---

## Phase 5: Documentation and Developer Experience

**Goal:** Improve developer experience and documentation.

### Task 5.1: Create Schema Authoring Guide

**What:**
- How to add new tools
- How to modify existing schemas
- Versioning best practices
- Testing requirements

**Dependencies:** Phase 0-4 complete

**Estimated complexity:** Low

---

### Task 5.2: Add Schema Linting

**What:**
- Lint for common mistakes
- Suggest improvements
- Check for missing fields

**Dependencies:** Phase 0 complete

**Estimated complexity:** Medium

---

### Task 5.3: Generate HTML Documentation

**What:**
- Auto-generate HTML docs from markdown schemas
- Interactive schema browser
- Version history visualization

**Dependencies:** Phase 0-4 complete

**Estimated complexity:** Medium

---

## Implementation Order

**Week 1:**
- Task 0.1: Bootstrap markdown files ✓
- Task 0.2: Meta-schemas
- Task 0.8: Cargo.toml setup

**Week 2:**
- Task 0.3: build.rs parser and validator
- Task 0.4: Code generation

**Week 3:**
- Task 0.5: Test infrastructure
- Task 0.6: CI pipeline
- Task 0.7: Git hooks

**Week 4:**
- Task 1.1: Tool input validation runtime
- Task 1.2: Contract test runner
- Task 1.3: Add more tool schemas

**Week 5-6:**
- Phase 2: Prompt template engine
- Phase 3: Evaluation suite runner

**Week 7-8:**
- Phase 4: Version management
- Phase 5: Documentation

---

## Risk Assessment

### High-Risk Items

1. **build.rs complexity**
   - Risk: Complex parsing and validation logic
   - Mitigation: Incremental implementation with extensive tests

2. **JSON Schema validation performance**
   - Risk: Slow validation for complex schemas
   - Mitigation: Compile validators once at build time

3. **Code generation correctness**
   - Risk: Generated code doesn't match schemas
   - Mitigation: Round-trip tests, manual review

4. **Version compatibility**
   - Risk: Breaking changes not detected
   - Mitigation: Automated breaking change detection

### Medium-Risk Items

1. **Cross-reference validation**
   - Risk: Missing or circular references
   - Mitigation: Explicit validation in build.rs

2. **LLM-as-judge reliability**
   - Risk: Non-deterministic evaluation results
   - Mitigation: Multiple trials, variance tolerance

3. **Template engine complexity**
   - Risk: Complex templates hard to debug
   - Mitigation: Clear error messages, examples

---

## Success Metrics

**Phase 0:**
- ✅ All 6 markdown files created
- [x] 100% schema validation coverage
- [x] All tests passing
- [ ] CI pipeline green
- [ ] Zero manual validation needed

**Phase 1:**
- [x] 10+ tool schemas defined
- [x] All contract tests passing
- [x] Runtime validation working

**Phase 2:**
- [x] 10+ prompt templates defined
- [x] Template rendering working
- [x] Variable validation working

**Phase 3:**
- [x] 5+ eval suites defined
- [x] Deterministic eval runner working
- [x] LLM-as-judge working
- [x] Regression gating working

**Phase 4:**
- [x] Multi-version support working
- [ ] Migration guides for all breaking changes
- [x] Deprecation warnings functional

**Phase 5:**
- [x] Complete authoring guide
- [x] Schema linting functional
- [x] HTML docs generated

---

## Current Status

**Completed:**
- [x] Design document
- [x] Implementation plan
- [x] Phase 0: Bootstrap and validation/codegen foundation
- [x] Phase 1: Tool validation runtime + contract runner + expanded tool catalog
- [x] Phase 2: Prompt rendering + variable validation + expanded prompt catalog
- [x] Phase 3: Deterministic and LLM-judge eval runners + regression gating
- [x] Phase 4: Version resolution and latest-aware runtime wrappers
- [x] Phase 5: Authoring guide + schema linting + HTML docs generation

**Next Steps:**
1. Add migration guides for each future breaking schema release (Phase 4 hardening)
2. Validate CI run status on PR and mark CI metric complete when green in GitHub Actions
3. Decide whether to keep generated-file rustfmt in build.rs or replace with stable formatting strategy

**Blockers:** None

---

**Last Updated:** 2026-02-23
**Next Review:** After CI confirmation and migration-guide rollout policy is finalized
