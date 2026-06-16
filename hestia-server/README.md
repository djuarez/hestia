# hestia-server

Central API. Aggregates `hestia-agent` instances across the cluster, talks to
Nomad (planned), and serves the web UI (planned).

## Run

```sh
# bind :4300, aggregate two agents, serve the built UI
HESTIA_SERVER_ADDR=0.0.0.0:4300 \
HESTIA_AGENTS="mini1=http://10.0.0.11:4400,mini2=http://10.0.0.12:4400" \
HESTIA_UI_DIR=../hestia-ui/dist \
cargo run -p hestia-server
```

`HESTIA_AGENTS` is a comma-separated list of `name=url` (or bare `url`, whose
host:port becomes the name). `HESTIA_UI_DIR` is optional — when set, the server
also serves that built UI bundle (otherwise it is API-only).

## Endpoints (v0.1)

### Cluster reads

| Method | Path              | Description                                   |
|--------|-------------------|-----------------------------------------------|
| GET    | `/health`         | Server liveness probe                         |
| GET    | `/v1/nodes`       | Per-agent reachability + metrics (concurrent) |
| GET    | `/v1/containers`  | All containers across nodes, tagged by `node` |

Unreachable agents are reported as `online: false` (nodes) or skipped
(containers); they never fail the whole response.

### Action proxying

Forwarded to the owning node's agent (the `node` comes from the aggregated
container list). Reads use a 5 s per-request timeout; actions allow 60 s.

| Method | Path                                          | Proxies to agent          |
|--------|-----------------------------------------------|---------------------------|
| POST   | `/v1/nodes/{node}/containers`                 | create                    |
| DELETE | `/v1/nodes/{node}/containers/{id}`            | delete (`?force=`)        |
| POST   | `/v1/nodes/{node}/containers/{id}/start`      | start                     |
| POST   | `/v1/nodes/{node}/containers/{id}/stop`       | stop                      |
| GET    | `/v1/nodes/{node}/containers/{id}/logs`       | logs (WebSocket bridge)   |

## Status / TODO

- [x] Static agent registry from config (`HESTIA_AGENTS`).
- [x] Concurrent aggregation of node metrics and containers.
- [x] Proxy container actions to the owning node
      (create / delete / start / stop / logs WebSocket bridge).
- [x] Serve the `hestia-ui` bundle (`HESTIA_UI_DIR`).
- [ ] SPA fallback should return 200 (currently 404) — only matters once the
      UI uses client-side routing.
- [ ] Dynamic agent registration (agents self-register).
- [ ] _(optional, much later)_ Nomad integration (`src/nomad.rs`): list and
      manage jobs alongside native containers.
