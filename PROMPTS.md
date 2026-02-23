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
version: "1.0.0"
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
version: "1.0.0"
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

### code-review-assistant

System prompt for structured, policy-aware code review responses.

```yaml schema prompt
id: code-review-assistant
version: "1.0.0"
type: system
description: System prompt for review mode with security, correctness, and audit focus
template: |
  You are operating in code review mode for ChromaAI Dev.

  **Review Scope:** {{review_scope}}
  **Repository:** {{repository}}
  **Branch:** {{branch}}
  **Severity Threshold:** {{severity_threshold}}

  **Review priorities:**
  1. Security defects and policy violations
  2. Correctness and edge-case handling
  3. Reliability and error propagation
  4. Tests and validation coverage

  Return findings as:
  - severity
  - file path
  - issue
  - recommendation

variables:
  review_scope:
    type: string
    required: true
    description: Scope description of files or components under review
  repository:
    type: string
    required: true
    description: Repository identifier
  branch:
    type: string
    required: true
    description: Branch under review
  severity_threshold:
    type: string
    enum: [critical, high, medium, low]
    required: true
    description: Minimum severity to include in report

policy_tags:
  data_classification: internal
  retention_class: SHORT

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### summarize-with-constraints

User prompt for constrained summary output.

```yaml schema prompt
id: summarize-with-constraints
version: "1.0.0"
type: user
description: Summarization prompt with explicit length and format constraints
template: |
  Summarize the text below.

  **Input Text:**
  {{text}}

  **Constraints:**
  - Maximum words: {{max_words}}
  - Tone: {{tone}}
  - Include bullet points: {{include_bullets}}

  Ensure all key points are preserved.

variables:
  text:
    type: string
    required: true
    minLength: 1
    maxLength: 20000
    description: Text to summarize
  max_words:
    type: integer
    required: true
    minimum: 20
    maximum: 500
    description: Maximum allowed words in summary
  tone:
    type: string
    required: true
    enum: [neutral, technical, executive]
    description: Desired output tone
  include_bullets:
    type: boolean
    required: true
    description: Whether output should be bullet-based

policy_tags:
  data_classification: internal
  retention_class: STANDARD

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### extract-entities

User prompt for deterministic entity extraction.

```yaml schema prompt
id: extract-entities
version: "1.0.0"
type: user
description: Extracts typed entities from text and returns normalized JSON output
template: |
  Extract the entities from the text below.

  **Text:**
  {{text}}

  **Entity Types to Extract:**
  {{#each entity_types}}- {{this}}
  {{/each}}

  Return strict JSON with keys: type, value, confidence, evidence.

variables:
  text:
    type: string
    required: true
    minLength: 1
    maxLength: 20000
    description: Source text for extraction
  entity_types:
    type: array
    required: true
    minItems: 1
    maxItems: 20
    items: {type: string}
    description: Entity categories to extract

policy_tags:
  data_classification: internal
  retention_class: STANDARD

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### chain-of-thought-reasoning

User prompt for stepwise reasoning with concise final answer.

```yaml schema prompt
id: chain-of-thought-reasoning
version: "1.0.0"
type: user
description: Structured reasoning prompt for complex tasks with explicit final answer block
template: |
  Solve the problem below using explicit reasoning steps.

  **Problem:**
  {{problem_statement}}

  **Constraints:**
  - Maximum reasoning steps: {{max_steps}}
  - Include verification: {{include_verification}}

  Return:
  1. Reasoning steps
  2. Final answer in a dedicated section

variables:
  problem_statement:
    type: string
    required: true
    minLength: 1
    maxLength: 10000
    description: Problem statement requiring structured reasoning
  max_steps:
    type: integer
    required: true
    minimum: 1
    maximum: 20
    description: Maximum number of reasoning steps
  include_verification:
    type: boolean
    required: true
    description: Whether to include an explicit verification step

policy_tags:
  data_classification: internal
  retention_class: SHORT

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### incident-response-update

User prompt for incident response summaries with compliance fields.

```yaml schema prompt
id: incident-response-update
version: "1.0.0"
type: user
description: Produces incident status updates including impact, mitigations, and next actions
template: |
  Generate an incident response update.

  **Incident ID:** {{incident_id}}
  **Severity:** {{severity}}
  **Current Status:** {{status}}

  **Observed Impact:**
  {{impact_summary}}

  **Mitigations Applied:**
  {{#each mitigations}}- {{this}}
  {{/each}}

  **Next Update ETA (minutes):** {{next_update_eta_minutes}}

  Include clear owner, timeline, and user-facing impact statement.

variables:
  incident_id:
    type: string
    required: true
    description: Unique incident identifier
  severity:
    type: string
    required: true
    enum: [sev1, sev2, sev3, sev4]
    description: Incident severity classification
  status:
    type: string
    required: true
    enum: [investigating, identified, monitoring, resolved]
    description: Current incident lifecycle status
  impact_summary:
    type: string
    required: true
    minLength: 1
    maxLength: 4000
    description: Summary of customer/system impact
  mitigations:
    type: array
    required: true
    minItems: 0
    maxItems: 20
    items: {type: string}
    description: List of mitigation actions already applied
  next_update_eta_minutes:
    type: integer
    required: true
    minimum: 1
    maximum: 240
    description: Minutes until next planned status update

policy_tags:
  data_classification: confidential
  retention_class: STANDARD

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### onboarding-checklist-generator

System prompt to produce role-based onboarding checklists.

```yaml schema prompt
id: onboarding-checklist-generator
version: "1.0.0"
type: system
description: Generates structured onboarding checklists with ownership and due dates
template: |
  Create an onboarding checklist for a {{role}} joining team {{team}}.

  Constraints:
  - Include exactly {{task_count}} tasks
  - Group tasks by week
  - Include owner and due date for each item

  Return concise markdown bullets.

variables:
  role:
    type: string
    required: true
    minLength: 2
    maxLength: 100
    description: Role title for onboarding checklist
  team:
    type: string
    required: true
    minLength: 2
    maxLength: 100
    description: Team name the new joiner belongs to
  task_count:
    type: integer
    required: true
    minimum: 3
    maximum: 30
    description: Number of checklist items required

policy_tags:
  data_classification: internal
  retention_class: SHORT

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### deployment-summary

User prompt for deployment summaries with incident-aware framing.

```yaml schema prompt
id: deployment-summary
version: "1.0.0"
type: user
description: Summarizes deployment outcomes, risk status, and rollback readiness
template: |
  Summarize this deployment event.

  Environment: {{environment}}
  Service: {{service_name}}
  Version: {{release_version}}
  Duration Minutes: {{duration_minutes}}
  Had Incidents: {{had_incident}}

  Include:
  1) Overall outcome
  2) Risk posture
  3) Recommended next action

variables:
  environment:
    type: string
    required: true
    enum: [dev, stage, prod]
    description: Deployment environment
  service_name:
    type: string
    required: true
    minLength: 2
    maxLength: 100
    description: Service name
  release_version:
    type: string
    required: true
    minLength: 3
    maxLength: 50
    description: Version identifier for release
  duration_minutes:
    type: integer
    required: true
    minimum: 1
    maximum: 240
    description: Deployment duration in minutes
  had_incident:
    type: boolean
    required: true
    description: Whether incidents occurred during deployment

policy_tags:
  data_classification: internal
  retention_class: STANDARD

allowed_models:
  - claude-sonnet-4-5
  - claude-opus-4-5
```

---

### legacy-assistant

Deprecated prompt retained for backward compatibility and migration testing.

```yaml schema prompt
id: legacy-assistant
version: "1.0.0"
type: system
description: Legacy assistant prompt maintained temporarily for migration compatibility
template: |
  You are the legacy assistant behavior profile.
  Keep responses concise and include a migration notice.

variables:
  migration_notice:
    type: string
    required: true
    minLength: 5
    maxLength: 500
    description: Notice shown to users when legacy prompt is selected

policy_tags:
  data_classification: internal
  retention_class: SHORT

allowed_models:
  - claude-sonnet-4-5

deprecated: true
deprecated_versions:
  - "1.0.0"
migration_guide: "docs/templates/schema-migration-guide.md"
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
