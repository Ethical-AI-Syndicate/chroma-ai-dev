// ChromaAI Dev - Ticket ID generation and validation
// Copyright (c) 2026 ChromaAI Dev Team

use crate::tickets::TicketError;
use regex::Regex;
use std::sync::OnceLock;
use uuid::Uuid;

const ID_PREFIX: &str = "chr-";

fn id_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"^chr-[a-f0-9]{4,8}$").expect("valid regex"))
}

/// Generates a new ticket ID (chr- + 8 hex chars from UUID v4).
/// Collision risk is negligible for typical repo sizes.
pub fn generate_id() -> String {
    let u = Uuid::new_v4();
    let hex = u.as_simple().to_string();
    format!("{}{}", ID_PREFIX, &hex[..8])
}

/// Validates ticket ID format. Does not check existence.
pub fn validate_id(id: &str) -> Result<(), TicketError> {
    if id_pattern().is_match(id) {
        Ok(())
    } else {
        Err(TicketError::InvalidId { id: id.to_string() })
    }
}

/// Returns the file name for a ticket (id + .md).
pub fn ticket_filename(id: &str) -> String {
    format!("{}.md", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_id_matches_pattern() {
        for _ in 0..100 {
            let id = generate_id();
            assert!(
                id_pattern().is_match(&id),
                "id {} did not match pattern",
                id
            );
            assert!(id.starts_with("chr-"));
            assert_eq!(id.len(), ID_PREFIX.len() + 8);
        }
    }

    #[test]
    fn validate_id_accepts_valid() {
        assert!(validate_id("chr-a1b2c3d4").is_ok());
        assert!(validate_id("chr-12345678").is_ok());
        assert!(validate_id("chr-abcd").is_ok());
    }

    #[test]
    fn validate_id_rejects_invalid() {
        assert!(validate_id("bd-a1b2").is_err());
        assert!(validate_id("chr-").is_err());
        assert!(validate_id("chr-xyz").is_err());
        assert!(validate_id("chr-a1b2c3d4e").is_err()); // 9 hex
    }
}
