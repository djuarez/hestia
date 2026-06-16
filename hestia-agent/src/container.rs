use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// A normalized view of a container, as returned by the Hestia agent API.
///
/// This is our stable API contract — it does not necessarily match the raw
/// shape emitted by `container ls --format json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
}

/// Body for `POST /v1/containers`. Mirrors the common `container create`
/// flags; unset fields fall back to runtime/image defaults.
#[derive(Debug, Deserialize)]
pub struct CreateContainerRequest {
    /// Image reference, e.g. `docker.io/library/alpine:latest`.
    pub image: String,
    /// Container id/name. The runtime generates one if omitted.
    #[serde(default)]
    pub name: Option<String>,
    /// Init-process arguments (the command to run).
    #[serde(default)]
    pub command: Vec<String>,
    /// Environment variables in `KEY=VALUE` form.
    #[serde(default)]
    pub env: Vec<String>,
    /// Number of CPUs to allocate.
    #[serde(default)]
    pub cpus: Option<u32>,
    /// Memory limit, e.g. `512M` or `1G`.
    #[serde(default)]
    pub memory: Option<String>,
    /// Start the container immediately after creating it.
    #[serde(default)]
    pub start: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("container `{0}` not found")]
    NotFound(String),
    #[error("failed to execute `container` CLI: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("`container {cmd}` failed (exit {code}): {stderr}")]
    Command {
        cmd: String,
        code: i32,
        stderr: String,
    },
    #[error("failed to parse `container` output: {0}")]
    Parse(#[from] serde_json::Error),
}

impl ContainerError {
    /// Promote a generic command failure to [`ContainerError::NotFound`] when
    /// the CLI reports a missing container, so callers can map it to a 404.
    fn into_not_found(self, id: &str) -> Self {
        match &self {
            ContainerError::Command { stderr, .. } => {
                let s = stderr.to_lowercase();
                if s.contains("not found") || s.contains("no such") {
                    return ContainerError::NotFound(id.to_string());
                }
                self
            }
            _ => self,
        }
    }
}

/// Client for the local Apple `container` runtime.
///
/// v0.1 shells out to the `container` CLI, which talks to the
/// `container-apiserver` over its local socket. A future revision may speak
/// to that socket directly to avoid per-call process spawns.
pub struct ContainerClient {
    bin: String,
}

impl ContainerClient {
    pub fn new(bin: impl Into<String>) -> Self {
        Self { bin: bin.into() }
    }

    /// Run the `container` CLI with the given args, returning stdout on
    /// success or a structured error on a non-zero exit.
    async fn run(&self, args: &[&str]) -> Result<Vec<u8>, ContainerError> {
        tracing::debug!(bin = %self.bin, ?args, "invoking container CLI");

        let output = Command::new(&self.bin)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(ContainerError::Command {
                cmd: args.join(" "),
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Ok(output.stdout)
    }

    /// List all containers (running and stopped).
    pub async fn list(&self) -> Result<Vec<Container>, ContainerError> {
        let stdout = self.run(&["ls", "--all", "--format", "json"]).await?;
        let raw: Vec<RawContainer> = serde_json::from_slice(&stdout)?;
        Ok(raw.into_iter().map(Container::from).collect())
    }

    /// Start a stopped container.
    pub async fn start(&self, id: &str) -> Result<(), ContainerError> {
        self.run(&["start", id])
            .await
            .map(|_| ())
            .map_err(|e| e.into_not_found(id))
    }

    /// Stop a running container.
    pub async fn stop(&self, id: &str) -> Result<(), ContainerError> {
        self.run(&["stop", id])
            .await
            .map(|_| ())
            .map_err(|e| e.into_not_found(id))
    }

    /// Create a container (optionally starting it) and return its record.
    pub async fn create(
        &self,
        req: &CreateContainerRequest,
    ) -> Result<Container, ContainerError> {
        let mut args: Vec<String> = vec!["create".into()];
        if let Some(name) = &req.name {
            args.push("--name".into());
            args.push(name.clone());
        }
        for e in &req.env {
            args.push("--env".into());
            args.push(e.clone());
        }
        if let Some(cpus) = req.cpus {
            args.push("--cpus".into());
            args.push(cpus.to_string());
        }
        if let Some(mem) = &req.memory {
            args.push("--memory".into());
            args.push(mem.clone());
        }
        args.push(req.image.clone());
        args.extend(req.command.iter().cloned());

        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let stdout = self.run(&arg_refs).await?;
        // `container create` prints the new container id on stdout.
        let id = String::from_utf8_lossy(&stdout).trim().to_string();

        if req.start {
            self.start(&id).await?;
        }

        self.get(&id).await
    }

    /// Fetch a single container by id.
    pub async fn get(&self, id: &str) -> Result<Container, ContainerError> {
        self.list()
            .await?
            .into_iter()
            .find(|c| c.id == id)
            .ok_or_else(|| ContainerError::NotFound(id.to_string()))
    }

    /// Delete a container. With `force`, running containers are removed too.
    pub async fn delete(&self, id: &str, force: bool) -> Result<(), ContainerError> {
        let mut args = vec!["delete"];
        if force {
            args.push("--force");
        }
        args.push(id);
        self.run(&args)
            .await
            .map(|_| ())
            .map_err(|e| e.into_not_found(id))
    }

    /// Build a `container logs` command ready to spawn for streaming. Keeping
    /// the bin/arg logic here mirrors the other methods.
    pub fn logs_command(&self, id: &str, follow: bool, tail: Option<u32>) -> Command {
        let mut cmd = Command::new(&self.bin);
        cmd.arg("logs");
        if follow {
            cmd.arg("--follow");
        }
        if let Some(n) = tail {
            cmd.arg("-n").arg(n.to_string());
        }
        cmd.arg(id)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd
    }
}

// --- Raw deserialization types ---------------------------------------------
//
// These mirror the (nested) shape emitted by `container ls --format json` on
// Apple `container` v1.0.0. We only declare the fields we consume; serde
// ignores everything else. They are mapped into our stable `Container` API
// type via the `From` impl below.

#[derive(Debug, Deserialize)]
struct RawContainer {
    id: String,
    configuration: RawConfiguration,
    status: RawStatus,
}

#[derive(Debug, Deserialize)]
struct RawConfiguration {
    image: RawImage,
}

#[derive(Debug, Deserialize)]
struct RawImage {
    /// Full image reference, e.g. `docker.io/library/alpine:latest`.
    reference: String,
}

#[derive(Debug, Deserialize)]
struct RawStatus {
    /// Lifecycle state, e.g. `running` / `stopped`.
    state: String,
    #[serde(default)]
    networks: Vec<RawNetwork>,
}

#[derive(Debug, Deserialize)]
struct RawNetwork {
    /// CIDR-formatted address, e.g. `192.168.64.2/24`.
    #[serde(rename = "ipv4Address", default)]
    ipv4_address: Option<String>,
}

impl From<RawContainer> for Container {
    fn from(raw: RawContainer) -> Self {
        // The runtime has no separate display name; the user-provided id
        // doubles as the name (mirrors Docker's `--name` -> id behaviour).
        let ip = raw
            .status
            .networks
            .first()
            .and_then(|n| n.ipv4_address.as_deref())
            // Drop the `/prefixlen` suffix so consumers get a bare address.
            .map(|addr| addr.split('/').next().unwrap_or(addr).to_string());

        Container {
            name: raw.id.clone(),
            id: raw.id,
            image: raw.configuration.image.reference,
            status: raw.status.state,
            ip,
        }
    }
}
