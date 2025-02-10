import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import RunningApps from "./components/RunningApps";
import InstalledApps from "./components/InstalledApps";
import SystemMonitor from "./pages/SystemMonitor";
import BrowserHistory from "./components/BrowserHistory";
import "./App.css";

function App() {
  return (
    <Router>
      <div className="flex">
        <Sidebar />
        <main className="flex-1 p-4">
          <h1>Welcome to Tauri + React</h1>
          <Routes>
            <Route path="/" element={<h2>Home Page</h2>} />
            <Route path="/system-monitor" element={<SystemMonitor />} />
            <Route path="/running-apps" element={<RunningApps />} />
            <Route path="/installed-apps" element={<InstalledApps />} />
            <Route path="/browser-history" element={<BrowserHistory />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App;
