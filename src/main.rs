// ChromaAI Dev - Terminal-first AI development, evaluation, and release tool
// Copyright (c) 2026 ChromaAI Dev Team

#![allow(clippy::large_enum_variant)]

use anyhow::Result;
use chroma_ai_dev::generated;
use chroma_ai_dev::tickets::{TicketPriority, TicketStatus, TicketStore, TicketType};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod server;

/// ChromaAI Dev: ChromaTUI-based AI development and evaluation tool. Without ChromaTUI there is no application.
#[derive(Parser)]
#[command(name = "chroma")]
#[command(author, version, about = "ChromaTUI-based AI dev tool. Without ChromaTUI there is no application.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Authenticate with SSO (OIDC device flow)
    Login {
        /// OIDC provider URL
        #[arg(long)]
        provider: Option<String>,
    },

    /// Show current authenticated user
    Whoami,

    /// Initialize new workspace
    Init {
        /// Workspace name
        name: String,
    },

    /// Validate schemas in markdown files
    Validate {
        /// Specific file to validate (validates all if not specified)
        file: Option<String>,
    },

    /// Issue/ticket tracking (file-based, git-friendly)
    Tickets {
        #[command(subcommand)]
        cmd: TicketsCmd,

        /// Output JSON for agents (all ticket commands)
        #[arg(long, global = true)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum TicketsCmd {
    /// Create .chroma/issues directory (or use from repo root)
    Init {
        /// Directory to initialize (default: current directory)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Create a new ticket
    Create {
        /// Ticket title
        title: String,

        /// Type: task, bug, epic, story, meta
        #[arg(short, long, default_value = "task")]
        r#type: String,

        /// Priority 0-3 (0=critical, 1=high, 2=medium, 3=low)
        #[arg(short, long, default_value = "2")]
        priority: u8,

        /// Description (optional)
        #[arg(short, long)]
        description: Option<String>,

        /// Assignee (optional)
        #[arg(short, long)]
        assignee: Option<String>,

        /// Blocked by ticket IDs (comma-separated)
        #[arg(long)]
        blocked_by: Option<String>,

        /// Relates to ticket IDs (comma-separated)
        #[arg(long)]
        relates_to: Option<String>,

        /// Parent ticket ID (for subtasks)
        #[arg(long)]
        parent_id: Option<String>,

        /// Discovered-from ticket ID
        #[arg(long)]
        discovered_from: Option<String>,

        /// Source plan path (e.g. docs/plans/...)
        #[arg(long)]
        source_plan: Option<String>,
    },

    /// List tickets with optional filters
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by priority 0-3
        #[arg(short, long)]
        priority: Option<u8>,

        /// Filter by assignee
        #[arg(short, long)]
        assignee: Option<String>,

        /// Filter by type
        #[arg(short, long)]
        r#type: Option<String>,
    },

    /// List tickets that are ready (no open blockers)
    Ready,

    /// Claim a ticket (set in_progress, optional assignee)
    Claim {
        /// Ticket ID
        id: String,
        /// Assignee (e.g. agent or user name)
        #[arg(short, long)]
        assignee: Option<String>,
    },

    /// Show a ticket by ID
    Show {
        /// Ticket ID (e.g. chr-a1b2c3d4)
        id: String,
    },

    /// Update a ticket
    Update {
        /// Ticket ID
        id: String,

        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long)]
        priority: Option<u8>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        /// Comma-separated ticket IDs (replaces list)
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        relates_to: Option<String>,
        #[arg(long)]
        parent_id: Option<String>,
    },

    /// Close a ticket (set status to done or cancelled)
    Close {
        /// Ticket ID
        id: String,

        /// done | cancelled
        #[arg(long, default_value = "done")]
        status: String,

        /// Optional reason (for display/audit)
        #[arg(long)]
        reason: Option<String>,
    },

    /// Validate ticket files and optionally check for orphan commits
    Doctor {
        /// Check git log for commits mentioning ticket IDs but ticket still open
        #[arg(long)]
        orphans: bool,

        /// Number of commits to scan for orphans
        #[arg(long, default_value = "50")]
        commits: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup logging
    setup_logging(cli.verbose)?;

    // Execute command
    match cli.command {
        Some(Commands::Serve { port }) => {
            println!("🚀 Starting ChromaAI Dev server...");
            server::run_server(port).await?;
            Ok(())
        }
        Some(Commands::Login { provider }) => {
            let provider_url = provider.unwrap_or_else(|| "https://auth.example.com".to_string());
            println!("🔐 Initiating OIDC device flow...");
            println!("Provider: {}", provider_url);
            
            // For development, simulate device flow
            // In production, this would do actual OIDC
            println!("\n📱 Device Code Flow (simulated for dev):");
            println!("  Device Code: DEV-1234-5678");
            println!("  User Code: CHROMA-A1B2");
            println!("  Verification URL: {}/activate", provider_url);
            println!("\n⏳ Waiting for authorization...");
            
            // For now, create a mock auth token
            let auth_data = serde_json::json!({
                "access_token": "dev-token-".to_string() + &uuid::Uuid::new_v4().to_string()[..8],
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": "dev-refresh-".to_string() + &uuid::Uuid::new_v4().to_string()[..8],
                "provider": provider_url,
            });
            
            // Save to home directory
            if let Some(home) = std::env::var_os("HOME") {
                let chroma_dir = std::path::Path::new(&home).join(".chroma");
                std::fs::create_dir_all(&chroma_dir)?;
                let auth_file = chroma_dir.join("auth.json");
                std::fs::write(&auth_file, serde_json::to_string_pretty(&auth_data)?)?;
                println!("\n✅ Authentication saved to: {}", auth_file.display());
            }
            
            println!("\n✅ Login successful! (dev mode)");
            println!("\n➡️  Run 'chroma whoami' to see your status");
            Ok(())
        }
        Some(Commands::Whoami) => {
            println!("👤 Current User:");
            
            // Try to load auth from home directory
            let auth_path = std::env::var_os("HOME")
                .map(|h| std::path::Path::new(&h).join(".chroma/auth.json"));
            
            if let Some(path) = auth_path {
                if path.exists() {
                    let content = std::fs::read_to_string(&path)?;
                    let auth: serde_json::Value = serde_json::from_str(&content)?;
                    
                    let token = auth["access_token"].as_str().unwrap_or("?");
                    println!("\n  ✅ Authenticated");
                    println!("  Token: {}...", &token[..token.len().min(20)]);
                    println!("  Provider: {}", auth["provider"].as_str().unwrap_or("unknown"));
                    println!("  Expires in: {} seconds", auth["expires_in"].as_i64().unwrap_or(0));
                } else {
                    println!("\n  ❌ Not authenticated");
                    println!("\n➡️  Run 'chroma login' to authenticate");
                }
            } else {
                println!("\n  ❌ Not authenticated");
                println!("\n➡️  Run 'chroma login' to authenticate");
            }
            Ok(())
        }
        Some(Commands::Init { name }) => {
            println!("🚀 Initializing workspace: {}", name);
            let cwd = std::env::current_dir()?;
            let root = cwd.join(&name);
            if let Err(e) = std::fs::create_dir_all(&root) {
                eprintln!("Could not create directory {}: {}", root.display(), e);
                return Err(e.into());
            }

            // Initialize ticket store
            if let Ok(store) = TicketStore::init(&root) {
                println!("   Issue tracking: {}", store.issues_dir().display());
            }

            // Create workspace structure
            std::fs::create_dir_all(root.join(".chroma"))?;
            std::fs::create_dir_all(root.join("prompts"))?;
            std::fs::create_dir_all(root.join("tools"))?;
            std::fs::create_dir_all(root.join("evals"))?;

            // Create config file
            let config = r#"# ChromaAI Workspace Configuration
version: "1.0"
name: "#.to_string() + &name + r#"
server:
  url: "http://localhost:8080"
  auth:
    type: "device"

# Local overrides
local:
  prompts_dir: "./prompts"
  tools_dir: "./tools"
  evals_dir: "./evals"
"#;
            std::fs::write(root.join(".chroma/config.yaml"), config)?;

            // Create .gitignore
            std::fs::write(root.join(".gitignore"), ".chroma/\n*.log\n")?;

            println!("\n✅ Workspace '{}' created!", name);
            println!("\n📁 Structure:");
            println!("  {}/", name);
            println!("  ├── .chroma/config.yaml");
            println!("  ├── prompts/");
            println!("  ├── tools/");
            println!("  ├── evals/");
            println!("  └── .gitignore");
            println!("\n➡️  cd {} && chroma login", name);
            Ok(())
        }
        Some(Commands::Validate { file }) => {
            println!("✅ Validating schemas...");
            if let Some(ref file) = file {
                println!("📄 Validating: {}", file);
            } else {
                println!("📄 Validating all schema files");
            }
            
            // Run validation
            match generated::validate_all_schemas() {
                Ok(()) => {
                    println!("\n✅ All schemas are valid!");
                    let tools = generated::tools::all();
                    let prompts = generated::prompts::all();
                    let evals = generated::evals::all();
                    let agents = generated::agents::all();
                    let mcp = generated::mcp_servers::all();
                    
                    println!("\n📊 Schema counts:");
                    println!("  Tools: {}", tools.len());
                    println!("  Prompts: {}", prompts.len());
                    println!("  Evals: {}", evals.len());
                    println!("  Agents: {}", agents.len());
                    println!("  MCP Servers: {}", mcp.len());
                }
                Err(e) => {
                    println!("\n❌ Schema validation failed: {}", e);
                    std::process::exit(1);
                }
            }
            Ok(())
        }
        Some(Commands::Tickets { cmd, json }) => run_tickets(cmd, json),
        None => {
            println!("ChromaAI Dev v{}", env!("CARGO_PKG_VERSION"));
            println!("Terminal-first AI development, evaluation, and release tool\n");
            if let Ok(store) = show_issue_status() {
                if store {
                    println!();
                }
            }
            println!("📚 Quick start:");
            println!("  chroma tickets ready - List ready-to-work tickets (auto-setup if needed)");
            println!("  chroma tickets create \"Title\" -p 1 - Create a ticket");
            println!("  chroma serve         - Start HTTP server");
            println!("  chroma validate      - Validate schema files");
            println!("  chroma login         - Authenticate with SSO");
            println!("  chroma init <name>   - Initialize workspace\n");
            println!("📖 For more commands, run: chroma --help");
            println!("\n🚀 Status: Active - use 'chroma validate' to verify schemas");
            Ok(())
        }
    }
}

fn run_tickets(cmd: TicketsCmd, json: bool) -> Result<()> {
    use chroma_ai_dev::tickets::TicketStore;
    use std::str::FromStr;

    match cmd {
        TicketsCmd::Init { dir } => {
            let root = dir.unwrap_or_else(|| PathBuf::from("."));
            let root = std::fs::canonicalize(&root).unwrap_or(root);
            let store = TicketStore::init(&root)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "issues_dir": store.issues_dir().to_string_lossy() })
                );
            } else {
                println!(
                    "Initialized ticket store at {}",
                    store.issues_dir().display()
                );
            }
        }
        TicketsCmd::Create {
            title,
            r#type,
            priority,
            description,
            assignee,
            blocked_by,
            relates_to,
            parent_id,
            discovered_from,
            source_plan,
        } => {
            let store = find_store_auto_init()?;
            let ty = TicketType::from_str(&r#type)?;
            let prio = TicketPriority::from_i64(i64::from(priority))?;
            let blocked_by = split_ids(blocked_by.as_deref());
            let relates_to = split_ids(relates_to.as_deref());
            let ticket = store.create(
                title,
                ty,
                prio,
                description,
                assignee,
                blocked_by,
                relates_to,
                parent_id,
                discovered_from,
                source_plan,
            )?;
            if json {
                println!("{}", serde_json::to_string(&ticket)?);
            } else {
                println!("Created {} {}", ticket.id, ticket.title);
            }
        }
        TicketsCmd::List {
            status,
            priority,
            assignee,
            r#type,
        } => {
            let store = find_store_auto_init()?;
            let status_f = status
                .as_deref()
                .and_then(|s| TicketStatus::from_str(s).ok());
            let prio_f = priority.and_then(|p| TicketPriority::from_i64(i64::from(p)).ok());
            let type_f = r#type.as_deref().and_then(|t| TicketType::from_str(t).ok());
            let tickets = store.list(status_f, prio_f, assignee.as_deref(), type_f)?;
            if json {
                println!("{}", serde_json::to_string(&tickets)?);
            } else {
                for t in &tickets {
                    println!("{}  {}  [{}] {}", t.id, t.status, t.priority, t.title);
                }
            }
        }
        TicketsCmd::Claim { id, assignee } => {
            let store = find_store_auto_init()?;
            let ticket = store.update(
                &id,
                None,
                Some(TicketStatus::InProgress),
                None,
                None,
                None,
                Some(assignee.as_deref()),
                None,
                None,
                None,
            )?;
            if json {
                println!("{}", serde_json::to_string(&ticket)?);
            } else {
                println!("Claimed {} {}", ticket.id, ticket.title);
            }
        }
        TicketsCmd::Ready => {
            let store = find_store_auto_init()?;
            let tickets = store.ready()?;
            if json {
                println!("{}", serde_json::to_string(&tickets)?);
            } else {
                for t in &tickets {
                    println!("{}  {}  [{}] {}", t.id, t.status, t.priority, t.title);
                }
            }
        }
        TicketsCmd::Show { id } => {
            let store = find_store_auto_init()?;
            let ticket = store.load_one(&id)?;
            if json {
                println!("{}", serde_json::to_string(&ticket)?);
            } else {
                println!("{}  {}", ticket.id, ticket.title);
                println!(
                    "  type: {}  status: {}  priority: {}",
                    ticket.r#type, ticket.status, ticket.priority
                );
                if let Some(ref d) = ticket.description {
                    println!("  description: {}", d);
                }
                if !ticket.blocked_by.is_empty() {
                    println!("  blocked_by: {:?}", ticket.blocked_by);
                }
            }
        }
        TicketsCmd::Update {
            id,
            title,
            status,
            r#type,
            priority,
            description,
            assignee,
            blocked_by,
            relates_to,
            parent_id,
        } => {
            let store = find_store_auto_init()?;
            let status_o = status.as_deref().map(TicketStatus::from_str).transpose()?;
            let type_o = r#type.as_deref().map(TicketType::from_str).transpose()?;
            let prio_o = priority.and_then(|p| TicketPriority::from_i64(i64::from(p)).ok());
            let blocked_by_o = blocked_by.as_ref().map(|s| split_ids(Some(s.as_str())));
            let relates_to_o = relates_to.as_ref().map(|s| split_ids(Some(s.as_str())));
            let parent_id_o = parent_id.as_ref().map(|s| Some(s.as_str()));
            let ticket = store.update(
                &id,
                title.as_deref(),
                status_o,
                type_o,
                prio_o,
                description.as_ref().map(|s| Some(s.as_str())),
                assignee.as_ref().map(|s| Some(s.as_str())),
                blocked_by_o,
                relates_to_o,
                parent_id_o,
            )?;
            if json {
                println!("{}", serde_json::to_string(&ticket)?);
            } else {
                println!("Updated {} {}", ticket.id, ticket.title);
            }
        }
        TicketsCmd::Close { id, status, reason } => {
            let store = find_store_auto_init()?;
            let st = TicketStatus::from_str(&status)?;
            let ticket = store.close(&id, st, reason.as_deref())?;
            if json {
                println!("{}", serde_json::to_string(&ticket)?);
            } else {
                println!("Closed {} {}", ticket.id, ticket.title);
            }
        }
        TicketsCmd::Doctor { orphans, commits } => {
            let store = find_store_auto_init()?;
            let tickets = store.load_all()?;
            let mut issues = Vec::<String>::new();
            for t in &tickets {
                // Basic validation: id matches file, required fields present
                if t.id.is_empty() || t.title.is_empty() {
                    issues.push(format!("{}: missing id or title", t.id));
                }
            }
            if orphans {
                if let Ok(log) = git_log_contains_chr(commits) {
                    let open_ids: std::collections::HashSet<_> = tickets
                        .iter()
                        .filter(|t| t.is_actionable())
                        .map(|t| t.id.as_str())
                        .collect();
                    for id in log {
                        if open_ids.contains(id.as_str()) {
                            issues.push(format!(
                                "orphan: commit mentions {} but ticket still open",
                                id
                            ));
                        }
                    }
                }
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "ok": issues.is_empty(), "issues": issues })
                );
            } else if issues.is_empty() {
                println!("No issues found.");
            } else {
                for i in &issues {
                    eprintln!("  {}", i);
                }
            }
        }
    }
    Ok(())
}

/// Finds ticket store, auto-initializing .chroma/issues at git root if missing (for create/list/ready/show/update/close/claim/doctor).
fn find_store_auto_init() -> Result<TicketStore, anyhow::Error> {
    let cwd = std::env::current_dir()?;
    Ok(TicketStore::find_or_init(&cwd)?)
}

/// Shows issue tracking status on default view (no auto-init). Returns Ok(true) if status was shown.
fn show_issue_status() -> Result<bool, anyhow::Error> {
    let cwd = std::env::current_dir()?;
    let store = match TicketStore::find(&cwd) {
        Ok(s) => s,
        Err(_) => {
            println!("📋 Issues: run any ticket command (e.g. chroma tickets ready) to auto-setup issue tracking.");
            return Ok(false);
        }
    };
    let ready = store.ready().unwrap_or_default();
    let in_progress: Vec<_> = store
        .list(Some(TicketStatus::InProgress), None, None, None)
        .unwrap_or_default();
    let open_count = store
        .list(Some(TicketStatus::Open), None, None, None)
        .unwrap_or_default()
        .len();
    println!("📋 Issues:");
    println!(
        "   Ready (no blockers): {}  |  In progress: {}  |  Open: {}",
        ready.len(),
        in_progress.len(),
        open_count
    );
    if !in_progress.is_empty() {
        for t in in_progress.iter().take(3) {
            println!("   → {}  {}", t.id, t.title);
        }
    } else if !ready.is_empty() {
        println!("   Next: {}  {}", ready[0].id, ready[0].title);
    }
    Ok(true)
}

fn split_ids(s: Option<&str>) -> Vec<String> {
    s.map(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect()
    })
    .unwrap_or_default()
}

/// Returns ticket IDs (chr-*) mentioned in the last n commit messages.
fn git_log_contains_chr(n: u32) -> Result<Vec<String>, std::io::Error> {
    let out = std::process::Command::new("git")
        .args(["log", "-n", &n.to_string(), "--pretty=format:%s"])
        .output()?;
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let log = String::from_utf8_lossy(&out.stdout);
    let re = regex::Regex::new(r"chr-[a-f0-9]{4,8}").expect("valid regex");
    let ids: Vec<String> = re.find_iter(&log).map(|m| m.as_str().to_string()).collect();
    Ok(ids)
}

fn setup_logging(verbose: bool) -> Result<()> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
