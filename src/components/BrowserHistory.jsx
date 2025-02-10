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

  // Grouping history by profile names
  const groupedHistory = history.reduce((grouped, entry) => {
    if (!grouped[entry.profile_display_name]) {
      grouped[entry.profile_display_name] = [];
    }
    grouped[entry.profile_display_name].push(entry);
    return grouped;
  }, {});

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-3xl font-semibold mb-6 text-gray-800">Recent Browser History</h2>

      {loading ? (
        <p className="text-center text-gray-600 text-lg">Loading...</p>
      ) : error ? (
        <p className="text-center text-red-600 text-lg">{error}</p>
      ) : (
        <div className="space-y-8">
          {Object.entries(groupedHistory).length === 0 ? (
            <p className="text-center text-gray-500 text-lg">No history found.</p>
          ) : (
            Object.entries(groupedHistory).map(([profile, entries], index) => (
              <div key={index} className="p-4 border border-gray-300 rounded-lg">
                <h3 className="text-xl font-semibold text-blue-600 mb-2">{profile}</h3>
                <ul className="list-disc pl-5">
                  {entries.map((entry, i) => (
                    <li key={i} className="mt-1">
                      <a href={entry.url} target="_blank" rel="noopener noreferrer" className="text-blue-500 hover:underline">
                        {entry.title || "No Title"}
                      </a>{" "}
                      - <span className="text-gray-500 text-sm">{entry.visit_time}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))
          )}
        </div>
      )}

      <button
        className="mt-6 px-5 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition"
        onClick={fetchBrowserHistory}
        disabled={loading}
      >
        {loading ? "Refreshing..." : "Refresh History"}
      </button>
    </div>
  );
};

export default BrowserHistory;
