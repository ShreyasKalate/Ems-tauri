import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const BrowserHistory = () => {
  const [history, setHistory] = useState([]);
  const [filteredHistory, setFilteredHistory] = useState([]);
  const [profiles, setProfiles] = useState([]);
  const [browsers, setBrowsers] = useState([]);
  const [selectedProfile, setSelectedProfile] = useState("");
  const [selectedBrowser, setSelectedBrowser] = useState("");
  const [selectedGmail, setSelectedGmail] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchBrowserHistory = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke("get_browser_history");
      setHistory(result);

      // Extract unique browsers
      const uniqueBrowsers = [...new Set(result.map(entry => entry.browser))];
      setBrowsers(uniqueBrowsers);

      if (uniqueBrowsers.length > 0) {
        setSelectedBrowser(uniqueBrowsers[0]);
      }

      updateProfiles(result, uniqueBrowsers[0]);
    } catch (error) {
      console.error("Failed to fetch browser history:", error);
      setError("Failed to fetch browser history.");
    } finally {
      setLoading(false);
    }
  };

  const updateProfiles = (historyData, browser) => {
    const uniqueProfiles = [
      ...new Map(
        historyData
          .filter(entry => entry.browser === browser)
          .map(entry => [entry.profile_display_name, entry])
      ).values(),
    ];
    setProfiles(uniqueProfiles);

    if (uniqueProfiles.length > 0) {
      setSelectedProfile(uniqueProfiles[0].profile_display_name);
      setSelectedGmail(uniqueProfiles[0].gmail);
    } else {
      setSelectedProfile("");
      setSelectedGmail("Unknown");
    }

    updateFilteredHistory(historyData, browser, uniqueProfiles[0]?.profile_display_name || "");
  };

  const updateFilteredHistory = (historyData, browser, profile) => {
    setFilteredHistory(
      historyData.filter(entry => entry.browser === browser && entry.profile_display_name === profile)
    );
  };

  useEffect(() => {
    fetchBrowserHistory();
  }, []);

  useEffect(() => {
    updateProfiles(history, selectedBrowser);
  }, [selectedBrowser, history]);

  useEffect(() => {
    updateFilteredHistory(history, selectedBrowser, selectedProfile);
    const selectedProfileData = history.find(
      entry => entry.profile_display_name === selectedProfile && entry.browser === selectedBrowser
    );
    setSelectedGmail(selectedProfileData ? selectedProfileData.gmail : "Unknown");
  }, [selectedProfile, selectedBrowser, history]);

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-3xl font-semibold mb-6 text-gray-800">Browser History</h2>

      {loading ? (
        <p className="text-center text-gray-600 text-lg">Loading...</p>
      ) : error ? (
        <p className="text-center text-red-600 text-lg">{error}</p>
      ) : (
        <>
          {/* Dropdown to Select Browser */}
          <div className="mb-4">
            <label className="block text-lg font-semibold mb-2">Select Browser:</label>
            <select
              className="p-2 border border-gray-300 rounded-md w-full md:w-1/3"
              value={selectedBrowser}
              onChange={(e) => setSelectedBrowser(e.target.value)}
            >
              {browsers.map((browser, index) => (
                <option key={index} value={browser}>{browser}</option>
              ))}
            </select>
          </div>

          {/* Dropdown to Select Profile */}
          <div className="mb-4">
            <label className="block text-lg font-semibold mb-2">Select Profile:</label>
            <select
              className="p-2 border border-gray-300 rounded-md w-full md:w-1/3"
              value={selectedProfile}
              onChange={(e) => setSelectedProfile(e.target.value)}
            >
              {profiles.map((profile, index) => (
                <option key={index} value={profile.profile_display_name}>
                  {profile.profile_display_name} ({profile.browser})
                </option>
              ))}
            </select>
            <p className="mt-2 text-gray-600"><strong>Gmail ID:</strong> {selectedGmail}</p>
          </div>

          {/* History Table */}
          <div className="overflow-x-auto">
            <table className="w-full border-collapse border border-gray-300">
              <thead className="bg-gray-200">
                <tr>
                  <th className="border p-2 w-1/4">Title</th>
                  <th className="border p-2 w-1/2">URL</th>
                  <th className="border p-2 w-1/4">Visit Time</th>
                </tr>
              </thead>
              <tbody>
                {filteredHistory.map((entry, index) => (
                  <tr key={index} className="hover:bg-gray-100">
                    <td className="border p-2">{entry.title || "No Title"}</td>
                    <td className="border p-2 truncate">
                      <a href={entry.url} target="_blank" rel="noopener noreferrer" className="text-blue-500 hover:underline" title={entry.url}>
                        {entry.url.length > 50 ? entry.url.substring(0, 50) + "..." : entry.url}
                      </a>
                    </td>
                    <td className="border p-2">{entry.visit_time}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
};

export default BrowserHistory;
