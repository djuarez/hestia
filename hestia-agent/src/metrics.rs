use axum::extract::State;
use axum::Json;
use serde::Serialize;
use sysinfo::{Components, System, MINIMUM_CPU_UPDATE_INTERVAL};

use crate::AppState;

/// Node-level metrics for a single Mac Mini.
#[derive(Debug, Clone, Serialize)]
pub struct NodeMetrics {
    pub cpu_usage_percent: f32,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    /// SoC temperature in °C. `None` when no sensor is exposed (common on
    /// Apple Silicon via the public API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_celsius: Option<f32>,
    pub container_count: u32,
}

/// GET /v1/metrics
pub async fn handler(State(state): State<AppState>) -> Json<NodeMetrics> {
    // `sysinfo` is synchronous and CPU sampling needs a short delay, so read
    // it on a blocking thread to avoid stalling the async runtime.
    let (cpu_usage_percent, memory_total_bytes, memory_used_bytes, temperature_celsius) =
        tokio::task::spawn_blocking(read_system)
            .await
            .unwrap_or((0.0, 0, 0, None));

    let container_count = state
        .containers
        .list()
        .await
        .map(|c| c.len() as u32)
        .unwrap_or(0);

    Json(NodeMetrics {
        cpu_usage_percent,
        memory_total_bytes,
        memory_used_bytes,
        temperature_celsius,
        container_count,
    })
}

/// Read CPU usage, memory, and (best-effort) temperature synchronously.
fn read_system() -> (f32, u64, u64, Option<f32>) {
    let mut sys = System::new();

    // CPU usage is a delta between two samples, so refresh, wait the minimum
    // interval, then refresh again before reading.
    sys.refresh_cpu_usage();
    std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    let cpu = sys.global_cpu_usage();

    sys.refresh_memory();
    let total = sys.total_memory();
    let used = sys.used_memory();

    // Apple Silicon typically exposes no components through the public API,
    // so this is best-effort and falls back to `None`.
    let components = Components::new_with_refreshed_list();
    let temp = components.iter().find_map(|c| c.temperature());

    (cpu, total, used, temp)
}
