// ChromaAI Dev - File-based ticket store
// Copyright (c) 2026 ChromaAI Dev Team

use crate::tickets::id::{generate_id, ticket_filename, validate_id};
use crate::tickets::types::{Ticket, TicketPriority, TicketStatus, TicketType};
use crate::tickets::TicketError;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CHROMA_DIR: &str = ".chroma";
const ISSUES_DIR: &str = "issues";
const FRONTMATTER_DELIM: &str = "---";

/// Returns git repository root containing `path`, or None if not in a git repo.
fn git_root(path: &Path) -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let root = std::str::from_utf8(out.stdout.as_slice()).ok()?.trim();
    if root.is_empty() {
        return None;
    }
    Some(PathBuf::from(root))
}

/// File-based ticket store under `.chroma/issues/`.
pub struct TicketStore {
    issues_dir: PathBuf,
}

impl TicketStore {
    /// Finds the issues directory by walking up from `start` until `.chroma/issues` exists.
    /// Returns error if not found.
    pub fn find(start: &Path) -> Result<Self, TicketError> {
        let mut current = start.to_path_buf();
        loop {
            let chroma = current.join(CHROMA_DIR);
            let issues = chroma.join(ISSUES_DIR);
            if issues.is_dir() {
                return Ok(Self { issues_dir: issues });
            }
            if !current.pop() {
                return Err(TicketError::IssuesDirNotFound);
            }
        }
    }

    /// Finds the issues directory, or initializes it at git root (or `start`) if not found.
    /// Use this so the application automatically sets up issue tracking on first use.
    pub fn find_or_init(start: &Path) -> Result<Self, TicketError> {
        match Self::find(start) {
            Ok(store) => Ok(store),
            Err(TicketError::IssuesDirNotFound) => {
                let root = git_root(start).unwrap_or_else(|| start.to_path_buf());
                Self::init(&root)
            }
            Err(e) => Err(e),
        }
    }

    /// Ensures `.chroma/issues` exists under `repo_root`, creating it if needed.
    /// Use when initializing or when creating the first ticket from a given root.
    pub fn init(repo_root: &Path) -> Result<Self, TicketError> {
        let issues_dir = repo_root.join(CHROMA_DIR).join(ISSUES_DIR);
        std::fs::create_dir_all(&issues_dir)?;
        Ok(Self { issues_dir })
    }

    /// Opens store at the given issues directory (for tests).
    #[cfg(test)]
    pub fn at(path: PathBuf) -> Self {
        Self { issues_dir: path }
    }

    pub fn issues_dir(&self) -> &Path {
        &self.issues_dir
    }

    /// Loads all tickets from the issues directory. Ignores non-.md files and invalid files.
    /// Returns only successfully parsed tickets; logs or skips parse errors.
    pub fn load_all(&self) -> Result<Vec<Ticket>, TicketError> {
        let mut tickets = Vec::new();
        let read_dir = std::fs::read_dir(&self.issues_dir)?;
        for entry in read_dir {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            if let Some(stem) = path.file_stem() {
                let stem = stem.to_string_lossy();
                if stem == "index" || stem.starts_with('.') {
                    continue;
                }
                if let Ok(t) = self.load_one(stem.as_ref()) {
                    tickets.push(t);
                }
            }
        }
        Ok(tickets)
    }

    /// Loads a single ticket by id.
    pub fn load_one(&self, id: &str) -> Result<Ticket, TicketError> {
        validate_id(id)?;
        let path = self.issues_dir.join(ticket_filename(id));
        let content = std::fs::read_to_string(&path)
            .map_err(|_| TicketError::NotFound { id: id.to_string() })?;
        parse_ticket_file(&content, &path).map_err(|msg| TicketError::ParseError {
            path: path.display().to_string(),
            message: msg,
        })
    }

    /// Saves a ticket to disk. Overwrites existing file.
    pub fn save(&self, ticket: &Ticket) -> Result<(), TicketError> {
        validate_id(&ticket.id)?;
        let path = self.issues_dir.join(ticket_filename(&ticket.id));
        let content = format_ticket_file(ticket);
        std::fs::write(&path, content).map_err(|e| TicketError::WriteError {
            path: path.display().to_string(),
            source: e,
        })?;
        Ok(())
    }

    /// Creates a new ticket with generated id. Sets created_at and updated_at.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        title: String,
        r#type: TicketType,
        priority: TicketPriority,
        description: Option<String>,
        assignee: Option<String>,
        blocked_by: Vec<String>,
        relates_to: Vec<String>,
        parent_id: Option<String>,
        discovered_from: Option<String>,
        source_plan: Option<String>,
    ) -> Result<Ticket, TicketError> {
        let now = Utc::now();
        let id = generate_id();
        for bid in &blocked_by {
            validate_id(bid)?;
        }
        for rid in &relates_to {
            validate_id(rid)?;
        }
        if let Some(ref pid) = parent_id {
            validate_id(pid)?;
        }
        if let Some(ref did) = discovered_from {
            validate_id(did)?;
        }

        let ticket = Ticket {
            id: id.clone(),
            title,
            r#type,
            status: TicketStatus::Open,
            priority,
            created_at: now,
            updated_at: now,
            description,
            assignee,
            blocked_by,
            relates_to,
            parent_id,
            discovered_from,
            source_plan,
            body: None,
        };

        let path = self.issues_dir.join(ticket_filename(&id));
        if path.exists() {
            return Err(TicketError::DuplicateId { id });
        }
        self.save(&ticket)?;
        Ok(ticket)
    }

    /// Lists tickets with optional filters. All filters are AND.
    pub fn list(
        &self,
        status_filter: Option<TicketStatus>,
        priority_filter: Option<TicketPriority>,
        assignee_filter: Option<&str>,
        type_filter: Option<TicketType>,
    ) -> Result<Vec<Ticket>, TicketError> {
        let mut tickets = self.load_all()?;
        if let Some(s) = status_filter {
            tickets.retain(|t| t.status == s);
        }
        if let Some(p) = priority_filter {
            tickets.retain(|t| t.priority == p);
        }
        if let Some(a) = assignee_filter {
            tickets.retain(|t| t.assignee.as_deref() == Some(a));
        }
        if let Some(ty) = type_filter {
            tickets.retain(|t| t.r#type == ty);
        }
        tickets.sort_by(|a, b| (a.priority.0, a.created_at).cmp(&(b.priority.0, b.created_at)));
        Ok(tickets)
    }

    /// Returns tickets that are actionable (open or in_progress) and have all blockers closed.
    pub fn ready(&self) -> Result<Vec<Ticket>, TicketError> {
        let all = self.load_all()?;
        let by_id: HashMap<&str, &Ticket> = all.iter().map(|t| (t.id.as_str(), t)).collect();

        let mut ready_list = Vec::new();
        for t in &all {
            if !t.is_actionable() {
                continue;
            }
            let all_blockers_closed = t
                .blocked_by
                .iter()
                .all(|bid| by_id.get(bid.as_str()).is_some_and(|b| b.is_closed()));
            if all_blockers_closed {
                ready_list.push(t.clone());
            }
        }
        ready_list.sort_by(|a, b| (a.priority.0, a.created_at).cmp(&(b.priority.0, b.created_at)));
        Ok(ready_list)
    }

    /// Updates a ticket by id. Only provided fields are updated; others stay unchanged.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &self,
        id: &str,
        title: Option<&str>,
        status: Option<TicketStatus>,
        r#type: Option<TicketType>,
        priority: Option<TicketPriority>,
        description: Option<Option<&str>>,
        assignee: Option<Option<&str>>,
        blocked_by: Option<Vec<String>>,
        relates_to: Option<Vec<String>>,
        parent_id: Option<Option<&str>>,
    ) -> Result<Ticket, TicketError> {
        let mut ticket = self.load_one(id)?;
        if let Some(t) = title {
            ticket.title = t.to_string();
        }
        if let Some(s) = status {
            ticket.status = s;
        }
        if let Some(ty) = r#type {
            ticket.r#type = ty;
        }
        if let Some(p) = priority {
            ticket.priority = p;
        }
        if let Some(d) = description {
            ticket.description = d.map(String::from);
        }
        if let Some(a) = assignee {
            ticket.assignee = a.map(String::from);
        }
        if let Some(b) = blocked_by {
            for bid in &b {
                validate_id(bid)?;
            }
            ticket.blocked_by = b;
        }
        if let Some(r) = relates_to {
            for rid in &r {
                validate_id(rid)?;
            }
            ticket.relates_to = r;
        }
        if let Some(p) = parent_id {
            ticket.parent_id = p.map(String::from);
            if let Some(pid) = &ticket.parent_id {
                validate_id(pid)?;
            }
        }
        ticket.updated_at = Utc::now();
        self.save(&ticket)?;
        Ok(ticket)
    }

    /// Closes a ticket (sets status to done or cancelled).
    pub fn close(
        &self,
        id: &str,
        status: TicketStatus,
        _reason: Option<&str>,
    ) -> Result<Ticket, TicketError> {
        if !matches!(status, TicketStatus::Done | TicketStatus::Cancelled) {
            return Err(TicketError::InvalidStatus {
                value: format!("{:?}", status),
            });
        }
        self.update(
            id,
            None,
            Some(status),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }
}

/// Parses a ticket file: YAML frontmatter between --- ... ---, optional body after.
fn parse_ticket_file(content: &str, _path: &Path) -> Result<Ticket, String> {
    let (yaml, body) = split_frontmatter(content)?;
    let mut ticket: Ticket = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
    let body = body.trim();
    if !body.is_empty() {
        ticket.body = Some(body.to_string());
    }
    if ticket.id.is_empty() {
        return Err("missing id in frontmatter".to_string());
    }
    validate_id(&ticket.id).map_err(|e| e.to_string())?;
    Ok(ticket)
}

fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let content = content.trim_start();
    if !content.starts_with(FRONTMATTER_DELIM) {
        return Err("file must start with --- (frontmatter)".to_string());
    }
    let rest = content[FRONTMATTER_DELIM.len()..].trim_start();
    let second = rest
        .find("\n---")
        .ok_or_else(|| "missing closing --- for frontmatter".to_string())?;
    let yaml = rest[..second].trim();
    let body = rest[second + 4..].trim_start(); // \n---
    Ok((yaml, body))
}

fn format_ticket_file(ticket: &Ticket) -> String {
    let mut out = String::from("---\n");
    // Serialize without body in YAML; body goes after the closing ---
    let mut value = serde_yaml::to_value(ticket).expect("ticket serializes");
    if let serde_yaml::Value::Mapping(ref mut map) = value {
        map.remove(serde_yaml::Value::String("body".into()));
    }
    let yaml = serde_yaml::to_string(&value).expect("value serializes to YAML");
    let yaml = yaml.trim_start_matches("---\n").trim_end_matches("\n---\n");
    out.push_str(yaml);
    out.push_str("\n---\n");
    if let Some(ref body) = ticket.body {
        out.push_str(body);
        if !body.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::types::TicketPriority;
    use tempfile::TempDir;

    #[test]
    fn split_frontmatter_parses() {
        let s = "---\nid: chr-a1b2\ntitle: Hi\n---\n\nbody here";
        let (yaml, body) = split_frontmatter(s).unwrap();
        assert!(yaml.contains("chr-a1b2"));
        assert_eq!(body.trim(), "body here");
    }

    #[test]
    fn create_list_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let store = TicketStore::at(tmp.path().to_path_buf());
        let t = store
            .create(
                "Test task".to_string(),
                TicketType::Task,
                TicketPriority(1),
                Some("desc".to_string()),
                None,
                vec![],
                vec![],
                None,
                None,
                None,
            )
            .unwrap();
        assert!(t.id.starts_with("chr-"));
        let list = store.load_all().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Test task");
    }

    #[test]
    fn find_or_init_creates_at_git_root() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();
        let store = TicketStore::find_or_init(path).unwrap();
        assert!(store.issues_dir().exists());
        let again = TicketStore::find(path).unwrap();
        assert_eq!(again.issues_dir(), store.issues_dir());
    }

    #[test]
    fn ready_excludes_blocked() {
        let tmp = TempDir::new().unwrap();
        let store = TicketStore::at(tmp.path().to_path_buf());
        let a = store
            .create(
                "Blocker".to_string(),
                TicketType::Task,
                TicketPriority(0),
                None,
                None,
                vec![],
                vec![],
                None,
                None,
                None,
            )
            .unwrap();
        let b = store
            .create(
                "Blocked".to_string(),
                TicketType::Task,
                TicketPriority(1),
                None,
                None,
                vec![a.id.clone()],
                vec![],
                None,
                None,
                None,
            )
            .unwrap();
        let ready = store.ready().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, a.id);
        store.close(&a.id, TicketStatus::Done, None).unwrap();
        let ready2 = store.ready().unwrap();
        assert_eq!(ready2.len(), 1);
        assert_eq!(ready2[0].id, b.id);
    }
}
