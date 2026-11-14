mod config;
mod identity;
mod registry;
mod serve;
mod wire;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name="ntps", version, about="NTPS developer hosting node")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Create the local NTPS identity and configuration.
    Init { #[arg(long)] registry: Option<String> },
    /// Print the local public node identity.
    Identity,
    /// Register this node with the configured registry.
    RegisterNode,
    /// Claim a domain for this node (registry API must support it).
    RegisterDomain { domain: String },
    /// Resolve a .ntp domain using the configured registry.
    Resolve { domain: String },
    /// Serve a directory over the NTPS framed TCP protocol.
    Serve {
        directory: PathBuf,
        #[arg(long, default_value="0.0.0.0:7443")] listen: SocketAddr,
        #[arg(long)] domain: Option<String>,
        #[arg(long, default_value="ntps.toml")] config: PathBuf,
    },
    /// Show effective configuration.
    Config { #[arg(long, default_value="ntps.toml")] file: PathBuf },
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { registry } => {
            let cfg = config::Config::default_with_registry(registry);
            let path = config::default_path()?;
            if path.exists() { anyhow::bail!("{} already exists; refusing to overwrite", path.display()); }
            identity::load_or_create(&cfg.identity_file)?;
            std::fs::write(&path, toml::to_string_pretty(&cfg)?)?;
            println!("Initialized NTPS at {}", path.display());
            println!("Identity: {}", identity::public_id(&cfg.identity_file)?);
        }
        Commands::Identity => {
            let cfg = config::load_default()?;
            println!("{}", identity::public_id(&cfg.identity_file)?);
        }
        Commands::RegisterNode => {
            let cfg = config::load_default()?;
            let id = identity::load_or_create(&cfg.identity_file)?;
            let result = registry::register_node(&cfg, &id).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::RegisterDomain { domain } => {
            let cfg = config::load_default()?;
            let id = identity::load_or_create(&cfg.identity_file)?;
            let result = registry::register_domain(&cfg, &id, &domain).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Resolve { domain } => {
            let cfg = config::load_default()?;
            let result = registry::resolve(&cfg, &domain).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Serve { directory, listen, domain, config: path } => {
            let cfg = config::load(&path).context("loading config")?;
            let root = std::fs::canonicalize(&directory).context("website directory does not exist")?;
            if !root.is_dir() { anyhow::bail!("serve path is not a directory"); }
            let id = identity::load_or_create(&cfg.identity_file)?;
            if let Some(name) = &domain {
                registry::register_node(&cfg, &id).await?;
                registry::register_domain(&cfg, &id, name).await?;
            }
            println!("NTPS node {} serving {} on {}", identity::public_id(&cfg.identity_file)?, root.display(), listen);
            println!("Protocol: NTPS/0.1 framed TCP (transport adapter; QUIC/P2P is a subsequent milestone)");
            serve::run(root, listen, cfg.max_file_bytes).await?;
        }
        Commands::Config { file } => {
            let cfg = config::load(&file)?;
            println!("{}", toml::to_string_pretty(&cfg)?);
        }
    }
    Ok(())
}
