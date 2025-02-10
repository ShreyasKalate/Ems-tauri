import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const BrowserHistory = () => {
  const [history, setHistory] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchBrowserHistory = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke("get_browser_history");
      setHistory(result);
    } catch (error) {
      console.error("Failed to fetch browser history:", error);
      setError("Failed to fetch browser history.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchBrowserHistory();
  }, []);

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-2xl font-semibold mb-4">Recent Browser History</h2>

      {loading ? (
        <p className="text-center text-gray-600">Loading...</p>
      ) : error ? (
        <p className="text-center text-red-600">{error}</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full border-collapse border border-gray-300">
            <thead className="bg-gray-200">
              <tr>
                <th className="border p-2">Title</th>
                <th className="border p-2">URL</th>
                <th className="border p-2">Visit Time</th>
              </tr>
            </thead>
            <tbody>
              {history.map((entry, index) => (
                <tr key={index} className="hover:bg-gray-100">
                  <td className="border p-2">{entry.title}</td>
                  <td className="border p-2">
                    <a href={entry.url} target="_blank" rel="noopener noreferrer">
                      {entry.url}
                    </a>
                  </td>
                  <td className="border p-2">{entry.visit_time}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <button
        className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        onClick={fetchBrowserHistory}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh History"}
      </button>
    </div>
  );
};

export default BrowserHistory;
