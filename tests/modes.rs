use chroma_ai_dev::modes::{
    transition_mode, AgentMode, ModeTransitionError, ModeTransitionRequest,
};

#[test]
fn test_valid_transition_plan_to_build() {
    let request = ModeTransitionRequest {
        target_mode: AgentMode::Build,
        reason: None,
        expires_at: None,
        elevated_by: None,
    };

    let result = transition_mode(AgentMode::Plan, request);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AgentMode::Build);
}

#[test]
fn test_valid_transition_build_to_review() {
    let request = ModeTransitionRequest {
        target_mode: AgentMode::Review,
        reason: None,
        expires_at: None,
        elevated_by: None,
    };

    let result = transition_mode(AgentMode::Build, request);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AgentMode::Review);
}

#[test]
fn test_valid_transition_review_to_plan() {
    let request = ModeTransitionRequest {
        target_mode: AgentMode::Plan,
        reason: None,
        expires_at: None,
        elevated_by: None,
    };

    let result = transition_mode(AgentMode::Review, request);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AgentMode::Plan);
}

#[test]
fn test_invalid_transition_plan_to_review_rejected() {
    let request = ModeTransitionRequest {
        target_mode: AgentMode::Review,
        reason: None,
        expires_at: None,
        elevated_by: None,
    };

    let result = transition_mode(AgentMode::Plan, request);
    assert!(result.is_err());
    match result.unwrap_err() {
        ModeTransitionError::InvalidTransition { .. } => (),
        e => panic!("Expected InvalidTransition error, got {:?}", e),
    }
}

#[test]
fn test_incident_mode_requires_reason() {
    let request = ModeTransitionRequest {
        target_mode: AgentMode::Incident,
        reason: None, // Missing reason
        expires_at: None,
        elevated_by: None,
    };

    let result = transition_mode(AgentMode::Plan, request);
    assert!(result.is_err());
    match result.unwrap_err() {
        ModeTransitionError::MissingRequiredField { field } => {
            assert_eq!(field, "reason");
        }
        e => panic!("Expected MissingRequiredField error, got {:?}", e),
    }
}

#[test]
fn test_incident_mode_requires_expiry() {
    use chrono::Utc;

    let request = ModeTransitionRequest {
        target_mode: AgentMode::Incident,
        reason: Some("security-breach".to_string()),
        expires_at: None, // Missing expiry
        elevated_by: None,
    };

    let result = transition_mode(AgentMode::Plan, request);
    assert!(result.is_err());
    match result.unwrap_err() {
        ModeTransitionError::MissingRequiredField { field } => {
            assert_eq!(field, "expires_at");
        }
        e => panic!("Expected MissingRequiredField error, got {:?}", e),
    }
}

#[test]
fn test_incident_mode_requires_elevated_by() {
    use chrono::Utc;

    let request = ModeTransitionRequest {
        target_mode: AgentMode::Incident,
        reason: Some("security-breach".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        elevated_by: None, // Missing elevated_by
    };

    let result = transition_mode(AgentMode::Plan, request);
    assert!(result.is_err());
    match result.unwrap_err() {
        ModeTransitionError::MissingRequiredField { field } => {
            assert_eq!(field, "elevated_by");
        }
        e => panic!("Expected MissingRequiredField error, got {:?}", e),
    }
}

#[test]
fn test_incident_mode_valid_transition() {
    use chrono::Utc;

    let request = ModeTransitionRequest {
        target_mode: AgentMode::Incident,
        reason: Some("security-breach".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        elevated_by: Some("incident-commander@example.com".to_string()),
    };

    let result = transition_mode(AgentMode::Plan, request);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AgentMode::Incident);
}
