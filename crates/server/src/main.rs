mod embed;
mod error;
mod routes;
mod state;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use clap::Parser;
use folder_sync_core::config::resolve_config;
use state::AppState;
use tower_http::trace::TraceLayer;

#[derive(Parser, Debug)]
#[command(name = "folder-sync", about = "Keep backup drive clones in sync")]
struct Cli {
    /// Run in read-only mode: inspection only, no sync/delete execution allowed.
    /// Fixed for the whole session (not toggleable at runtime).
    #[arg(long, default_value_t = false)]
    read_only: bool,

    /// Port to bind the web UI/API to.
    #[arg(long, default_value_t = 13322)]
    port: u16,

    /// Root directory to scan for drive clones (default: /media/$USER).
    /// Overrides and persists into the saved config file.
    #[arg(long)]
    scan_root: Option<PathBuf>,

    /// Override the config file location (mainly for tests/devcontainer use).
    #[arg(long)]
    config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("folder_sync_server=info".parse()?))
        .init();

    let cli = Cli::parse();

    let config_path = match cli.config_path.clone() {
        Some(p) => p,
        None => folder_sync_core::config::config_file_path()?,
    };
    let config = resolve_config(&config_path, cli.scan_root.clone())?;

    let state = AppState::new(cli.read_only, config_path, config);

    let app = routes::api_router()
        .fallback(embed::static_handler)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), cli.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("folder-sync listening on http://{addr} (read_only={})", cli.read_only);

    axum::serve(listener, app).await?;

    Ok(())
}
