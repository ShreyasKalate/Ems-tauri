import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const AfkTracker = () => {
  const [afkData, setAfkData] = useState({
    last_active: "",
    afk_start: null,
    afk_end: null,
    afk_duration: null,
    is_afk: false,
  });

  // Function to fetch AFK data
  const fetchAfkStatus = async () => {
    try {
      const data = await invoke("get_afk_status");
      setAfkData(data);
    } catch (error) {
      console.error("Error fetching AFK status:", error);
    }
  };

  // Fetch AFK status every second
  useEffect(() => {
    fetchAfkStatus(); // Initial call
    const interval = setInterval(fetchAfkStatus, 1000); // Polling every second

    return () => clearInterval(interval); // Cleanup interval on unmount
  }, []);

  return (
    <div className="max-w-lg mx-auto p-6 bg-gray-100 rounded-lg shadow-md">
      <h2 className="text-xl font-semibold mb-4">AFK Tracker</h2>

      <div className="mb-2">
        <span className="font-semibold">Last Active:</span>{" "}
        {afkData.last_active ? `${afkData.last_active} seconds ago` : "N/A"}
      </div>

      <div className="mb-2">
        <span className="font-semibold">AFK Start Time:</span>{" "}
        {afkData.afk_start ? afkData.afk_start : "Not AFK"}
      </div>

      <div className="mb-2">
        <span className="font-semibold">AFK End Time:</span>{" "}
        {afkData.afk_end ? afkData.afk_end : "N/A"}
      </div>

      <div className="mb-2">
        <span className="font-semibold">AFK Duration:</span>{" "}
        {afkData.afk_duration ? afkData.afk_duration : "N/A"}
      </div>

      <div className={`mt-4 p-2 text-center rounded ${afkData.is_afk ? "bg-red-500 text-white" : "bg-green-500 text-white"}`}>
        {afkData.is_afk ? "User is AFK" : "User is Active"}
      </div>
    </div>
  );
};

export default AfkTracker;
