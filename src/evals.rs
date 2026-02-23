use crate::generated;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct EvalCaseReport {
    pub case_id: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalSuiteReport {
    pub suite_id: String,
    pub version: String,
    pub pass_rate: f64,
    pub failures: usize,
    pub regression_blocking: bool,
    pub passed: bool,
    pub case_reports: Vec<EvalCaseReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmJudgeDecision {
    pub passed: bool,
    pub explanation: String,
}

#[derive(Debug, Error)]
pub enum EvalExecutionError {
    #[error("unknown eval suite {suite_id}@{version}")]
    UnknownSuite { suite_id: String, version: String },

    #[error("eval suite {suite_id}@{version} has no cases")]
    EmptySuite { suite_id: String, version: String },

    #[error("eval suite {suite_id}@{version} has no llm_judge cases")]
    NoLlmJudgeCases { suite_id: String, version: String },
}

pub fn run_deterministic_suite(
    suite_id: &str,
    version: &str,
    actual_outcomes: &HashMap<String, Value>,
) -> Result<EvalSuiteReport, EvalExecutionError> {
    let suite = find_suite(suite_id, version)?;
    let cases = suite
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| EvalExecutionError::EmptySuite {
            suite_id: suite_id.to_string(),
            version: version.to_string(),
        })?;

    let mut reports = Vec::new();
    for case in cases {
        if case.get("grading_method").and_then(Value::as_str) != Some("deterministic") {
            continue;
        }

        let case_id = case
            .get("case_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown-case")
            .to_string();

        let expected = case.get("expected_outcome");
        let actual = actual_outcomes.get(&case_id);

        let (passed, details) = match (expected, actual) {
            (Some(expected), Some(actual)) => {
                if is_subset(expected, actual) {
                    (true, "actual outcome matches expected subset".to_string())
                } else {
                    (
                        false,
                        format!(
                            "actual outcome mismatch. expected subset={}, actual={}",
                            expected, actual
                        ),
                    )
                }
            }
            (Some(_), None) => (false, "missing actual outcome for case".to_string()),
            (None, _) => (false, "case missing expected_outcome".to_string()),
        };

        reports.push(EvalCaseReport {
            case_id,
            passed,
            details,
        });
    }

    if reports.is_empty() {
        return Err(EvalExecutionError::EmptySuite {
            suite_id: suite_id.to_string(),
            version: version.to_string(),
        });
    }

    Ok(finalize_suite_report(suite_id, version, suite, reports))
}

pub fn run_llm_judge_suite<F>(
    suite_id: &str,
    version: &str,
    actual_outputs: &HashMap<String, String>,
    judge: F,
) -> Result<EvalSuiteReport, EvalExecutionError>
where
    F: Fn(&str, usize, &str, &str) -> LlmJudgeDecision,
{
    let suite = find_suite(suite_id, version)?;
    let cases = suite
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| EvalExecutionError::EmptySuite {
            suite_id: suite_id.to_string(),
            version: version.to_string(),
        })?;

    let repeat_trials = suite
        .get("judge_config")
        .and_then(|config| config.get("repeat_trials"))
        .and_then(Value::as_u64)
        .unwrap_or(1) as usize;
    let threshold_pass_rate = suite
        .get("thresholds")
        .and_then(|thresholds| thresholds.get("pass_rate"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0);

    let mut reports = Vec::new();
    for case in cases {
        if case.get("grading_method").and_then(Value::as_str) != Some("llm_judge") {
            continue;
        }

        let case_id = case
            .get("case_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown-case")
            .to_string();
        let prompt = case
            .get("judge_prompt")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let output = actual_outputs
            .get(&case_id)
            .map(|s| s.as_str())
            .unwrap_or_default();

        let mut trial_passes = 0usize;
        let mut explanations = Vec::with_capacity(repeat_trials.max(1));

        for trial in 0..repeat_trials.max(1) {
            let decision = judge(&case_id, trial, prompt, output);
            if decision.passed {
                trial_passes += 1;
            }
            explanations.push(decision.explanation);
        }

        let trials = repeat_trials.max(1);
        let trial_pass_rate = trial_passes as f64 / trials as f64;
        let passed = trial_pass_rate >= threshold_pass_rate;
        reports.push(EvalCaseReport {
            case_id,
            passed,
            details: format!(
                "llm judge passed {}/{} trials (rate {:.2}, threshold {:.2}): {}",
                trial_passes,
                trials,
                trial_pass_rate,
                threshold_pass_rate,
                explanations.join(" | ")
            ),
        });
    }

    if reports.is_empty() {
        return Err(EvalExecutionError::NoLlmJudgeCases {
            suite_id: suite_id.to_string(),
            version: version.to_string(),
        });
    }

    Ok(finalize_suite_report(suite_id, version, suite, reports))
}

fn find_suite(suite_id: &str, version: &str) -> Result<&'static Value, EvalExecutionError> {
    generated::evals::find_by_name_and_version(suite_id, version).ok_or_else(|| {
        EvalExecutionError::UnknownSuite {
            suite_id: suite_id.to_string(),
            version: version.to_string(),
        }
    })
}

fn finalize_suite_report(
    suite_id: &str,
    version: &str,
    suite: &Value,
    case_reports: Vec<EvalCaseReport>,
) -> EvalSuiteReport {
    let total = case_reports.len();
    let failures = case_reports.iter().filter(|report| !report.passed).count();
    let pass_rate = if total == 0 {
        0.0
    } else {
        (total - failures) as f64 / total as f64
    };

    let threshold_pass_rate = suite
        .get("thresholds")
        .and_then(|thresholds| thresholds.get("pass_rate"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0);
    let max_failures = suite
        .get("thresholds")
        .and_then(|thresholds| thresholds.get("max_failures"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let regression_blocking = suite
        .get("regression_blocking")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let passed = pass_rate >= threshold_pass_rate && failures <= max_failures;

    EvalSuiteReport {
        suite_id: suite_id.to_string(),
        version: version.to_string(),
        pass_rate,
        failures,
        regression_blocking,
        passed,
        case_reports,
    }
}

fn is_subset(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Object(expected_map), Value::Object(actual_map)) => {
            expected_map.iter().all(|(key, expected_value)| {
                actual_map
                    .get(key)
                    .is_some_and(|actual_value| is_subset(expected_value, actual_value))
            })
        }
        (Value::Array(expected_arr), Value::Array(actual_arr)) => {
            expected_arr.len() <= actual_arr.len()
                && expected_arr
                    .iter()
                    .zip(actual_arr.iter())
                    .all(|(expected_item, actual_item)| is_subset(expected_item, actual_item))
        }
        _ => expected == actual,
    }
}
