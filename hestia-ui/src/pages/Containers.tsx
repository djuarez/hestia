import { useState } from "react";
import { api, type Container } from "../api";
import { usePolling } from "../hooks";
import ContainerTable from "../components/ContainerTable";
import LogViewer from "../components/LogViewer";
import CreateForm from "../components/CreateForm";

export default function Containers() {
  const { data, error, loading, refresh } = usePolling(api.containers, 3000);
  const { data: nodes } = usePolling(api.nodes, 5000);
  const [logTarget, setLogTarget] = useState<Container | null>(null);
  const [creating, setCreating] = useState(false);

  const onlineNodes = (nodes ?? []).filter((n) => n.online).map((n) => n.name);

  return (
    <section>
      <div className="page-head">
        <h1>Containers</h1>
        <button className="primary" onClick={() => setCreating(true)} disabled={onlineNodes.length === 0}>
          + New container
        </button>
      </div>

      {error && <p className="error">Failed to load containers: {error}</p>}
      {loading && !data && <p className="muted">Loading…</p>}
      {data && data.length === 0 && <p className="muted">No containers across the cluster.</p>}

      {data && data.length > 0 && (
        <ContainerTable containers={data} onChanged={refresh} onLogs={setLogTarget} />
      )}

      {logTarget && <LogViewer container={logTarget} onClose={() => setLogTarget(null)} />}

      {creating && (
        <CreateForm
          nodes={onlineNodes}
          onClose={() => setCreating(false)}
          onCreated={() => {
            setCreating(false);
            refresh();
          }}
        />
      )}
    </section>
  );
}
