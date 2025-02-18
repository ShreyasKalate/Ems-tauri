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
      const result = await invoke("get_visible_apps");
      setApps(result);
    } catch (error) {
      console.error("Failed to fetch visible applications:", error);
      setError("Failed to fetch visible applications.");
      setApps([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchVisibleApps(); // Initial fetch

    // Update running time locally every second
    const interval = setInterval(() => {
      setApps((prevApps) =>
        prevApps.map((app) => {
          const timeParts = app.running_time.split(":").map(Number);
          let [hours, minutes, seconds] = timeParts;

          seconds += 1;
          if (seconds === 60) {
            seconds = 0;
            minutes += 1;
          }
          if (minutes === 60) {
            minutes = 0;
            hours += 1;
          }

          return {
            ...app,
            running_time: `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`,
          };
        })
      );
    }, 1000);

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
                <th className="border p-2">Running Time</th>
              </tr>
            </thead>
            <tbody>
              {apps.map((app, index) => (
                <tr key={index} className="hover:bg-gray-100">
                  <td className="border p-2">{app.pid}</td>
                  <td className="border p-2 font-semibold">{app.name}</td>
                  <td className="border p-2 font-mono text-green-600">{app.running_time}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="text-center text-gray-500">No visible applications found.</p>
      )}

      <button
        className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        onClick={fetchVisibleApps}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh List"}
      </button>
    </div>
  );
};

export default VisibleApps;
