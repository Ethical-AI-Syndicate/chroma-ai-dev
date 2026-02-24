use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Plan,
    Build,
    Review,
    Incident,
}

impl std::fmt::Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentMode::Plan => write!(f, "plan"),
            AgentMode::Build => write!(f, "build"),
            AgentMode::Review => write!(f, "review"),
            AgentMode::Incident => write!(f, "incident"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransitionRequest {
    pub target_mode: AgentMode,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub elevated_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum ModeTransitionError {
    InvalidTransition {
        from: AgentMode,
        to: AgentMode,
        message: String,
    },
    MissingRequiredField {
        field: String,
    },
}

const VALID_TRANSITIONS: &[(AgentMode, AgentMode)] = &[
    (AgentMode::Plan, AgentMode::Build),
    (AgentMode::Build, AgentMode::Review),
    (AgentMode::Review, AgentMode::Plan),
    (AgentMode::Plan, AgentMode::Incident),
    (AgentMode::Build, AgentMode::Incident),
    (AgentMode::Review, AgentMode::Incident),
];

pub fn transition_mode(
    current: AgentMode,
    request: ModeTransitionRequest,
) -> Result<AgentMode, ModeTransitionError> {
    let target = request.target_mode;

    if current == target {
        return Ok(target);
    }

    let is_valid = VALID_TRANSITIONS
        .iter()
        .any(|(from, to)| *from == current && *to == target);

    if !is_valid {
        return Err(ModeTransitionError::InvalidTransition {
            from: current,
            to: target,
            message: format!(
                "Cannot transition from {:?} to {:?}. Valid transitions: Plan->Build, Build->Review, Review->Plan, any->Incident",
                current, target
            ),
        });
    }

    if target == AgentMode::Incident {
        if request.reason.is_none() {
            return Err(ModeTransitionError::MissingRequiredField {
                field: "reason".to_string(),
            });
        }
        if request.expires_at.is_none() {
            return Err(ModeTransitionError::MissingRequiredField {
                field: "expires_at".to_string(),
            });
        }
        if request.elevated_by.is_none() {
            return Err(ModeTransitionError::MissingRequiredField {
                field: "elevated_by".to_string(),
            });
        }
    }

    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(AgentMode::Plan.to_string(), "plan");
        assert_eq!(AgentMode::Build.to_string(), "build");
        assert_eq!(AgentMode::Review.to_string(), "review");
        assert_eq!(AgentMode::Incident.to_string(), "incident");
    }
}
