import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const InstalledApps = () => {
  const [systemApps, setSystemApps] = useState([]);
  const [userApps, setUserApps] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchInstalledApps = async () => {
    setLoading(true);
    setError(null);
    try {
      const [system, user] = await invoke("get_installed_apps");
      setSystemApps(system);
      setUserApps(user);
    } catch (error) {
      console.error("Failed to fetch installed applications:", error);
      setError("Failed to fetch installed applications.");
      setSystemApps([]);
      setUserApps([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchInstalledApps();
  }, []);

  const renderTable = (apps, title) => (
    <div className="mb-6">
      <h3 className="text-xl font-semibold mb-2">{title}</h3>
      {apps.length > 0 ? (
        <div className="overflow-x-auto">
          <table className="w-full border-collapse border border-gray-300">
            <thead className="bg-gray-200">
              <tr>
                <th className="border p-2">Identifying Number</th>
                <th className="border p-2">Install Date</th>
                <th className="border p-2">Install Location</th>
                <th className="border p-2">Name</th>
                <th className="border p-2">Vendor</th>
                <th className="border p-2">Version</th>
              </tr>
            </thead>
            <tbody>
              {apps.map((app, index) => (
                <tr key={index} className="hover:bg-gray-100">
                  <td className="border p-2">{app.identifying_number || "N/A"}</td>
                  <td className="border p-2">{app.install_date || "N/A"}</td>
                  <td className="border p-2">{app.install_location || "N/A"}</td>
                  <td className="border p-2 font-semibold">{app.name || "Unknown"}</td>
                  <td className="border p-2">{app.vendor || "Unknown"}</td>
                  <td className="border p-2">{app.version || "Unknown"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="text-center text-gray-500">No installed applications found.</p>
      )}
    </div>
  );

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-2xl font-semibold mb-4">Installed Applications</h2>

      {loading ? (
        <p className="text-center text-gray-600">Loading...</p>
      ) : error ? (
        <p className="text-center text-red-600">{error}</p>
      ) : (
        <>
          {renderTable(systemApps, "System-Wide Installed Applications")}
          {renderTable(userApps, "User-Specific Installed Applications")}
        </>
      )}

      <button
        className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        onClick={fetchInstalledApps}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh List"}
      </button>
    </div>
  );
};

export default InstalledApps;
