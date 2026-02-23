// ChromaAI Dev - Terminal-first AI development, evaluation, and release tool
// Copyright (c) 2026 ChromaAI Dev Team

use anyhow::Result;
use chroma_ai_dev::generated;
use clap::{Parser, Subcommand};

/// ChromaAI Dev - Terminal-first AI development and evaluation tool
#[derive(Parser)]
#[command(name = "chroma")]
#[command(author, version, about, long_about = None)]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup logging
    setup_logging(cli.verbose)?;

    // Execute command
    match cli.command {
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
            
            // Create workspace directory
            let workspace_path = std::path::Path::new(&name);
            if workspace_path.exists() {
                println!("\n❌ Workspace '{}' already exists!", name);
                std::process::exit(1);
            }
            
            // Create directory structure
            std::fs::create_dir_all(workspace_path)?;
            std::fs::create_dir_all(workspace_path.join(".chroma"))?;
            std::fs::create_dir_all(workspace_path.join("prompts"))?;
            std::fs::create_dir_all(workspace_path.join("tools"))?;
            std::fs::create_dir_all(workspace_path.join("evals"))?;
            
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
            std::fs::write(workspace_path.join(".chroma/config.yaml"), config)?;
            
            // Create .gitignore
            std::fs::write(workspace_path.join(".gitignore"), ".chroma/\n*.log\n")?;
            
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
        None => {
            println!("ChromaAI Dev v{}", env!("CARGO_PKG_VERSION"));
            println!("Terminal-first AI development, evaluation, and release tool\n");
            println!("📚 Quick start:");
            println!("  chroma validate    - Validate schema files");
            println!("  chroma login       - Authenticate with SSO");
            println!("  chroma init <name> - Initialize workspace\n");
            println!("📖 For more commands, run: chroma --help");
            println!("\n🚀 Status: Active - use 'chroma validate' to verify schemas");
            Ok(())
        }
    }
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
