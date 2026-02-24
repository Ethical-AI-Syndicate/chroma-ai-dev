use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

use crate::modes::AgentMode;

/// Maximum audit events to retain in memory (prevents unbounded growth)
const MAX_AUDIT_EVENTS: usize = 10_000;

/// Maximum policy decisions to retain in memory (prevents unbounded growth)
const MAX_POLICY_DECISIONS: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    decision_id: String,
    allowed: bool,
    reason: String,
    timestamp: DateTime<Utc>,
}

impl PolicyDecision {
    pub fn allow(reason: &str) -> Self {
        Self {
            decision_id: Uuid::new_v4().to_string(),
            allowed: true,
            reason: reason.to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn deny(reason: &str) -> Self {
        Self {
            decision_id: Uuid::new_v4().to_string(),
            allowed: false,
            reason: reason.to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn is_allowed(&self) -> bool {
        self.allowed
    }

    pub fn is_denied(&self) -> bool {
        !self.allowed
    }

    pub fn decision_id(&self) -> &str {
        &self.decision_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetState {
    limit: f64,
    spent: f64,
    remaining: f64,
}

impl BudgetState {
    pub fn new(limit: f64) -> Self {
        Self {
            limit,
            spent: 0.0,
            remaining: limit,
        }
    }

    pub fn consume(&mut self, amount: f64) -> Result<(), ControlPlaneError> {
        if self.remaining < amount {
            return Err(ControlPlaneError::BudgetExceeded {
                limit: self.limit,
                requested: amount,
                available: self.remaining,
            });
        }
        self.spent += amount;
        self.remaining -= amount;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum ControlPlaneError {
    BudgetExceeded {
        limit: f64,
        requested: f64,
        available: f64,
    },
    PolicyDenial {
        decision_id: String,
        reason: String,
    },
    UnauthorizedOperation {
        actor: String,
        operation: String,
    },
}

pub struct ControlPlane {
    policy_mode: String,
    budget: BudgetState,
    audit_events: VecDeque<AuditEvent>,
    policy_decisions: VecDeque<PolicyDecision>,
    tool_allowlist: HashMap<String, Vec<String>>,
}

impl ControlPlane {
    pub fn new() -> Self {
        Self {
            policy_mode: "standard".to_string(),
            budget: BudgetState::new(100.0),
            audit_events: VecDeque::with_capacity(MAX_AUDIT_EVENTS),
            policy_decisions: VecDeque::with_capacity(MAX_POLICY_DECISIONS),
            tool_allowlist: HashMap::new(),
        }
    }

    pub fn set_policy_mode(&mut self, mode: &str) {
        self.policy_mode = mode.to_string();
    }

    pub fn set_budget_limit(&mut self, limit: f64) {
        self.budget = BudgetState::new(limit);
    }

    pub fn check_mode_transition(&self, _from: AgentMode, to: AgentMode) -> PolicyDecision {
        if self.policy_mode == "strict" && to == AgentMode::Review {
            PolicyDecision::deny("Strict mode prevents direct transition to Review")
        } else {
            PolicyDecision::allow("Transition allowed")
        }
    }

    pub fn check_tool_permission(&self, agent_id: &str, tool: &str) -> PolicyDecision {
        if let Some(allowed_tools) = self.tool_allowlist.get(agent_id) {
            if allowed_tools.contains(&tool.to_string()) {
                return PolicyDecision::allow("Tool in agent allowlist");
            }
        }

        if self.policy_mode == "permissive" {
            PolicyDecision::allow("Permissive mode allows all tools")
        } else {
            PolicyDecision::deny("Tool not in allowlist")
        }
    }

    pub fn consume_budget(&mut self, amount: f64) -> Result<(), ControlPlaneError> {
        self.budget.consume(amount)
    }

    pub fn record_audit_event(&mut self, actor_id: &str, action: &str, details: serde_json::Value) {
        let event = AuditEvent {
            id: Uuid::new_v4().to_string(),
            actor_id: actor_id.to_string(),
            action: action.to_string(),
            details,
            timestamp: Utc::now(),
        };
        // Bounded push to prevent unbounded memory growth
        if self.audit_events.len() >= MAX_AUDIT_EVENTS {
            self.audit_events.pop_front();
        }
        self.audit_events.push_back(event);
    }

    pub fn get_audit_events(&self) -> Vec<AuditEvent> {
        self.audit_events.iter().cloned().collect()
    }

    pub fn get_policy_decisions(&self) -> Vec<PolicyDecision> {
        self.policy_decisions.iter().cloned().collect()
    }

    pub fn add_tool_allowlist(&mut self, agent_id: &str, tools: Vec<String>) {
        self.tool_allowlist.insert(agent_id.to_string(), tools);
    }
}

impl Default for ControlPlane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_state_creation() {
        let budget = BudgetState::new(10.0);
        assert_eq!(budget.limit, 10.0);
        assert_eq!(budget.spent, 0.0);
        assert_eq!(budget.remaining, 10.0);
    }

    #[test]
    fn test_budget_consume() {
        let mut budget = BudgetState::new(10.0);
        assert!(budget.consume(5.0).is_ok());
        assert_eq!(budget.remaining, 5.0);
    }

    #[test]
    fn test_budget_exceeded() {
        let mut budget = BudgetState::new(10.0);
        let result = budget.consume(15.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_policy_decision_allow() {
        let decision = PolicyDecision::allow("test");
        assert!(decision.is_allowed());
        assert!(!decision.is_denied());
    }

    #[test]
    fn test_policy_decision_deny() {
        let decision = PolicyDecision::deny("test");
        assert!(decision.is_denied());
        assert!(!decision.is_allowed());
    }
}
