import type { Node } from "../api";

function gib(bytes: number): string {
  return (bytes / 1024 ** 3).toFixed(1);
}

function Metric({ label, value, pct }: { label: string; value: string; pct: number }) {
  return (
    <div className="metric">
      <div className="row">
        <span>{label}</span>
        <strong>{value}</strong>
      </div>
      <div className="track">
        <div className="fill" style={{ width: `${Math.min(Math.max(pct, 0), 100)}%` }} />
      </div>
    </div>
  );
}

export default function NodeCard({ node }: { node: Node }) {
  const m = node.metrics;
  const memPct =
    m && m.memory_total_bytes > 0 ? (m.memory_used_bytes / m.memory_total_bytes) * 100 : 0;

  return (
    <div className={`card ${node.online ? "" : "offline"}`}>
      <div className="card-head">
        <span className={`dot ${node.online ? "on" : "off"}`} />
        <h3>{node.name}</h3>
      </div>

      {node.online && m ? (
        <div className="metrics">
          <Metric label="CPU" value={`${m.cpu_usage_percent.toFixed(0)}%`} pct={m.cpu_usage_percent} />
          <Metric
            label="RAM"
            value={`${gib(m.memory_used_bytes)} / ${gib(m.memory_total_bytes)} GiB`}
            pct={memPct}
          />
          <div className="row">
            <span>Temp</span>
            <strong>
              {m.temperature_celsius != null ? `${m.temperature_celsius.toFixed(1)} °C` : "—"}
            </strong>
          </div>
          <div className="row">
            <span>Containers</span>
            <strong>{m.container_count}</strong>
          </div>
        </div>
      ) : (
        <p className="error small">{node.error ?? "offline"}</p>
      )}
    </div>
  );
}
