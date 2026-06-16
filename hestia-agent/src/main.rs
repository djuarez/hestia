mod api;
mod container;
mod error;
mod metrics;

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::container::ContainerClient;

/// hestia-agent — runs on each Mac Mini. Talks to the local Apple
/// `container` runtime and exposes a REST API for container management
/// and node metrics, consumed by `hestia-server`.
#[derive(Debug, Parser)]
#[command(name = "hestia-agent", version, about)]
struct Args {
    /// Address to bind the HTTP server to.
    #[arg(long, env = "HESTIA_AGENT_ADDR", default_value = "0.0.0.0:4400")]
    addr: SocketAddr,

    /// Path to the Apple `container` CLI binary.
    #[arg(long, env = "HESTIA_CONTAINER_BIN", default_value = "container")]
    container_bin: String,
}

/// Shared application state, cheap to clone (everything behind an `Arc`).
#[derive(Clone)]
pub struct AppState {
    pub containers: Arc<ContainerClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hestia_agent=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let state = AppState {
        containers: Arc::new(ContainerClient::new(args.container_bin.clone())),
    };

    let app = api::router(state);

    let listener = tokio::net::TcpListener::bind(args.addr).await?;
    tracing::info!(addr = %args.addr, "hestia-agent listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Resolves when the process receives Ctrl-C, so axum can drain in-flight
/// requests before exiting.
async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
