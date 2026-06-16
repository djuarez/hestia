import { type FormEvent, useState } from "react";
import { api } from "../api";

interface Props {
  nodes: string[];
  onClose: () => void;
  onCreated: () => void;
}

export default function CreateForm({ nodes, onClose, onCreated }: Props) {
  const [node, setNode] = useState(nodes[0] ?? "");
  const [image, setImage] = useState("docker.io/library/alpine:latest");
  const [name, setName] = useState("");
  const [command, setCommand] = useState("sleep 3600");
  const [start, setStart] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await api.create(node, {
        image: image.trim(),
        name: name.trim() || undefined,
        command: command.trim() ? command.trim().split(/\s+/) : undefined,
        start,
      });
      onCreated();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  }

  return (
    <div className="overlay" onClick={onClose}>
      <form className="dialog" onClick={(e) => e.stopPropagation()} onSubmit={submit}>
        <h2>New container</h2>

        <label>
          Node
          <select value={node} onChange={(e) => setNode(e.target.value)} required>
            {nodes.length === 0 && <option value="">(no online nodes)</option>}
            {nodes.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>

        <label>
          Image
          <input value={image} onChange={(e) => setImage(e.target.value)} required />
        </label>

        <label>
          Name
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="(optional — runtime generates one)"
          />
        </label>

        <label>
          Command
          <input
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            placeholder="(optional)"
          />
        </label>

        <label className="check">
          <input type="checkbox" checked={start} onChange={(e) => setStart(e.target.checked)} />
          Start immediately
        </label>

        {error && <p className="error small">{error}</p>}

        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            Cancel
          </button>
          <button className="primary" type="submit" disabled={busy || !node}>
            {busy ? "Creating…" : "Create"}
          </button>
        </div>
      </form>
    </div>
  );
}
