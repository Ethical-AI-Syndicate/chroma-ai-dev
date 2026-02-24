use chroma_ai_dev::control_plane::{ControlPlane, ControlPlaneError};
use chroma_ai_dev::modes::AgentMode;

#[test]
fn test_control_plane_creation() {
    let cp = ControlPlane::new();
    assert!(cp.get_policy_decisions().is_empty());
}

#[test]
fn test_policy_enforcement_deny_mode_transition() {
    let mut cp = ControlPlane::new();

    cp.set_policy_mode("strict");

    let decision = cp.check_mode_transition(AgentMode::Plan, AgentMode::Review);

    assert!(decision.is_denied());
    assert!(!decision.decision_id().is_empty());
}

#[test]
fn test_policy_enforcement_allow_tool() {
    let mut cp = ControlPlane::new();

    cp.set_policy_mode("permissive");

    let decision = cp.check_tool_permission("test-agent", "web_search");

    assert!(decision.is_allowed());
    assert!(!decision.decision_id().is_empty());
}

#[test]
fn test_budget_enforcement_hard_stop() {
    let mut cp = ControlPlane::new();

    cp.set_budget_limit(0.01);

    let result = cp.consume_budget(0.02);

    assert!(result.is_err());
    match result.unwrap_err() {
        ControlPlaneError::BudgetExceeded { .. } => (),
        e => panic!("Expected BudgetExceeded error, got {:?}", e),
    }
}

#[test]
fn test_audit_event_emission() {
    let mut cp = ControlPlane::new();

    cp.record_audit_event(
        "test-agent",
        "mode_transition",
        serde_json::json!({
            "from": "plan",
            "to": "build"
        }),
    );

    let events = cp.get_audit_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "mode_transition");
}

#[test]
fn test_policy_decision_id_generation() {
    let cp = ControlPlane::new();

    let decision1 = cp.check_tool_permission("agent1", "read_file");
    let decision2 = cp.check_tool_permission("agent2", "write_file");

    assert_ne!(decision1.decision_id(), decision2.decision_id());
}
