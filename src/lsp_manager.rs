use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageKind {
    Rust,
    TypeScript,
    Python,
    Go,
    Java,
}

impl fmt::Display for LanguageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageKind::Rust => write!(f, "rust"),
            LanguageKind::TypeScript => write!(f, "typescript"),
            LanguageKind::Python => write!(f, "python"),
            LanguageKind::Go => write!(f, "go"),
            LanguageKind::Java => write!(f, "java"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub language: LanguageKind,
    pub adapter: String,
    pub running: bool,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SessionStatus {
    pub fn is_running(&self) -> bool {
        self.running
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum LspSessionError {
    AdapterNotRegistered { language: LanguageKind },
    SessionAlreadyRunning { language: LanguageKind },
    SessionNotRunning { language: LanguageKind },
    AdapterNotFound { adapter: String },
}

pub struct LspSessionManager {
    adapters: HashMap<LanguageKind, String>,
    sessions: HashMap<LanguageKind, SessionStatus>,
}

impl LspSessionManager {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    pub async fn register_adapter(
        &mut self,
        language: LanguageKind,
        adapter: &str,
    ) -> Result<(), LspSessionError> {
        self.adapters.insert(language, adapter.to_string());
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<LanguageKind> {
        self.sessions.keys().cloned().collect()
    }

    pub async fn start_session(&mut self, language: LanguageKind) -> Result<(), LspSessionError> {
        let adapter = self
            .adapters
            .get(&language)
            .ok_or(LspSessionError::AdapterNotRegistered { language })?;

        if let Some(session) = self.sessions.get(&language) {
            if session.running {
                return Err(LspSessionError::SessionAlreadyRunning { language });
            }
        }

        let status = SessionStatus {
            language,
            adapter: adapter.clone(),
            running: true,
            started_at: Some(chrono::Utc::now()),
        };

        self.sessions.insert(language, status);

        Ok(())
    }

    pub async fn stop_session(&mut self, language: LanguageKind) -> Result<(), LspSessionError> {
        let session = self
            .sessions
            .get_mut(&language)
            .ok_or(LspSessionError::SessionNotRunning { language })?;

        if !session.running {
            return Err(LspSessionError::SessionNotRunning { language });
        }

        session.running = false;
        session.started_at = None;

        Ok(())
    }

    pub fn session_status(&self, language: LanguageKind) -> Result<SessionStatus, LspSessionError> {
        self.sessions
            .get(&language)
            .cloned()
            .ok_or(LspSessionError::SessionNotRunning { language })
    }
}

impl Default for LspSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_kind_display() {
        assert_eq!(LanguageKind::Rust.to_string(), "rust");
        assert_eq!(LanguageKind::TypeScript.to_string(), "typescript");
    }

    #[test]
    fn test_session_status_is_running() {
        let status = SessionStatus {
            language: LanguageKind::Rust,
            adapter: "rust-analyzer".to_string(),
            running: true,
            started_at: Some(chrono::Utc::now()),
        };
        assert!(status.is_running());
    }
}
