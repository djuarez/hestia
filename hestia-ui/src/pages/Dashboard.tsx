import { api } from "../api";
import { usePolling } from "../hooks";
import NodeCard from "../components/NodeCard";

export default function Dashboard() {
  const { data: nodes, error, loading } = usePolling(api.nodes, 3000);

  return (
    <section>
      <h1>Cluster</h1>

      {loading && !nodes && <p className="muted">Loading nodes…</p>}
      {error && <p className="error">Failed to load nodes: {error}</p>}

      {nodes && nodes.length === 0 && (
        <p className="muted">
          No agents registered. Set <code>HESTIA_AGENTS</code> on the server.
        </p>
      )}

      <div className="grid">
        {nodes?.map((node) => (
          <NodeCard key={node.name} node={node} />
        ))}
      </div>
    </section>
  );
}
