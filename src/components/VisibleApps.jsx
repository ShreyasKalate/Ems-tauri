import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const VisibleApps = () => {
  const [apps, setApps] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchVisibleApps = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await invoke("get_visible_apps");
      const parsedData = JSON.parse(response);
      console.log("Visible applications fetched:", parsedData);
      setApps(parsedData);
    } catch (error) {
      console.error("Failed to fetch visible applications:", error);
      setError("Failed to fetch visible applications.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchVisibleApps();

    const interval = setInterval(async () => {
      try {
        const response = await invoke("get_visible_apps");
        const parsedData = JSON.parse(response);
        console.log("Visible applications updated:", parsedData);
        setApps(parsedData);
      } catch (error) {
        console.error("Error fetching visible applications:", error);
      }
    }, 1000); // Update every second

    return () => clearInterval(interval); // Cleanup on unmount
  }, []);

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-2xl font-semibold mb-4">Visible Applications</h2>

      {loading ? (
        <p className="text-center text-gray-600">Loading...</p>
      ) : error ? (
        <p className="text-center text-red-600">{error}</p>
      ) : apps.length > 0 ? (
        <div className="overflow-x-auto">
          <table className="w-full border-collapse border border-gray-300">
            <thead className="bg-gray-200">
              <tr>
                <th className="border p-2">PID</th>
                <th className="border p-2">Process Name</th>
                <th className="border p-2">Window Title</th>
                <th className="border p-2">Current Session</th>
                <th className="border p-2">Total Usage</th>
                <th className="border p-2">Top Usage</th>
              </tr>
            </thead>
            <tbody>
              {apps.map((app, index) => (
                <tr key={index} className="hover:bg-gray-100">
                  <td className="border p-2">{app.pid || "N/A"}</td>
                  <td className="border p-2 font-semibold">{app.name}</td>
                  <td className="border p-2">{app.window_title}</td>
                  <td className="border p-2 font-mono text-green-600">{app.curr_session}</td>
                  <td className="border p-2 font-mono text-blue-600">{app.total_usage}</td>
                  <td className="border p-2 font-mono text-red-600">{app.top_usage}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="text-center text-gray-500">No visible applications found.</p>
      )}
    </div>
  );
};

export default VisibleApps;
