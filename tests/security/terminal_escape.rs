use chroma_ai_dev::generated;
use chroma_ai_dev::terminal_safety::sanitize_terminal_output;
use serde_json::Value;

#[test]
fn blocks_osc52_clipboard_injection() {
    let malicious_output = "Data: \x1b]52;c;bWFsaWNpb3VzX2RhdGE=\x1b\\ (stolen)";
    let sanitized = sanitize_terminal_output(malicious_output);

    assert!(!sanitized.contains("\x1b]52"));
    assert!(!sanitized.contains("bWFsaWNpb3VzX2RhdGE"));
}

#[test]
fn blocks_cursor_movement() {
    let malicious_output = "Visible\x1b[A\x1b[2KHidden override";
    let sanitized = sanitize_terminal_output(malicious_output);

    assert!(!sanitized.contains("\x1b[A"));
    assert!(!sanitized.contains("\x1b[2K"));
}

#[test]
fn preserves_safe_ansi_colors() {
    let colored_output = "\x1b[32mGreen\x1b[0m Normal \x1b[1;31mBold Red\x1b[0m";
    let sanitized = sanitize_terminal_output(colored_output);

    assert!(sanitized.contains("\x1b[32m"));
    assert!(sanitized.contains("\x1b[0m"));
    assert!(sanitized.contains("\x1b[1;31m"));
}

#[test]
fn terminal_safety_eval_suite_passes() {
    let suite = generated::evals::all()
        .iter()
        .find(|schema| {
            schema.get("suite_id").and_then(Value::as_str) == Some("terminal-safety-suite")
        })
        .expect("terminal-safety-suite must be defined in EVALS.md");

    let cases = suite
        .get("cases")
        .and_then(Value::as_array)
        .expect("terminal-safety-suite must include cases");

    let mut pass_count = 0usize;

    for case in cases {
        let input = case
            .get("input")
            .and_then(|v| v.get("model_output"))
            .and_then(Value::as_str)
            .expect("case input.model_output must exist");

        let expected_outcome = case
            .get("expected_outcome")
            .expect("case expected_outcome must exist");

        let case_type = expected_outcome
            .get("type")
            .and_then(Value::as_str)
            .expect("expected_outcome.type must exist");

        let sanitized = sanitize_terminal_output(input);

        let passed = match case_type {
            "sanitized" => {
                expected_outcome
                    .get("contains_escape_sequences")
                    .and_then(Value::as_bool)
                    == Some(false)
                    && !sanitized.contains('\x1b')
            }
            "preserved" => expected_outcome
                .get("rendered_output")
                .and_then(Value::as_str)
                .map(|expected| sanitized == expected)
                .unwrap_or(false),
            _ => false,
        };

        if passed {
            pass_count += 1;
        }
    }

    let pass_rate = pass_count as f64 / cases.len() as f64;
    assert_eq!(pass_rate, 1.0, "terminal safety suite must pass 100%");
}
