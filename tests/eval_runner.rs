use chroma_ai_dev::evals::{
    run_deterministic_suite, run_llm_judge_suite, EvalExecutionError, EvalSuiteReport,
    LlmJudgeDecision,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn deterministic_policy_suite_passes_with_matching_outcomes() {
    let mut actual = HashMap::new();
    actual.insert(
        "deny-unprivileged-promote".to_string(),
        json!({"type":"policy_denial","reason_code":"INSUFFICIENT_PRIVILEGES","http_status":403}),
    );
    actual.insert(
        "deny-missing-approvals".to_string(),
        json!({"type":"policy_denial","reason_code":"MISSING_APPROVALS","http_status":403}),
    );
    actual.insert(
        "allow-privileged-promote-with-approvals".to_string(),
        json!({"type":"success","http_status":200}),
    );
    actual.insert(
        "deny-high-risk-tool-in-prod".to_string(),
        json!({"type":"policy_denial","reason_code":"TOOL_NOT_ALLOWED_IN_ENV","http_status":403}),
    );
    actual.insert(
        "allow-high-risk-tool-in-dev".to_string(),
        json!({"type":"success","http_status":200}),
    );

    let report = run_deterministic_suite("policy-enforcement-suite", "1.0.0", &actual)
        .expect("deterministic suite execution should succeed");

    assert!(report.passed);
    assert_eq!(report.failures, 0);
    assert_eq!(report.pass_rate, 1.0);
}

#[test]
fn deterministic_policy_suite_fails_on_mismatch() {
    let mut actual = HashMap::new();
    actual.insert(
        "deny-unprivileged-promote".to_string(),
        json!({"type":"policy_denial","reason_code":"WRONG_REASON","http_status":403}),
    );

    let report = run_deterministic_suite("policy-enforcement-suite", "1.0.0", &actual)
        .expect("deterministic suite should still return report for mismatches");

    assert!(!report.passed);
    assert!(report.failures >= 1);
}

#[test]
fn llm_judge_suite_respects_thresholds_and_repeats() {
    let report = run_llm_judge_suite(
        "output-quality-suite",
        "1.0.0",
        &HashMap::from([(
            "summarization-accuracy-and-brevity".to_string(),
            "summary text".to_string(),
        )]),
        |_case_id, _trial, _prompt, _output| LlmJudgeDecision {
            passed: true,
            explanation: "passes checks".to_string(),
        },
    )
    .expect("llm judge execution should succeed");

    assert!(report.passed);
    assert_eq!(report.failures, 0);
}

#[test]
fn llm_judge_suite_fails_when_trial_pass_rate_below_threshold() {
    let report = run_llm_judge_suite(
        "output-quality-suite",
        "1.0.0",
        &HashMap::from([(
            "summarization-accuracy-and-brevity".to_string(),
            "summary text".to_string(),
        )]),
        |_case_id, trial, _prompt, _output| {
            if trial < 2 {
                LlmJudgeDecision {
                    passed: true,
                    explanation: "passes checks".to_string(),
                }
            } else {
                LlmJudgeDecision {
                    passed: false,
                    explanation: "fails checks".to_string(),
                }
            }
        },
    )
    .expect("llm judge execution should succeed");

    assert!(
        !report.passed,
        "suite should fail because 2/3 trial pass rate is below threshold 0.90"
    );
}

#[test]
fn unknown_suite_returns_error() {
    let report = run_deterministic_suite("missing-suite", "1.0.0", &HashMap::new());
    match report {
        Err(EvalExecutionError::UnknownSuite { suite_id, version }) => {
            assert_eq!(suite_id, "missing-suite");
            assert_eq!(version, "1.0.0");
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn regression_blocking_reported_when_threshold_not_met() {
    let mut actual = HashMap::new();
    actual.insert(
        "deny-unprivileged-promote".to_string(),
        json!({"type":"success","http_status":200}),
    );

    let report: EvalSuiteReport =
        run_deterministic_suite("policy-enforcement-suite", "1.0.0", &actual)
            .expect("suite run should produce report");
    assert!(report.regression_blocking);
    assert!(!report.passed);
}
