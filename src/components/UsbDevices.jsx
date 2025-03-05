import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function UsbDevices() {
  const [devices, setDevices] = useState([]);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function fetchDevices() {
      try {
        const result = await invoke("list_usb_devices");
        setDevices(result);
      } catch (err) {
        setError(err.toString());
      }
    }

    fetchDevices();

    // Optional: Refresh USB devices list every 10 seconds
    const interval = setInterval(fetchDevices, 10000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-4 bg-gray-100 rounded-lg shadow-md">
      <h2 className="text-xl font-bold mb-3">Connected USB Devices</h2>

      {error && <p className="text-red-500">{error}</p>}

      {devices.length === 0 ? (
        <p>No USB devices found.</p>
      ) : (
        <ul className="list-disc list-inside space-y-2">
          {devices.map((device, index) => (
            <li key={index} className="p-2 border border-gray-300 rounded-lg">
              <strong>Vendor ID:</strong> {device.vendor_id} <br />
              <strong>Product ID:</strong> {device.product_id} <br />
              {device.manufacturer && (
                <span>
                  <strong>Manufacturer:</strong> {device.manufacturer} <br />
                </span>
              )}
              {device.product && (
                <span>
                  <strong>Product:</strong> {device.product}
                </span>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default UsbDevices;
