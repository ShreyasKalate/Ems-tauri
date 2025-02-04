import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const RunningApps = () => {
  const [apps, setApps] = useState([]);
  const [loading, setLoading] = useState(true);

  const fetchRunningApps = async () => {
    setLoading(true);
    try {
      const result = await invoke("get_running_apps");
      setApps(result);
    } catch (error) {
      console.error("Failed to fetch running applications:", error);
      setApps(["Error fetching data."]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRunningApps();
  }, []);

  return (
    <div className="p-4 border border-gray-300 rounded shadow-md">
      <h2 className="text-lg font-bold">Running Applications</h2>
      {loading ? (
        <p>Loading...</p>
      ) : (
        <ul className="text-sm">
          {apps.length > 0 ? (
            apps.map((app, index) => <li key={index}>{app}</li>)
          ) : (
            <li>No running applications found.</li>
          )}
        </ul>
      )}
      <button
        className="mt-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        onClick={fetchRunningApps}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh List"}
      </button>
    </div>
  );
};

export default RunningApps;
