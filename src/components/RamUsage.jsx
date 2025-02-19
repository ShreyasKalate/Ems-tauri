import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const RamUsage = () => {
  const [ramUsage, setRamUsage] = useState({ total_ram_gb: 0, used_ram_gb: 0 });
  const [loading, setLoading] = useState(true);

  const fetchRamUsage = async () => {
    setLoading(true);
    try {
      const response = await invoke("get_ram_usage");
      const parsedData = JSON.parse(response);
      console.log("RAM usage:", parsedData);
      setRamUsage(parsedData);
    } catch (error) {
      console.error("Failed to fetch RAM usage:", error);
      setRamUsage({ total_ram_gb: "Error", used_ram_gb: "Error" });
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
      {loading ? (
        <p>Loading...</p>
      ) : (
        <pre className="text-sm">
          Total RAM: {ramUsage.total_ram_gb} GB{"\n"}
          Used RAM: {ramUsage.used_ram_gb} GB
        </pre>
      )}
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
