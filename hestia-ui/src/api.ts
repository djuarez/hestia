// Typed client for the hestia-server API. All paths are same-origin `/v1/...`
// (the Vite dev server proxies them to the server).

export interface NodeMetrics {
  cpu_usage_percent: number;
  memory_total_bytes: number;
  memory_used_bytes: number;
  temperature_celsius?: number;
  container_count: number;
}

export interface Node {
  name: string;
  url: string;
  online: boolean;
  metrics?: NodeMetrics;
  error?: string;
}

export interface Container {
  id: string;
  name: string;
  image: string;
  status: string;
  ip?: string;
  node: string;
}

export interface CreateRequest {
  image: string;
  name?: string;
  command?: string[];
  env?: string[];
  cpus?: number;
  memory?: string;
  start?: boolean;
}

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let detail = `${res.status} ${res.statusText}`;
    try {
      const body = await res.json();
      if (body?.error) detail = body.error;
    } catch {
      /* non-JSON body; keep the status line */
    }
    throw new Error(detail);
  }
  return res.json() as Promise<T>;
}

export const api = {
  nodes: () => fetch("/v1/nodes").then(json<Node[]>),
  containers: () => fetch("/v1/containers").then(json<Container[]>),

  start: (node: string, id: string) =>
    fetch(`/v1/nodes/${node}/containers/${id}/start`, { method: "POST" }).then(json),
  stop: (node: string, id: string) =>
    fetch(`/v1/nodes/${node}/containers/${id}/stop`, { method: "POST" }).then(json),
  remove: (node: string, id: string) =>
    fetch(`/v1/nodes/${node}/containers/${id}?force=true`, { method: "DELETE" }).then(json),
  create: (node: string, body: CreateRequest) =>
    fetch(`/v1/nodes/${node}/containers`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }).then(json<Container>),

  logsUrl: (node: string, id: string) => {
    const proto = location.protocol === "https:" ? "wss" : "ws";
    return `${proto}://${location.host}/v1/nodes/${node}/containers/${id}/logs`;
  },
};
