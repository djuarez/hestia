# hestia-ui

React + TypeScript web UI for Hestia. Talks to `hestia-server` over same-origin
`/v1/...` paths (the Vite dev server proxies them).

## Develop

```sh
npm install
npm run dev          # http://localhost:5173
```

Requires a running `hestia-server` on `http://127.0.0.1:4300` (configurable in
`vite.config.ts`). The proxy forwards both HTTP and the log WebSocket.

```sh
npm run build        # type-check + production bundle into dist/
npm run typecheck    # types only
```

## Structure

```
src/
├── main.tsx              # entry
├── App.tsx               # top bar + tab navigation
├── api.ts                # typed hestia-server client
├── hooks.ts              # usePolling
├── styles.css            # dark "hearth" theme
├── pages/
│   ├── Dashboard.tsx     # cluster overview (node cards)
│   └── Containers.tsx    # container management
└── components/
    ├── NodeCard.tsx      # CPU / RAM / temp / count
    ├── ContainerTable.tsx# list + start/stop/delete
    ├── CreateForm.tsx    # create-container dialog
    └── LogViewer.tsx     # live log streaming over WebSocket
```

## Status / TODO

- [x] Dashboard with live node metrics (polled).
- [x] Container list with start / stop / delete / create.
- [x] Live log streaming over WebSocket.
- [x] Served directly by `hestia-server` in production (`HESTIA_UI_DIR=dist`).
- [ ] Per-node detail view; Nomad jobs view.
