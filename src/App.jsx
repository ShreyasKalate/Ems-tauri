import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import Sidebar from "./components/Sidebar";
import RunningApps from "./components/RunningApps";
import InstalledApps from "./components/InstalledApps";
import SystemMonitor from "./pages/SystemMonitor";
import BrowserHistory from "./components/BrowserHistory";
import VisibleApps from "./components/VisibleApps";
import UsbDevices from "./components/UsbDevices";
import "./App.css";
import AfkTracker from "./components/afkTracker";

function App() {
  useEffect(() => {
    const runInitialCommands = async () => {
      try {
        await invoke("get_visible_apps");
        await invoke("get_running_apps");
        await invoke("get_ram_usage");
        await invoke("get_browser_history");
        console.log("All monitoring commands executed at app launch");
      } catch (error) {
        console.error("Error executing initial commands:", error);
      }
    };
    runInitialCommands();
  }, []);

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
            <Route path="/installed-apps" element={<InstalledApps />} /> {/* Installed apps on click */}
            <Route path="/visible-apps" element={<VisibleApps />} />
            <Route path="/browser-history" element={<BrowserHistory />} />
            <Route path="/usb-devices" element={<UsbDevices />} />
            <Route path="/afk-tracker" element={<AfkTracker />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App;
