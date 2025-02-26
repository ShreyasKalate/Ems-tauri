import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const RamUsage = () => {
  const [ramUsage, setRamUsage] = useState({
    timestamp: "",
    min_ram_gb: 0,
    max_ram_gb: 0,
    avg_ram_gb: 0,
    total_ram_gb: 0,
    ram_usage_percent: 0,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  // Fetch RAM usage from Rust backend
  const fetchRamUsage = async () => {
    setLoading(true);
    try {
      const response = await invoke("get_ram_usage");
      const parsedData = JSON.parse(response);
      console.log("Fetched RAM usage:", parsedData);
      setRamUsage(parsedData || {});
    } catch (error) {
      console.error("Failed to fetch RAM usage:", error);
      setError("Error fetching RAM usage.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRamUsage(); // Fetch on mount
    const interval = setInterval(fetchRamUsage, 60000); // Auto-refresh every 1 min
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-4 border border-gray-300 rounded shadow-md bg-white">
      <h2 className="text-lg font-bold mb-2">RAM Usage Overview</h2>
      {loading ? (
        <p>Loading...</p>
      ) : error ? (
        <p className="text-red-500">{error}</p>
      ) : (
        <table className="w-full border-collapse border border-gray-300">
          <tbody>
            <tr>
              <td className="border p-2 font-semibold">Timestamp:</td>
              <td className="border p-2">{ramUsage.timestamp}</td>
            </tr>
            <tr>
              <td className="border p-2 font-semibold">Min RAM Usage:</td>
              <td className="border p-2">{ramUsage.min_ram_gb.toFixed(2)} GB</td>
            </tr>
            <tr>
              <td className="border p-2 font-semibold">Max RAM Usage:</td>
              <td className="border p-2">{ramUsage.max_ram_gb.toFixed(2)} GB</td>
            </tr>
            <tr>
              <td className="border p-2 font-semibold">Avg RAM Usage:</td>
              <td className="border p-2">{ramUsage.avg_ram_gb.toFixed(2)} GB</td>
            </tr>
            <tr>
              <td className="border p-2 font-semibold">Total RAM:</td>
              <td className="border p-2">{ramUsage.total_ram_gb.toFixed(2)} GB</td>
            </tr>
            <tr>
              <td className="border p-2 font-semibold">RAM Usage %:</td>
              <td className="border p-2">{ramUsage.ram_usage_percent.toFixed(2)}%</td>
            </tr>
          </tbody>
        </table>
      )}
      <button
        className="mt-3 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        onClick={fetchRamUsage}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh RAM Usage"}
      </button>
    </div>
  );
};

export default RamUsage;
