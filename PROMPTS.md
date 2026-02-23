---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft
---

# PROMPTS.md - Prompt Template Library

This file defines prompt templates with typed variables, policy tags, and versioning. Prompts are immutable once published - modifications create new versions.

**Purpose:** Source of truth for prompt templates used throughout ChromaAI Dev.

---

## System Prompts

### default-assistant

Default system prompt for ChromaAI Dev assistant with policy awareness.

```yaml schema prompt
id: default-assistant
version: "1.0"
type: system
description: Default system prompt with identity binding and policy context
template: |
  You are an AI assistant integrated into ChromaAI Dev, an enterprise-grade
  terminal-first AI development, evaluation, and release tool.

  You have access to tools and must follow strict policy constraints.
  All your actions are audited and tied to user identity.

  **Current Session Context:**
  - User: {{user_id}}
  - Session: {{session_id}}
  - Environment: {{environment}}
  - Workspace: {{workspace_id}}
  - Allowed tools: {{#each allowed_tools}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
  - Budget remaining: ${{budget_remaining}}

  **Policy Requirements:**
  - Follow the principle of least privilege
  - Always validate inputs before processing
  - Respect tool risk ratings and environment restrictions
  - Do not attempt to bypass policy controls
  - Report policy violations immediately

  **Tool Usage:**
  - Only use tools from the allowed list
  - Validate tool inputs match schemas
  - Handle tool errors gracefully
  - Do not retry failed tools without user confirmation

  Provide clear, concise responses. Explain your reasoning when using tools.

variables:
  user_id:
    type: string
    required: true
    description: Authenticated user identifier (email or SSO subject)
  session_id:
    type: string
    required: true
    description: Session identifier for audit trail correlation
  environment:
    type: string
    enum: [dev, stage, prod]
    required: true
    description: Current environment context (affects policy)
  workspace_id:
    type: string
    required: true
    description: Active workspace identifier
  allowed_tools:
    type: array
    items: {type: string}
    required: true
    description: List of tools available in this session (post-policy-filtering)
  budget_remaining:
    type: number
    minimum: 0
    required: true
    description: Remaining budget in dollars for this session/workspace

policy_tags:
  data_classification: internal
  retention_class: SHORT

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

## User Prompts

### rag-query

Query template for RAG-enhanced responses with document context.

```yaml schema prompt
id: rag-query
version: "1.0"
type: user
description: Query with retrieved document context and citation requirements
template: |
  **User Query:** {{query}}

  **Retrieved Context** (corpus: {{corpus_id}}, version: {{corpus_version}}):

  {{#each retrieved_docs}}
  ---
  **Document {{inc @index}}** (score: {{this.score}}, source: {{this.source_id}}):

  {{this.content}}

  Metadata:
  - ACL Groups: {{#each this.acl_groups}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
  - Ingested: {{this.ingested_at}}

  {{/each}}

  **Instructions:**
  1. Answer the query using ONLY the provided context above
  2. If the context doesn't contain sufficient information, explicitly state: "The provided context does not contain enough information to answer this question."
  3. Cite document numbers when referencing specific information (e.g., "According to Document 2...")
  4. Do not use external knowledge or make assumptions beyond what's in the context
  5. If there are contradictions between documents, note them explicitly

variables:
  query:
    type: string
    required: true
    minLength: 1
    maxLength: 2000
    description: User's question or query
  corpus_id:
    type: string
    required: true
    description: Identifier of the corpus used for retrieval
  corpus_version:
    type: string
    required: true
    description: Version of the corpus (for reproducibility)
  retrieved_docs:
    type: array
    required: true
    minItems: 0
    maxItems: 50
    description: Array of retrieved documents with metadata
    items:
      type: object
      properties:
        content:
          type: string
          description: Document text content
        source_id:
          type: string
          description: Source identifier (file path, URL, etc.)
        score:
          type: number
          description: Retrieval relevance score
        acl_groups:
          type: array
          items: {type: string}
          description: ACL groups that have access to this doc
        ingested_at:
          type: string
          format: date-time
          description: When document was ingested
      required: [content, source_id, score]

policy_tags:
  data_classification: varies  # Depends on corpus classification
  retention_class: STANDARD

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

## Prompt Rendering

**Template engine:** Handlebars

**Helpers available:**
- `{{inc}}` - Increment number (for 1-indexed display)
- `{{#each}}` - Iterate over arrays
- `{{#unless}}` - Conditional negation
- `{{#if}}` - Conditional

**Variable validation:**
- Types checked at render time
- Required fields enforced
- Enum values validated
- Min/max lengths checked
- Min/max values checked

**Example usage:**
```rust
let variables = json!({
    "user_id": "alice@example.com",
    "session_id": "sess_abc123",
    "environment": "prod",
    "workspace_id": "ws_xyz",
    "allowed_tools": ["web_search", "retrieve_docs"],
    "budget_remaining": 0.75
});

let rendered = render_prompt("default-assistant", &variables)?;
// Returns fully rendered system prompt string
```

---

## Adding New Prompts

**Process:**

1. **Add schema block** to this file
2. **Define variables** with types and constraints
3. **Write template** using Handlebars syntax
4. **Test rendering** with sample variables
5. **Validate against meta-schema:** Run `cargo build`
6. **Add examples** in this file
7. **Version appropriately:** Start at 1.0 for new prompts

**Versioning rules:**
- **Major:** Breaking changes (removed variables, changed semantics)
- **Minor:** New optional variables, clarifications
- **Patch:** Typo fixes, formatting improvements

---

## Prompt Best Practices

**Clarity:**
- Be explicit about instructions
- Use numbered lists for multi-step tasks
- Define expected output format

**Safety:**
- Include policy reminders in system prompts
- Specify constraints and limitations
- Handle edge cases (empty context, no results, etc.)

**Variables:**
- Use descriptive names
- Provide clear descriptions
- Set appropriate constraints (min/max, enum)
- Mark required vs optional

**Maintenance:**
- Keep prompts focused (single responsibility)
- Reuse common patterns via variables
- Document intent and use cases
- Test with real data

---

## Changelog

### 1.0 (2026-02-23)
- Initial version
- Added default-assistant system prompt
- Added rag-query user prompt
- Established prompt schema format

---

## Next Steps

- Add more system prompts for specialized roles (code review, security audit, etc.)
- Add user prompt patterns (summarization, extraction, transformation, etc.)
- Define prompt composition strategies
- Add few-shot example templates
- Add chain-of-thought prompting patterns
