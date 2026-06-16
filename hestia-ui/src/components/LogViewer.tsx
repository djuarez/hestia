import { useEffect, useRef, useState } from "react";
import { api, type Container } from "../api";

type Status = "connecting" | "open" | "closed";

export default function LogViewer({
  container,
  onClose,
}: {
  container: Container;
  onClose: () => void;
}) {
  const [lines, setLines] = useState<string[]>([]);
  const [status, setStatus] = useState<Status>("connecting");
  const boxRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ws = new WebSocket(api.logsUrl(container.node, container.id));
    ws.onopen = () => setStatus("open");
    // Keep the last 1000 lines to bound memory on chatty containers.
    ws.onmessage = (e) => setLines((prev) => [...prev.slice(-999), String(e.data)]);
    ws.onclose = () => setStatus("closed");
    ws.onerror = () => setStatus("closed");
    return () => ws.close();
  }, [container]);

  // Auto-scroll to the newest line.
  useEffect(() => {
    const box = boxRef.current;
    if (box) box.scrollTop = box.scrollHeight;
  }, [lines]);

  return (
    <div className="overlay" onClick={onClose}>
      <div className="logpanel" onClick={(e) => e.stopPropagation()}>
        <header>
          <span>
            logs · <span className="mono">{container.name}</span>{" "}
            <span className="muted">@{container.node}</span>
          </span>
          <span className={`tag ${status}`}>{status}</span>
          <button onClick={onClose} aria-label="Close">
            ✕
          </button>
        </header>
        <div className="logbox" ref={boxRef}>
          {lines.length === 0 && <div className="muted">waiting for output…</div>}
          {lines.map((line, i) => (
            <div key={i} className="logline">
              {line}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
