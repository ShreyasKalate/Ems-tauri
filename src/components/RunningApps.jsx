import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const RunningApps = () => {
  const [apps, setApps] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchRunningApps = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await invoke("get_running_apps");
      const parsedData = JSON.parse(response);
      console.log("Running applications:", parsedData);
      setApps(parsedData);
    } catch (error) {
      console.error("Failed to fetch running applications:", error);
      setError("Failed to fetch running applications.");
      setApps([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRunningApps();
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

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-2xl font-semibold mb-4">Running Applications</h2>

      <button
        className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 m-4"
        onClick={fetchRunningApps}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh List"}
      </button>

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
                <th className="border p-2">CPU Usage (%)</th>
                <th className="border p-2">Memory Usage (MB)</th>
                <th className="border p-2">Start Time</th>
                <th className="border p-2">Running Time</th>
              </tr>
            </thead>
            <tbody>
              {apps.map((app, index) => (
                <tr key={index} className="hover:bg-gray-100">
                  <td className="border p-2">{app.pid}</td>
                  <td className="border p-2 font-semibold">{app.name}</td>
                  <td className="border p-2">{app.cpu_usage.toFixed(2)}</td>
                  <td className="border p-2">{app.memory_usage_mb.toFixed(2)} MB</td>
                  <td className="border p-2">{app.start_time}</td>
                  <td className="border p-2 font-mono text-green-600">{app.running_time}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="text-center text-gray-500">No running applications found.</p>
      )}
    </div>
  );
};

export default RunningApps;
