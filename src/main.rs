use clap::{Parser, Subcommand};
use gemini_proxy::config::{load_config, save_config};
use gemini_proxy::oauth::{start_oauth_flow, refresh_access_token};
use gemini_proxy::server::start_server;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "gemini-proxy")]
#[command(about = "OpenAI-compatible proxy for Google Gemini", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with Google
    Login,
    /// Remove saved credentials
    Logout,
    /// Check authentication status
    Status,
    /// Start the proxy server
    Start {
        /// Port to run on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        /// Host to bind to
        #[arg(short, long, default_value = "localhost")]
        host: String,
    },
    /// Set a specific Google Cloud project ID
    SetProject {
        #[arg(name = "projectId")]
        project_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Login => {
            println!("\n🔐 Starting OAuth flow for Gemini...\n");
            let result = start_oauth_flow().await?;
            
            let mut config = load_config()?;
            config.auth = Some(gemini_proxy::config::AuthConfig {
                access_token: result.access_token,
                refresh_token: result.refresh_token,
                expires_at: result.expires_at,
                email: Some(result.email.clone()),
            });
            save_config(&config)?;

            println!("\n═══════════════════════════════════════════════════════");
            println!("✅ Authentication successful!");
            println!("═══════════════════════════════════════════════════════");
            println!("   Email: {}", result.email);
            println!("   Expires: {}", chrono::DateTime::from_timestamp(result.expires_at as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "Unknown".to_string()));
            println!("\nYou can now start the proxy server:");
            println!("   gemini-proxy start");
            println!("═══════════════════════════════════════════════════════\n");
        }
        Commands::Logout => {
            let mut config = load_config()?;
            if config.auth.is_some() {
                config.auth = None;
                save_config(&config)?;
                println!("✅ Logged out successfully");
            } else {
                println!("No authentication found");
            }
        }
        Commands::Status => {
            let config = load_config()?;
            let auth = config.auth.as_ref();
            
            match auth {
                Some(auth) => {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs();
                    let is_valid = now < auth.expires_at;
                    
                    println!("\n═══════════════════════════════════════════════════════");
                    println!("📋 Authentication Status");
                    println!("═══════════════════════════════════════════════════════");
                    println!("   Email: {}", auth.email.as_deref().unwrap_or("Unknown"));
                    println!("   Project ID: {}", config.project_id.as_deref().unwrap_or("auto-detected"));
                    println!("   Expires: {}", chrono::DateTime::from_timestamp(auth.expires_at as i64, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "Unknown".to_string()));
                    println!("   Valid: {}", if is_valid { "✅ Yes" } else { "❌ No (expired)" });
                    println!("═══════════════════════════════════════════════════════\n");
                }
                None => {
                    println!("❌ Not authenticated. Run 'gemini-proxy login' first.");
                }
            }
        }
        Commands::Start { port, host } => {
            let mut config = load_config()?;
            let auth = config.auth.as_mut().ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'gemini-proxy login' first."))?;
            
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            
            if now >= auth.expires_at - 60 {
                println!("🔄 Access token expired, refreshing...");
                let (new_token, new_expires) = refresh_access_token(&auth.refresh_token).await?;
                auth.access_token = new_token;
                auth.expires_at = new_expires;
                save_config(&config)?;
            }
            
            start_server(&host, port).await?;
        }
        Commands::SetProject { project_id } => {
            let mut config = load_config()?;
            config.project_id = Some(project_id.clone());
            save_config(&config)?;
            println!("✅ Project ID set to: {}", project_id);
        }
    }

    Ok(())
}
