// ChromaAI Dev - Ticket types and schema
// Copyright (c) 2026 ChromaAI Dev Team

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// --- Errors -----------------------------------------------------------------

#[derive(Debug, Error)]
pub enum TicketError {
    #[error("ticket not found: {id}")]
    NotFound { id: String },

    #[error("invalid ticket id '{id}': must match chr-[a-f0-9]{{4,8}}")]
    InvalidId { id: String },

    #[error("invalid ticket type '{value}'; must be one of: task, bug, epic, story, meta")]
    InvalidType { value: String },

    #[error("invalid ticket status '{value}'; must be one of: open, in_progress, done, cancelled")]
    InvalidStatus { value: String },

    #[error("invalid priority {value}; must be 0-3")]
    InvalidPriority { value: i64 },

    #[error("issues directory not found; run 'chroma tickets init' from repo root")]
    IssuesDirNotFound,

    #[error("failed to parse ticket file {path}: {message}")]
    ParseError { path: String, message: String },

    #[error("failed to write ticket file {path}: {source}")]
    WriteError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("duplicate ticket id: {id}")]
    DuplicateId { id: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

// --- Enums -------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TicketType {
    #[default]
    Task,
    Bug,
    Epic,
    Story,
    Meta,
}

impl fmt::Display for TicketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TicketType::Task => write!(f, "task"),
            TicketType::Bug => write!(f, "bug"),
            TicketType::Epic => write!(f, "epic"),
            TicketType::Story => write!(f, "story"),
            TicketType::Meta => write!(f, "meta"),
        }
    }
}

impl std::str::FromStr for TicketType {
    type Err = TicketError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "task" => Ok(TicketType::Task),
            "bug" => Ok(TicketType::Bug),
            "epic" => Ok(TicketType::Epic),
            "story" => Ok(TicketType::Story),
            "meta" => Ok(TicketType::Meta),
            _ => Err(TicketError::InvalidType {
                value: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    #[default]
    Open,
    InProgress,
    Done,
    Cancelled,
}

impl fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TicketStatus::Open => write!(f, "open"),
            TicketStatus::InProgress => write!(f, "in_progress"),
            TicketStatus::Done => write!(f, "done"),
            TicketStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TicketStatus {
    type Err = TicketError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(TicketStatus::Open),
            "in_progress" | "in progress" => Ok(TicketStatus::InProgress),
            "done" | "closed" => Ok(TicketStatus::Done),
            "cancelled" | "canceled" => Ok(TicketStatus::Cancelled),
            _ => Err(TicketError::InvalidStatus {
                value: s.to_string(),
            }),
        }
    }
}

/// Priority: 0 = critical, 1 = high, 2 = medium, 3 = low.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TicketPriority(pub u8);

impl Default for TicketPriority {
    fn default() -> Self {
        TicketPriority(2)
    }
}

impl TicketPriority {
    pub const CRITICAL: u8 = 0;
    pub const HIGH: u8 = 1;
    pub const MEDIUM: u8 = 2;
    pub const LOW: u8 = 3;

    pub fn from_i64(v: i64) -> Result<Self, TicketError> {
        let u = u8::try_from(v).map_err(|_| TicketError::InvalidPriority { value: v })?;
        if u <= 3 {
            Ok(TicketPriority(u))
        } else {
            Err(TicketError::InvalidPriority {
                value: i64::from(u),
            })
        }
    }

    pub fn as_u8(self) -> u8 {
        self.0
    }
}

impl fmt::Display for TicketPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for TicketPriority {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Int {
            I64(i64),
            U8(u8),
        }
        let n = Int::deserialize(deserializer)?;
        let i = match n {
            Int::I64(v) => v,
            Int::U8(v) => i64::from(v),
        };
        TicketPriority::from_i64(i).map_err(de::Error::custom)
    }
}

// --- Ticket ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub r#type: TicketType,
    #[serde(default)]
    pub status: TicketStatus,
    #[serde(default)]
    pub priority: TicketPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relates_to: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_from: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_plan: Option<String>,

    /// Optional body after frontmatter (notes); not in canonical schema, for display.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub body: Option<String>,
}

impl Ticket {
    /// Returns true if this ticket is considered "open" for ready-work (open or in_progress).
    pub fn is_actionable(&self) -> bool {
        matches!(self.status, TicketStatus::Open | TicketStatus::InProgress)
    }

    /// Returns true if this ticket is closed (done or cancelled).
    pub fn is_closed(&self) -> bool {
        matches!(self.status, TicketStatus::Done | TicketStatus::Cancelled)
    }
}
