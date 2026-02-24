use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LeaseMode {
    #[default]
    Read,
    Write,
    Exclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub registered: bool,
    pub mailbox_id: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxMessage {
    pub id: String,
    pub sender_id: String,
    pub thread_id: String,
    pub message: String,
    pub priority: Option<String>,
    pub sent_at: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLease {
    pub lease_id: String,
    pub path: String,
    pub mode: LeaseMode,
    pub agent_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum AgentMailError {
    AgentNotFound {
        agent_id: String,
    },
    MessageNotFound {
        message_id: String,
    },
    LeaseNotFound {
        lease_id: String,
    },
    LeaseConflict {
        path: String,
        existing_agent: String,
    },
    InvalidLeaseMode,
}

pub struct AgentMailer {
    agents: HashMap<String, Registration>,
    messages: HashMap<String, MailboxMessage>,
    leases: HashMap<String, FileLease>,
    message_index: HashMap<String, Vec<String>>,
}

impl AgentMailer {
    pub async fn new_in_memory() -> Self {
        Self {
            agents: HashMap::new(),
            messages: HashMap::new(),
            leases: HashMap::new(),
            message_index: HashMap::new(),
        }
    }

    pub async fn register_agent(
        &mut self,
        agent_id: &str,
        _display_name: &str,
    ) -> Result<Registration, AgentMailError> {
        let mailbox_id = Uuid::new_v4().to_string();
        let registered_at = Utc::now();

        let registration = Registration {
            registered: true,
            mailbox_id: mailbox_id.clone(),
            registered_at,
        };

        self.agents
            .insert(agent_id.to_string(), registration.clone());

        Ok(registration)
    }

    pub async fn send_message(
        &mut self,
        recipient_id: &str,
        thread_id: &str,
        message: &str,
        priority: Option<String>,
    ) -> Result<String, AgentMailError> {
        if !self.agents.contains_key(recipient_id) {
            return Err(AgentMailError::AgentNotFound {
                agent_id: recipient_id.to_string(),
            });
        }

        let msg_id = Uuid::new_v4().to_string();
        let mailbox_message = MailboxMessage {
            id: msg_id.clone(),
            sender_id: "system".to_string(),
            thread_id: thread_id.to_string(),
            message: message.to_string(),
            priority,
            sent_at: Utc::now(),
            acknowledged: false,
        };

        self.messages.insert(msg_id.clone(), mailbox_message);

        self.message_index
            .entry(recipient_id.to_string())
            .or_default()
            .push(msg_id.clone());

        Ok(msg_id)
    }

    pub async fn fetch_inbox(
        &self,
        agent_id: &str,
        since: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<Vec<MailboxMessage>, AgentMailError> {
        if !self.agents.contains_key(agent_id) {
            return Err(AgentMailError::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        }

        let message_ids = self
            .message_index
            .get(agent_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let mut results: Vec<MailboxMessage> = message_ids
            .iter()
            .filter_map(|id| self.messages.get(id).cloned())
            .filter(|msg| {
                if let Some(since_time) = since {
                    msg.sent_at > since_time
                } else {
                    true
                }
            })
            .collect();

        results.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));

        results.truncate(limit);

        Ok(results)
    }

    pub async fn ack_message(
        &mut self,
        agent_id: &str,
        message_id: &str,
    ) -> Result<(), AgentMailError> {
        if !self.agents.contains_key(agent_id) {
            return Err(AgentMailError::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        }

        let message =
            self.messages
                .get_mut(message_id)
                .ok_or_else(|| AgentMailError::MessageNotFound {
                    message_id: message_id.to_string(),
                })?;

        message.acknowledged = true;

        Ok(())
    }

    pub async fn claim_file_lease(
        &mut self,
        agent_id: &str,
        path: &str,
        lease_seconds: u32,
        mode: LeaseMode,
    ) -> Result<FileLease, AgentMailError> {
        if !self.agents.contains_key(agent_id) {
            return Err(AgentMailError::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        }

        for lease in self.leases.values() {
            if lease.path == path
                && (lease.mode == LeaseMode::Exclusive || mode == LeaseMode::Exclusive)
            {
                return Err(AgentMailError::LeaseConflict {
                    path: path.to_string(),
                    existing_agent: lease.agent_id.clone(),
                });
            }
        }

        let lease_id = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(lease_seconds as i64);

        let lease = FileLease {
            lease_id: lease_id.clone(),
            path: path.to_string(),
            mode,
            agent_id: agent_id.to_string(),
            expires_at,
        };

        self.leases.insert(lease_id, lease.clone());

        Ok(lease)
    }

    pub async fn release_file_lease(
        &mut self,
        agent_id: &str,
        lease_id: &str,
    ) -> Result<(), AgentMailError> {
        let lease = self
            .leases
            .get(lease_id)
            .ok_or_else(|| AgentMailError::LeaseNotFound {
                lease_id: lease_id.to_string(),
            })?;

        if lease.agent_id != agent_id {
            return Err(AgentMailError::LeaseNotFound {
                lease_id: lease_id.to_string(),
            });
        }

        self.leases.remove(lease_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_mailer_creation() {
        let mailer = AgentMailer::new_in_memory().await;
        assert!(mailer.agents.is_empty());
    }
}
