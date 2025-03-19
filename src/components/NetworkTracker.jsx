import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const NetworkTracker = () => {
  const [network, setNetwork] = useState(null);
  const [devices, setDevices] = useState([]);
  const [error, setError] = useState(null);

  const fetchNetworkStatus = async () => {
    try {
      const data = await invoke("get_network_info");
      setNetwork(data);
    } catch (err) {
      console.error("Error fetching network status:", err);
      setError("Failed to fetch network status.");
    }
  };

  const fetchConnectedDevices = async () => {
    try {
      const data = await invoke("get_connected_devices_list");
      setDevices(data);
    } catch (err) {
      console.error("Error fetching connected devices:", err);
      setError("Failed to fetch connected devices.");
    }
  };

  useEffect(() => {
    fetchNetworkStatus();
    fetchConnectedDevices();
    const interval = setInterval(() => {
      fetchNetworkStatus();
      fetchConnectedDevices();
    }, 10000); 

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-6 bg-white shadow-md rounded-md">
      <h2 className="text-2xl font-semibold mb-4">Network Status</h2>

      {error ? (
        <p className="text-red-600">{error}</p>
      ) : network ? (
        <div>
          <p><strong>SSID:</strong> {network.ssid}</p>
          <p><strong>Private IP:</strong> {network.private_ip}</p>
          <p><strong>Public IP:</strong> {network.public_ip}</p>
          <p><strong>MAC Address:</strong> {network.mac_address}</p>
          <p><strong>Network Type:</strong> {network.network_type}</p>
        </div>
      ) : (
        <p className="text-gray-600">Loading network info...</p>
      )}

      <h2 className="text-xl font-semibold mt-6">Connected Devices</h2>
      {devices.length === 0 ? (
        <p className="text-gray-500">No devices found.</p>
      ) : (
        <table className="w-full border-collapse border border-gray-300 mt-4">
          <thead className="bg-gray-200">
            <tr>
              <th className="border p-2">IP Address</th>
              <th className="border p-2">MAC Address</th>
              <th className="border p-2">OS Type</th>
              <th className="border p-2">Device Type</th>
            </tr>
          </thead>
          <tbody>
            {devices.map((device, index) => (
              <tr key={index} className="hover:bg-gray-100">
                <td className="border p-2">{device.ip_address}</td>
                <td className="border p-2">{device.mac_address}</td>
                <td className="border p-2">{device.os_type}</td>
                <td className="border p-2">{device.device_type}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};

export default NetworkTracker;
