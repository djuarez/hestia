mod agents;
mod api;
mod error;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::agents::Registry;

/// hestia-server — central API. Aggregates `hestia-agent` instances across
/// the cluster and (later) talks to Nomad, plus serves the web UI.
#[derive(Debug, Parser)]
#[command(name = "hestia-server", version, about)]
struct Args {
    /// Address to bind the HTTP server to.
    #[arg(long, env = "HESTIA_SERVER_ADDR", default_value = "0.0.0.0:4300")]
    addr: SocketAddr,

    /// Agents to aggregate, as `name=url` (or bare `url`), comma-separated.
    /// e.g. `mini1=http://10.0.0.11:4400,mini2=http://10.0.0.12:4400`
    #[arg(long, env = "HESTIA_AGENTS", value_delimiter = ',')]
    agents: Vec<String>,

    /// Directory of built UI assets to serve (e.g. `hestia-ui/dist`). When
    /// unset, the server is API-only.
    #[arg(long, env = "HESTIA_UI_DIR")]
    ui_dir: Option<PathBuf>,
}

/// Shared application state, cheap to clone.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Registry>,
    pub http: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hestia_server=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let registry = Registry::from_specs(&args.agents);
    tracing::info!(agents = registry.len(), "agent registry loaded");
    for agent in registry.agents() {
        tracing::info!(name = %agent.name, url = %agent.base_url, "registered agent");
    }

    // Generous default for action proxying (e.g. `stop` waits on a grace
    // period). Aggregation reads set a shorter per-request timeout so a slow
    // node can't stall `/v1/nodes` or `/v1/containers`.
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let state = AppState {
        registry: Arc::new(registry),
        http,
    };

    let mut app = api::router(state);

    // Optionally serve the built UI. Unknown paths fall back to index.html so
    // the single-page app loads from any entry point.
    if let Some(dir) = &args.ui_dir {
        let index = dir.join("index.html");
        app = app.fallback_service(ServeDir::new(dir).not_found_service(ServeFile::new(index)));
        tracing::info!(dir = %dir.display(), "serving UI assets");
    }

    let listener = tokio::net::TcpListener::bind(args.addr).await?;
    tracing::info!(addr = %args.addr, "hestia-server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
