import { useState } from "react";
import Dashboard from "./pages/Dashboard";
import Containers from "./pages/Containers";

type View = "dashboard" | "containers";

export default function App() {
  const [view, setView] = useState<View>("dashboard");

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="flame">🔥</span> Hestia
        </div>
        <nav className="nav">
          <button
            className={view === "dashboard" ? "active" : ""}
            onClick={() => setView("dashboard")}
          >
            Dashboard
          </button>
          <button
            className={view === "containers" ? "active" : ""}
            onClick={() => setView("containers")}
          >
            Containers
          </button>
        </nav>
      </header>
      <main className="content">
        {view === "dashboard" ? <Dashboard /> : <Containers />}
      </main>
    </div>
  );
}
