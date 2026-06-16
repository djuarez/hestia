# 🔥 Hestia

> Home for your Mac Mini cluster.

Hestia is an open-source cluster management UI for Apple Silicon Mac Minis — think Proxmox, but native to macOS and Apple Container.

It gives you a single web interface to manage containers, monitor nodes, and orchestrate workloads across multiple Mac Minis, without Docker Desktop, without heavy daemons, and without licensing fees.

---

## Why Hestia?

Apple's [`container`](https://github.com/apple/container) tool brings native Linux containers to macOS with per-container VM isolation, sub-second startup times, and zero idle memory footprint. Nomad handles orchestration. But there's no unified UI to manage it all.

Hestia fills that gap.

| Feature | Proxmox | Docker Desktop | Hestia |
|---|---|---|---|
| Web UI | ✅ | ✅ | ✅ |
| Multi-node | ✅ | ❌ | ✅ |
| Apple Silicon native | ❌ | ❌ | ✅ |
| Linux Containers | ✅ (LXC) | ✅ | ✅ (Apple Container) |
| Virtual Machines | ✅ (KVM) | ❌ | ✅ (Virtualization.framework) |
| Per-container VM isolation | ❌ | ❌ | ✅ |
| Open source | ✅ | ❌ | ✅ |
| No licensing fees | ✅ | ❌ | ✅ |

---

## Architecture

```
┌─────────────────────────────────────────┐
│              Browser UI                 │
│           (hestia-ui / React)           │
└──────────────────┬──────────────────────┘
                   │ HTTP / WebSocket
┌──────────────────▼──────────────────────┐
│           hestia-server (Rust)          │
│         Central API + Nomad API         │
└──────┬───────────────────────┬──────────┘
       │ HTTP                  │ HTTP
┌──────▼──────┐         ┌──────▼──────┐
│hestia-agent │   ...   │hestia-agent │
│  Mac Mini 1 │         │  Mac Mini N │
│  (Rust)     │         │  (Rust)     │
└──────┬──────┘         └──────┬──────┘
       │                       │
┌──────▼──────┐         ┌──────▼──────┐
│  container  │         │  container  │
│  apiserver  │         │  apiserver  │
└─────────────┘         └─────────────┘
```

**Two Rust binaries:**

- **`hestia-agent`** — runs on each Mac Mini. Talks to `container-apiserver` and exposes a local REST API with metrics, container management, and log streaming via WebSocket.
- **`hestia-server`** — runs on one node. Aggregates all agents, talks to Nomad, and serves the React UI.

---

## Features

- **Node dashboard** — CPU, RAM, temperature and container count per Mac Mini
- **Container management** — list, start, stop, create and delete containers across all nodes
- **Real-time logs** — WebSocket log streaming per container
- **Image management** — pull, list and remove OCI images
- **Nomad integration** — view and manage Nomad jobs alongside native containers
- **Setup scripts** — bootstrap a new node with a single command

---

## Stack

| Layer | Technology |
|---|---|
| Agent & Server | Rust (axum, tokio, reqwest) |
| UI | React + TypeScript |
| Container runtime | Apple Container (apple/container) |
| Orchestration | Nomad |
| Container format | OCI (compatible with any registry) |

---

## Requirements

- Mac with Apple Silicon (M1 or later)
- macOS 26 (Tahoe) or later
- [`container`](https://github.com/apple/container) v1.0.0+
- [Nomad](https://www.nomadproject.io/) (optional, for orchestration)

---

## Project Structure

```
hestia/
├── README.md
├── docs/
│   ├── architecture.md          # Deep dive on design decisions
│   ├── comparison-proxmox.md    # Guide for users coming from Proxmox
│   └── networking.md            # Per-container IP model explained
├── hestia-agent/                # Rust — runs on each Mac Mini
│   └── src/
│       ├── main.rs
│       ├── api/                 # REST endpoints
│       ├── container.rs         # container-apiserver client
│       └── metrics.rs           # CPU, RAM, temperature
├── hestia-server/               # Rust — central API
│   └── src/
│       ├── main.rs
│       ├── api/                 # REST + WebSocket
│       ├── agents.rs            # Agent registry
│       └── nomad.rs             # Nomad API client
├── hestia-ui/                   # React — web UI
│   └── src/
│       ├── pages/
│       │   ├── Dashboard.tsx    # Cluster overview
│       │   ├── Nodes.tsx        # Per-node view
│       │   └── Containers.tsx   # Container management
│       └── components/
└── scripts/
    ├── install-agent.sh         # Bootstrap hestia-agent on a Mac Mini
    ├── install-server.sh        # Bootstrap hestia-server
    └── examples/                # Pre-configured container stacks
        ├── postgres.sh
        ├── redis.sh
        └── nginx.sh
```

---

## Roadmap

### v0.1 — Containers
- [ ] `hestia-agent` — container list, start, stop, create, remove
- [ ] `hestia-agent` — real-time log streaming (WebSocket)
- [ ] `hestia-agent` — node metrics (CPU, RAM, temperature)
- [ ] `hestia-server` — agent registry and aggregation
- [ ] `hestia-ui` — node dashboard
- [ ] `hestia-ui` — container management
- [ ] `hestia-ui` — real-time logs viewer
- [ ] Setup scripts for bootstrap

### v0.2 — Polish
- [ ] Multi-arch OCI image support
- [ ] Container snapshots
- [ ] Scheduled backups
- [ ] Role-based access control

### v0.3 — Virtual Machines
- [ ] VM creation and management via `Virtualization.framework`
- [ ] Linux guest support (Ubuntu, Debian, Alpine)
- [ ] macOS guest support
- [ ] VM snapshots and restore
- [ ] `hestia-ui` — VM management (mirrors container UI)

### v0.4 — Beyond
- [ ] Community scripts and templates
- [ ] Metrics history and graphs
- [ ] Alerting (Telegram / webhook)

---

## Contributing

Hestia is in early development. Contributions, ideas and feedback are very welcome.

1. Fork the repo
2. Create a feature branch (`git checkout -b feat/your-feature`)
3. Commit your changes
4. Open a pull request

---

## Name

Hestia is the Greek goddess of the hearth and home — the fire that keeps the household running. A fitting name for the tool that keeps your Mac Mini cluster alive.

---

## License

MIT
