import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import Sidebar from "./components/Sidebar";
import SystemMonitor from "./pages/SystemMonitor";
import RunningApps from "./components/RunningApps";
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
            {/* <Route path="/apps" element={<InstalledApps />} /> */}
          </Routes>
        </main>
      </div>
    </Router>
  );
}

export default App;
