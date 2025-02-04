import { useState } from "react";
import SystemMonitor from "./pages/SystemMonitor";
import "./App.css";

function App() {
  const [showMonitor, setShowMonitor] = useState(false);

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <button
        className="p-2 bg-blue-500 text-white rounded"
        onClick={() => setShowMonitor(!showMonitor)}
      >
        {showMonitor ? "Hide System Monitor" : "Show System Monitor"}
      </button>

      {showMonitor && <SystemMonitor />}
    </main>
  );
}

export default App;