# hestia-agent

Runs on each Mac Mini. Talks to the local Apple `container` runtime and
exposes a small REST API consumed by `hestia-server`.

## Run

```sh
# defaults: binds 0.0.0.0:4400, uses the `container` binary on PATH
cargo run -p hestia-agent

# override bind address and/or container CLI path
HESTIA_AGENT_ADDR=0.0.0.0:4400 \
HESTIA_CONTAINER_BIN=/usr/local/bin/container \
cargo run -p hestia-agent
```

Set `RUST_LOG=hestia_agent=debug` for verbose logs.

## Endpoints (v0.1)

| Method | Path                          | Description                          |
|--------|-------------------------------|--------------------------------------|
| GET    | `/health`                     | Liveness probe                       |
| GET    | `/v1/containers`              | List all containers on this node     |
| POST   | `/v1/containers`              | Create a container (`start` to run)  |
| DELETE | `/v1/containers/{id}`         | Delete a container (`?force=true`)   |
| POST   | `/v1/containers/{id}/start`   | Start a stopped container            |
| POST   | `/v1/containers/{id}/stop`    | Stop a running container             |
| GET    | `/v1/containers/{id}/logs`    | Stream logs over WebSocket (follow)  |
| GET    | `/v1/metrics`                 | Node metrics (CPU/RAM/temp)          |

### Create payload

```json
{
  "image": "docker.io/library/alpine:latest",
  "name": "my-container",
  "command": ["sh", "-c", "echo hello"],
  "env": ["KEY=VALUE"],
  "cpus": 2,
  "memory": "512M",
  "start": true
}
```

Only `image` is required.

## Status / TODO

- [x] List / start / stop / create / delete verified end-to-end against
      Apple `container` v1.0.0 (typed parsing in `src/container.rs`).
- [x] WebSocket log streaming (`container logs --follow`).
- [x] Real metrics via `sysinfo` — CPU/RAM verified; SoC temperature is
      best-effort (present on Apple Silicon here, `None` if unavailable).
- [ ] Image management endpoints (`pull` / `ls` / `rm`).
