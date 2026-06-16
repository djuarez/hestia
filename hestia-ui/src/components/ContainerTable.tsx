import { useState } from "react";
import { api, type Container } from "../api";

interface Props {
  containers: Container[];
  onChanged: () => void;
  onLogs: (c: Container) => void;
}

export default function ContainerTable({ containers, onChanged, onLogs }: Props) {
  const [busy, setBusy] = useState<string | null>(null);

  async function act(c: Container, fn: () => Promise<unknown>) {
    setBusy(`${c.node}/${c.id}`);
    try {
      await fn();
      onChanged();
    } catch (e) {
      alert(`Action failed: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setBusy(null);
    }
  }

  return (
    <table className="table">
      <thead>
        <tr>
          <th>Status</th>
          <th>Name</th>
          <th>Image</th>
          <th>Node</th>
          <th>IP</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {containers.map((c) => {
          const key = `${c.node}/${c.id}`;
          const running = c.status === "running";
          const isBusy = busy === key;
          return (
            <tr key={key}>
              <td>
                <span className={`status ${running ? "running" : "stopped"}`}>{c.status}</span>
              </td>
              <td className="mono">{c.name}</td>
              <td className="mono dim">{c.image}</td>
              <td>{c.node}</td>
              <td className="mono">{c.ip ?? "—"}</td>
              <td className="actions">
                {running ? (
                  <button disabled={isBusy} onClick={() => act(c, () => api.stop(c.node, c.id))}>
                    Stop
                  </button>
                ) : (
                  <button disabled={isBusy} onClick={() => act(c, () => api.start(c.node, c.id))}>
                    Start
                  </button>
                )}
                <button onClick={() => onLogs(c)}>Logs</button>
                <button
                  className="danger"
                  disabled={isBusy}
                  onClick={() => {
                    if (confirm(`Delete container "${c.name}" on ${c.node}?`)) {
                      act(c, () => api.remove(c.node, c.id));
                    }
                  }}
                >
                  Delete
                </button>
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
