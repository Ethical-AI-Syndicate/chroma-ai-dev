---
schema_version: "1.0"
last_updated: "2026-02-23"
validated_by: build_system
status: draft
---

# EVALS.md - Evaluation Suite Definitions

This file defines evaluation suites with test cases, grading methods, and regression gates. Eval suites are used for:
1. Regression testing (block releases on failure)
2. Quality gates (enforce minimum standards)
3. A/B testing (compare prompt/model changes)
4. Continuous validation (run on every change)

**Purpose:** Source of truth for evaluation criteria and quality standards.

---

## Evaluation Suites

### policy-enforcement-suite

Critical regression suite validating policy enforcement works correctly.

```yaml schema eval
suite_id: policy-enforcement-suite
version: "1.0.0"
description: Validates that policy enforcement blocks unauthorized actions and allows authorized ones
severity: critical
cases:
  - case_id: deny-unprivileged-promote
    description: Developer role cannot promote to prod without ReleaseManager role
    input:
      actor_role: Developer
      action: promote_to_prod
      release_id: "rel-test-123"
      environment: prod
    expected_outcome:
      type: policy_denial
      reason_code: INSUFFICIENT_PRIVILEGES
      http_status: 403
    grading_method: deterministic

  - case_id: deny-missing-approvals
    description: ReleaseManager cannot promote without required approvals
    input:
      actor_role: ReleaseManager
      action: promote_to_prod
      release_id: "rel-test-123"
      approvals: []
      required_approvals: 2
    expected_outcome:
      type: policy_denial
      reason_code: MISSING_APPROVALS
      http_status: 403
    grading_method: deterministic

  - case_id: allow-privileged-promote-with-approvals
    description: ReleaseManager can promote with sufficient approvals
    input:
      actor_role: ReleaseManager
      action: promote_to_prod
      release_id: "rel-test-123"
      approvals: ["approver1@example.com", "approver2@example.com"]
      required_approvals: 2
    expected_outcome:
      type: success
      http_status: 200
    grading_method: deterministic

  - case_id: deny-high-risk-tool-in-prod
    description: High-risk tools forbidden in prod environment
    input:
      actor_role: Developer
      action: execute_run
      tools: ["execute_sql_query"]
      environment: prod
    expected_outcome:
      type: policy_denial
      reason_code: TOOL_NOT_ALLOWED_IN_ENV
      http_status: 403
    grading_method: deterministic

  - case_id: allow-high-risk-tool-in-dev
    description: High-risk tools allowed in dev environment with proper role
    input:
      actor_role: Developer
      action: execute_run
      tools: ["execute_sql_query"]
      environment: dev
    expected_outcome:
      type: success
      http_status: 200
    grading_method: deterministic

thresholds:
  pass_rate: 1.0  # 100% must pass
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

**Grading logic (deterministic):**
```rust
fn grade_policy_case(case: &EvalCase, actual_response: &ApiResponse) -> CaseResult {
    let expected = &case.expected_outcome;

    if expected.outcome_type == "policy_denial" {
        if actual_response.status != expected.http_status {
            return CaseResult::Fail(format!(
                "Expected status {}, got {}",
                expected.http_status,
                actual_response.status
            ));
        }

        if !actual_response.error_code.contains(&expected.reason_code) {
            return CaseResult::Fail(format!(
                "Expected reason code {}, got {}",
                expected.reason_code,
                actual_response.error_code
            ));
        }

        CaseResult::Pass
    } else if expected.outcome_type == "success" {
        if actual_response.status != expected.http_status {
            return CaseResult::Fail(format!(
                "Expected success ({}), got {}",
                expected.http_status,
                actual_response.status
            ));
        }

        CaseResult::Pass
    } else {
        CaseResult::Error("Unknown outcome type".to_string())
    }
}
```

---

### terminal-safety-suite

Critical security suite validating ANSI escape sequence sanitization.

```yaml schema eval
suite_id: terminal-safety-suite
version: "1.0.0"
description: Validates terminal escape sequence sanitization prevents injection attacks
severity: critical
cases:
  - case_id: block-osc52-clipboard-injection
    description: OSC 52 clipboard escape sequences must be completely removed
    input:
      model_output: "Here is your data: \x1b]52;c;bWFsaWNpb3VzX2RhdGE=\x1b\\ (clipboard hijacked)"
    expected_outcome:
      type: sanitized
      contains_escape_sequences: false
      contains_osc52: false
      preserved_text: "Here is your data:  (clipboard hijacked)"
    grading_method: deterministic

  - case_id: block-cursor-movement-overwrite
    description: Cursor movement sequences that could overwrite output must be removed
    input:
      model_output: "Visible line\x1b[A\x1b[2KHidden override"
    expected_outcome:
      type: sanitized
      contains_escape_sequences: false
      preserved_text_contains: "Visible line"
      preserved_text_contains: "Hidden override"
    grading_method: deterministic

  - case_id: block-screen-clear
    description: Screen clearing sequences must be removed
    input:
      model_output: "Before\x1b[2JAfter (screen cleared)"
    expected_outcome:
      type: sanitized
      contains_escape_sequences: false
      preserved_text: "BeforeAfter (screen cleared)"
    grading_method: deterministic

  - case_id: preserve-safe-ansi-colors
    description: Safe ANSI color codes should be preserved for UX
    input:
      model_output: "\x1b[32mGreen text\x1b[0m Normal \x1b[1;31mBold Red\x1b[0m"
    expected_outcome:
      type: preserved
      contains_safe_color_codes: true
      rendered_output: "\x1b[32mGreen text\x1b[0m Normal \x1b[1;31mBold Red\x1b[0m"
    grading_method: deterministic

  - case_id: preserve-basic-formatting
    description: Basic text formatting (bold, underline) should be preserved
    input:
      model_output: "\x1b[1mBold\x1b[0m and \x1b[4mUnderline\x1b[0m"
    expected_outcome:
      type: preserved
      contains_safe_formatting: true
      rendered_output: "\x1b[1mBold\x1b[0m and \x1b[4mUnderline\x1b[0m"
    grading_method: deterministic

  - case_id: block-title-change
    description: Terminal title change sequences must be removed
    input:
      model_output: "\x1b]0;Malicious Title\x07Normal text"
    expected_outcome:
      type: sanitized
      contains_escape_sequences: false
      preserved_text: "Normal text"
    grading_method: deterministic

thresholds:
  pass_rate: 1.0  # 100% must pass - security critical
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

**Implementation reference:**
```rust
fn sanitize_terminal_output(input: &str) -> String {
    // Remove dangerous sequences
    let without_osc52 = remove_osc52_sequences(input);
    let without_cursor = remove_cursor_movement(without_osc52);
    let without_clear = remove_screen_clear(without_cursor);
    let without_title = remove_title_change(without_clear);

    // Preserve safe ANSI codes (basic colors and formatting)
    without_title  // Already filtered to safe subset
}
```

---

### output-quality-suite

Quality evaluation using LLM-as-judge for subjective criteria.

```yaml schema eval
suite_id: output-quality-suite
version: "1.0.0"
description: Validates AI output quality using LLM-as-judge with repeated trials
severity: high

judge_config:
  model: claude-sonnet-4-5
  temperature: 0.0
  repeat_trials: 3
  variance_tolerance: 0.15  # 15% variance allowed between trials
  max_tokens: 1000

cases:
  - case_id: summarization-accuracy-and-brevity
    description: Summary must contain key points and respect length limit
    input:
      prompt_id: rag-query
      variables:
        query: "Summarize the key requirements"
        corpus_id: "product-spec"
        corpus_version: "v1.0"
        retrieved_docs:
          - content: |
              Requirement A: System must support SSO via OIDC.
              Requirement B: All actions must be audited with request_id.
              Requirement C: Budget enforcement in real-time.
              Implementation details: Use tokio for async, vault for secrets...
            source_id: "spec-section-3"
            score: 0.95

    expected_constraints:
      - type: contains_key_points
        key_points:
          - "SSO"
          - "OIDC"
          - "audit"
          - "request_id"
          - "budget"
      - type: length_limit
        max_words: 100
      - type: no_hallucination
        check_against_source: true

    grading_method: llm_judge
    judge_prompt: |
      Evaluate the AI's response against these criteria:

      1. **Key Points Coverage:** Does the summary mention SSO/OIDC, audit with request_id, and budget enforcement?
      2. **Brevity:** Is the summary under 100 words?
      3. **Accuracy:** Does it avoid adding information not present in the source document?

      Respond with:
      ```
      PASS - if all three criteria are met
      FAIL - if any criterion is not met

      Explanation: <brief justification>
      ```

thresholds:
  pass_rate: 0.90  # 90% of trials must pass
  max_failures: 1

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### schema-registry-consistency-suite

Deterministic checks for generated schema registry consistency.

```yaml schema eval
suite_id: schema-registry-consistency-suite
version: "1.0.0"
description: Validates schema registry loading and versioned lookups behave consistently
severity: medium

cases:
  - case_id: tool-registry-non-empty
    description: Tool registry should load at least one tool schema
    input:
      prompt_id: default-assistant
      check: tools_all_len
    expected_outcome:
      type: success
      min_count: 1
    grading_method: deterministic

  - case_id: prompt-registry-lookup
    description: Prompt lookup should resolve default-assistant v1.0.0
    input:
      prompt_id: default-assistant
      check: prompt_lookup
      version: "1.0.0"
    expected_outcome:
      type: success
      resolved: true
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### incident-update-quality-suite

LLM-as-judge suite for structured incident update quality.

```yaml schema eval
suite_id: incident-update-quality-suite
version: "1.0.0"
description: Evaluates clarity and actionability of incident response updates
severity: high

judge_config:
  model: claude-sonnet-4-5
  temperature: 0.0
  repeat_trials: 3
  variance_tolerance: 0.20
  max_tokens: 1200

cases:
  - case_id: includes-owner-and-next-step
    description: Incident update must include owner, impact, and next update timing
    input:
      prompt_id: incident-response-update
      variables:
        incident_id: INC-2026-0142
        severity: sev2
        status: investigating
        impact_summary: "Users intermittently cannot authenticate via OIDC."
        mitigations:
          - "Scaled auth workers"
          - "Enabled fallback provider"
        next_update_eta_minutes: 30
    expected_constraints:
      - type: includes_fields
        required:
          - owner
          - timeline
          - impact
          - next_update
      - type: actionability
        minimum_score: 0.8
    grading_method: llm_judge
    judge_prompt: |
      Evaluate whether the incident update is operationally useful.

      Criteria:
      1) Includes clear owner and timeline
      2) States user-facing impact
      3) Includes specific next update timing

      Respond with PASS or FAIL and one-sentence explanation.

thresholds:
  pass_rate: 0.85
  max_failures: 1

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

**LLM-as-judge grading logic:**
```rust
async fn grade_with_llm_judge(
    case: &EvalCase,
    actual_output: &str,
    judge_config: &JudgeConfig
) -> Result<f64> {
    let mut pass_count = 0;

    for trial in 0..judge_config.repeat_trials {
        let judge_response = claude_client.send_request(ClaudeRequest {
            model: judge_config.model.clone(),
            messages: vec![
                Message {
                    role: "user",
                    content: format!("{}\n\nAI Output to evaluate:\n{}",
                        case.judge_prompt, actual_output),
                },
            ],
            temperature: Some(judge_config.temperature),
            max_tokens: judge_config.max_tokens,
            ..Default::default()
        }).await?;

        let response_text = judge_response.content[0].text.to_lowercase();
        if response_text.starts_with("pass") {
            pass_count += 1;
        }
    }

    let pass_rate = pass_count as f64 / judge_config.repeat_trials as f64;

    // Check variance tolerance
    let variance = calculate_variance(pass_count, judge_config.repeat_trials);
    if variance > judge_config.variance_tolerance {
        warn!("High variance in LLM judge results: {}", variance);
    }

    Ok(pass_rate)
}
```

---

### mode-routing-suite

Deterministic suite validating mode-to-config routing invariants.

```yaml schema eval
suite_id: mode-routing-suite
version: "1.0.0"
description: Validates requested execution modes resolve to the expected agent config primitives
severity: high

cases:
  - case_id: single-shot-resolves-single-shot-config
    description: Single-shot mode should resolve mode_single_shot_defaults as active policy config
    input:
      requested_mode: single_shot
      expected_agent_config: mode_single_shot_defaults
      expected_version: "1.0.0"
    expected_outcome:
      type: success
      matched: true
    grading_method: deterministic

  - case_id: multi-turn-resolves-multi-turn-config
    description: Multi-turn mode should resolve mode_multi_turn_defaults for bounded conversation loops
    input:
      requested_mode: multi_turn
      expected_agent_config: mode_multi_turn_defaults
      expected_version: "1.0.0"
    expected_outcome:
      type: success
      matched: true
    grading_method: deterministic

  - case_id: parallel-orchestration-resolves-orchestration-config
    description: Parallel orchestration mode should resolve orchestration_parallel_defaults controls
    input:
      requested_mode: orchestrated_parallel
      expected_agent_config: orchestration_parallel_defaults
      expected_version: "1.0.0"
    expected_outcome:
      type: success
      matched: true
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### agent-mail-tool-contract-suite

Deterministic suite validating schema-level contracts for agent-mail tools.

```yaml schema eval
suite_id: agent-mail-tool-contract-suite
version: "1.0.0"
description: Validates all agent-mail tool schema contracts are executable and regression-blocking
severity: high

cases:
  - case_id: register-tool-contracts-load
    description: agent_mail_register should expose at least one passing contract test case
    input:
      tool_name: agent_mail_register
      expected_contract_tests_min: 1
    expected_outcome:
      type: success
      contract_tests_loaded: true
    grading_method: deterministic

  - case_id: send-message-tool-contracts-load
    description: agent_mail_send_message should expose at least one passing contract test case
    input:
      tool_name: agent_mail_send_message
      expected_contract_tests_min: 1
    expected_outcome:
      type: success
      contract_tests_loaded: true
    grading_method: deterministic

  - case_id: inbox-tool-contracts-load
    description: agent_mail_check_inbox should expose at least one passing contract test case
    input:
      tool_name: agent_mail_check_inbox
      expected_contract_tests_min: 1
    expected_outcome:
      type: success
      contract_tests_loaded: true
    grading_method: deterministic

  - case_id: reservation-tool-contracts-load
    description: agent_mail_reserve_file should expose at least one passing contract test case
    input:
      tool_name: agent_mail_reserve_file
      expected_contract_tests_min: 1
    expected_outcome:
      type: success
      contract_tests_loaded: true
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### mode-policy-suite

Deterministic suite validating mode transition policies.

```yaml schema eval
suite_id: mode-policy-suite
version: "1.0.0"
description: Validates mode transition rules and central policy enforcement
severity: critical

cases:
  - case_id: plan-to-build-allowed
    description: Plan to Build transition should be allowed by default policy
    input:
      from: plan
      to: build
    expected_outcome:
      type: success
      allowed: true
    grading_method: deterministic

  - case_id: plan-to-review-denied
    description: Plan to Review transition should be denied (must go through build)
    input:
      from: plan
      to: review
    expected_outcome:
      type: success
      allowed: false
      error_type: invalid_transition
    grading_method: deterministic

  - case_id: incident-mode-requires-auth
    description: Transition to incident mode requires elevated_by and reason
    input:
      to: incident
      missing_fields: [reason, elevated_by]
    expected_outcome:
      type: success
      allowed: false
      error_type: missing_required_field
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: restricted
  retention_class: STANDARD
```

---

### lease-conflict-suite

Deterministic suite validating inter-agent file lease coordination.

```yaml schema eval
suite_id: lease-conflict-suite
version: "1.0.0"
description: Validates file lease conflict detection and exclusive access invariants
severity: high

cases:
  - case_id: exclusive-lease-blocks-read
    description: Exclusive lease held by Agent A should block read lease for Agent B
    input:
      existing_lease: { agent_id: agent_a, path: "src/lib.rs", mode: exclusive }
      requested_lease: { agent_id: agent_b, path: "src/lib.rs", mode: read }
    expected_outcome:
      type: success
      allowed: false
      error_type: lease_conflict
    grading_method: deterministic

  - case_id: write-lease-blocks-exclusive
    description: Write lease held by Agent A should block exclusive lease for Agent B
    input:
      existing_lease: { agent_id: agent_a, path: "src/lib.rs", mode: write }
      requested_lease: { agent_id: agent_b, path: "src/lib.rs", mode: exclusive }
    expected_outcome:
      type: success
      allowed: false
      error_type: lease_conflict
    grading_method: deterministic

  - case_id: concurrent-read-allowed
    description: Multiple agents should be able to hold read leases on the same path
    input:
      existing_lease: { agent_id: agent_a, path: "src/lib.rs", mode: read }
      requested_lease: { agent_id: agent_b, path: "src/lib.rs", mode: read }
    expected_outcome:
      type: success
      allowed: true
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

### orchestrator-join-suite

Deterministic suite validating DAG join behavior and parallel execution.

```yaml schema eval
suite_id: orchestrator-join-suite
version: "1.0.0"
description: Validates DAG scheduling, parallel readiness, and deterministic join reduction
severity: high

cases:
  - case_id: independent-nodes-ready-concurrently
    description: Multiple nodes with no dependencies should all be ready at once
    input:
      nodes: [a, b, c]
      dependencies: {}
    expected_outcome:
      type: success
      ready_count: 3
    grading_method: deterministic

  - case_id: join-blocks-until-all-parents-done
    description: A join node should stay blocked until ALL parent dependencies complete
    input:
      nodes: [a, b, join]
      dependencies: { join: [a, b] }
      completed: [a]
    expected_outcome:
      type: success
      ready_nodes: [b]
      blocked_nodes: [join]
    grading_method: deterministic

  - case_id: failure-aborts-downstream
    description: Failure in a parent node should mark downstream nodes as blocked in fail-fast mode
    input:
      nodes: [a, b]
      dependencies: { b: [a] }
      failed: [a]
    expected_outcome:
      type: success
      node_statuses: { b: blocked }
    grading_method: deterministic

thresholds:
  pass_rate: 1.0
  max_failures: 0

regression_blocking: true

policy_tags:
  data_classification: internal
  retention_class: STANDARD
```

---

## Evaluation Execution

**Running eval suites:**
```rust
let suite = get_eval_suite("policy-enforcement-suite")?;
let result = run_eval_suite(&suite).await?;

if result.pass_rate < suite.thresholds.pass_rate {
    return Err(EvalError::RegressionDetected {
        suite_id: suite.suite_id,
        pass_rate: result.pass_rate,
        threshold: suite.thresholds.pass_rate,
    });
}
```

**Regression gating:**
- Eval suites marked `regression_blocking: true` MUST pass before promotion
- Promotion request denied if any blocking suite fails
- Eval run results stored as immutable artifacts
- Decision logged with request_id for audit

---

## Adding New Eval Suites

**Process:**

1. **Define test cases** with clear inputs and expected outcomes
2. **Choose grading method:** deterministic or llm_judge
3. **Set thresholds:** pass_rate and max_failures
4. **Mark regression_blocking** if quality gate
5. **Test the suite:** Run with known-good and known-bad inputs
6. **Add to this file** with schema block
7. **Validate:** Run `cargo build`

**Grading methods:**

- **Deterministic:** Exact matching, schema validation, status codes
  - Fast, reliable, no variance
  - Use for: policy checks, security tests, schema validation

- **LLM-as-judge:** Use LLM to evaluate subjective quality
  - Slower, some variance, requires careful prompt design
  - Use for: summarization quality, tone, helpfulness, completeness
  - MUST set repeat_trials >= 3 and variance_tolerance

---

## Changelog

### 1.0 (2026-02-23)
- Initial version
- Added policy-enforcement-suite (deterministic)
- Added terminal-safety-suite (deterministic, security critical)
- Added output-quality-suite (llm-judge example)

---

## Next Steps

- Add more deterministic suites (schema validation, tool contract tests)
- Add more LLM-judge suites (code quality, documentation quality)
- Define A/B testing evaluation patterns
- Add performance regression suites (latency, cost)
- Add accessibility evaluation suites (TUI rendering)
