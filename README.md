# ChromaAI Dev

**ChromaTUI-based AI development, evaluation, and release tool.** Without ChromaTUI there is no application—the TUI is the foundation everything is built on.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Status: Phase 0 (Bootstrap)](https://img.shields.io/badge/status-Phase%200%20(Bootstrap)-yellow.svg)](docs/plans/2026-02-23-implementation-plan.md)

---

## Overview

ChromaAI Dev is an enterprise-grade terminal-first tool for AI development, evaluation, and release management. It provides:

- **Prompt/workflow authoring** with typed variables and versioning
- **RAG corpus management** with ACL enforcement and reproducibility
- **Evaluation harness** with regression gates and LLM-as-judge support
- **Release management** with approvals, rollbacks, and audit trails
- **Cost controls** with budgets and real-time enforcement
- **Incident response** controls with kill switches and forensic exports

## Architecture

The application **is** ChromaTUI. Without ChromaTUI there is no application. All features are delivered through the TUI.

ChromaAI Dev follows a **thin-client** architecture:

- **Client (ChromaTUI)**: The application; user interaction, validates for UX
- **Control Plane**: Server-side policy enforcement, artifact storage, audit logging
- **Execution Plane**: Provider gateways, tool execution, retrieval services

### Thin-Client Principle

- **Server decides**: Policy, artifacts, budgets, audit
- **Server stores**: Credentials, artifacts, audit logs
- **Client validates for UX, Server validates for security** (defense in depth)

## Documentation Structure

This project uses **markdown-first, schema-embedded documentation**:

| File | Purpose |
|------|---------|
| [AGENTS.md](AGENTS.md) | AI assistant instructions + agent runtime specs |
| [CLAUDE.md](CLAUDE.md) | Project-specific instructions + Claude API integration |
| [MCP_SERVERS.md](MCP_SERVERS.md) | MCP server registry with security posture |
| [PROMPTS.md](PROMPTS.md) | Prompt template library with typed variables |
| [EVALS.md](EVALS.md) | Evaluation suites with regression gates |
| [TOOLS.md](TOOLS.md) | Tool/function registry with JSON schemas |

Each file serves **dual purposes**:
1. **Part A**: Instructions for AI assistants developing ChromaAI Dev
2. **Part B**: Runtime specifications (schemas) that ChromaAI Dev uses

### Schema Validation

Schemas are embedded in markdown as ` ```yaml schema <type> ` blocks and validated at build time:

```yaml schema tool
name: web_search
version: "1.0"
description: Performs web search
risk_rating: low
allowed_environments: [dev, stage, prod]
input_schema:
  type: object
  properties:
    query: {type: string}
  required: [query]
```

**Build pipeline:**
1. `build.rs` extracts schema blocks from markdown
2. Validates against meta-schemas (`docs/schemas/*.json`)
3. Generates Rust code (`src/generated/`)
4. CI ensures schemas valid and generated code in sync

## Project Status

**Current Phase**: Phase 0 (Bootstrap) - See [Implementation Plan](docs/plans/2026-02-23-implementation-plan.md)

✅ **Completed:**
- Design document
- Implementation plan
- All 6 markdown files with bootstrap content
- Meta-schemas for validation
- Cargo.toml with dependencies
- Basic src/ structure

🚧 **In Progress:**
- build.rs (schema extraction and validation)
- Code generation from schemas
- Test infrastructure

📋 **Next Steps:**
- Week 2: Implement build.rs parser and validator
- Week 2: Implement code generation
- Week 3: Test infrastructure and CI pipeline

## Quick Start

### Prerequisites

- Rust 1.75+ (2021 edition)
- Git

### Build

```bash
# Clone repository
git clone https://github.com/Ethical-AI-Syndicate/chroma-ai-dev.git
cd chroma-ai-dev

# Build (will run build.rs to validate schemas)
cargo build

# Run tests
cargo test

# Run CLI
cargo run -- --help
```

### Available Commands

```bash
# Validate schema files
cargo run -- validate

# Authenticate with SSO (not implemented yet)
cargo run -- login

# Initialize workspace (not implemented yet)
cargo run -- init my-workspace
```

## Development

### Adding New Schemas

1. **Add schema block** to appropriate markdown file (TOOLS.md, PROMPTS.md, etc.)
2. **Run build**: `cargo build` (validates and generates code)
3. **Write tests**: Add contract tests or validation tests
4. **Commit**: Include markdown changes + generated code

**Example (adding a new tool):**

```bash
# 1. Edit TOOLS.md and add schema block
vim TOOLS.md

# 2. Build (validates schema and generates code)
cargo build

# 3. Write contract tests
vim tests/tools/contract_tests.rs

# 4. Run tests
cargo test

# 5. Commit all changes
git add TOOLS.md src/generated/ tests/
git commit -m "feat(tools): add new_tool v1.0.0"
```

### Schema Versioning

Follow **semantic versioning** (semver):

- **Major** (1.0 → 2.0): Breaking changes (removed fields, changed types)
- **Minor** (1.0 → 1.1): Backward-compatible additions (new optional fields)
- **Patch** (1.0.0 → 1.0.1): Documentation updates, bug fixes

See [Design Document](docs/plans/2026-02-23-ai-development-files-design.md) for full versioning guidelines.

### Testing Requirements

- **Policy enforcement tests**: 100% pass rate (regression blocking)
- **Terminal safety tests**: 100% pass rate (security critical)
- **Contract tests**: Required for all tools
- **Integration tests**: Required for SSO/RBAC flows

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run all tests
cargo test

# Run specific test suite
cargo test schema_validation
```

## Documentation

- **[Design Document](docs/plans/2026-02-23-ai-development-files-design.md)**: Complete architecture and design
- **[Implementation Plan](docs/plans/2026-02-23-implementation-plan.md)**: Phased implementation approach
- **[Product Specification](docs/product-spec.md)**: Full product requirements (not yet created)

## Contributing

**Phase 0 Status**: Project is in bootstrap phase. Contribution guidelines will be established in Phase 1.

For now, see:
- [AGENTS.md](AGENTS.md) - Development guidelines for AI assistants
- [CLAUDE.md](CLAUDE.md) - Project-specific requirements

## License

Dual-licensed under MIT OR Apache-2.0 at your option.

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/)
- [Tokio](https://tokio.rs/) - Async runtime
- [Serde](https://serde.rs/) - Serialization
- [JSON Schema](https://json-schema.org/) - Validation
- [Handlebars](https://handlebarsjs.com/) - Templating

---

**Status**: 🚧 Phase 0 (Bootstrap) - See [implementation plan](docs/plans/2026-02-23-implementation-plan.md) for roadmap
