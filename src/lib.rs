// ChromaAI Dev - Library crate
// Copyright (c) 2026 ChromaAI Dev Team

//! ChromaAI Dev - Terminal-first AI development, evaluation, and release tool
//!
//! This library provides the core functionality for ChromaAI Dev, including:
//! - Schema validation and code generation (from markdown files)
//! - Tool execution and validation
//! - Prompt rendering with variable substitution
//! - Evaluation suite execution
//! - Agent runtime configuration
//!
//! ## Architecture
//!
//! ChromaAI Dev follows a thin-client architecture:
//! - **Client**: TUI/CLI for user interaction, validates for UX
//! - **Control Plane**: Server-side policy enforcement, artifact storage, audit logging
//! - **Execution Plane**: Provider gateways, tool execution, retrieval services
//!
//! ## Generated Code
//!
//! The `generated` module contains Rust code automatically generated from schema
//! definitions in markdown files (TOOLS.md, PROMPTS.md, EVALS.md, etc.).
//!
//! Generation happens at build time via `build.rs`.

pub mod config;
pub mod docs_generation;
pub mod evals;
pub mod generated;
pub mod modes;
pub mod prompts;
pub mod schema_lint;
pub mod terminal_safety;
pub mod tools;
pub mod versioning;

/// Error types for ChromaAI Dev
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum ChromaError {
        #[error("Schema validation failed: {0}")]
        SchemaValidation(String),

        #[error("Tool execution failed: {tool} - {message}")]
        ToolExecution { tool: String, message: String },

        #[error("Policy denial: {reason}")]
        PolicyDenial { reason: String },

        #[error("Authentication failed: {0}")]
        AuthenticationFailed(String),

        #[error("Budget exceeded: limit={limit}, attempted={attempted}")]
        BudgetExceeded { limit: f64, attempted: f64 },

        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        #[error("Serialization error: {0}")]
        Serialization(#[from] serde_json::Error),

        #[error("Network error: {0}")]
        Network(#[from] reqwest::Error),

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }

    pub type Result<T> = std::result::Result<T, ChromaError>;
}

pub use error::{ChromaError, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::docs_generation;
    pub use crate::error::{ChromaError, Result};
    pub use crate::evals;
    pub use crate::generated;
    pub use crate::prompts;
    pub use crate::schema_lint;
    pub use crate::tools;
    pub use crate::versioning;
}
