import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const RamUsage = () => {
  const [ramUsage, setRamUsage] = useState("Fetching RAM usage...");
  const [loading, setLoading] = useState(true);

  const fetchRamUsage = async () => {
    setLoading(true);
    try {
      const usage = await invoke("get_ram_usage");
      setRamUsage(usage);
    } catch (error) {
      console.error("Failed to fetch RAM usage:", error);
      setRamUsage("Failed to fetch RAM usage.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRamUsage();
  }, []);

  return (
    <div className="p-4 border border-gray-300 rounded shadow-md">
      <h2 className="text-lg font-bold">RAM Usage</h2>
      {loading ? <p>Loading...</p> : <pre className="text-sm">{ramUsage}</pre>}
      <button
        className="mt-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        onClick={fetchRamUsage}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh RAM Usage"}
      </button>
    </div>
  );
};

export default RamUsage;
