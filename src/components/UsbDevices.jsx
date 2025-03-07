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
    const interval = setInterval(fetchDevices, 10000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-6 bg-gray-100 rounded-lg shadow-md">
      <h2 className="text-2xl font-bold mb-4">Connected USB Devices</h2>

      {error && <p className="text-red-500 font-semibold">{error}</p>}

      {devices.length === 0 ? (
        <p className="text-gray-500">No USB devices found.</p>
      ) : (
        <div className="space-y-5">
          {devices.map((device, index) => (
            <div key={index} className="p-4 border border-gray-300 rounded-lg bg-white shadow-sm">
              <h3 className="text-lg font-semibold mb-1">Device {index + 1}</h3>
              <p className="text-gray-600"><strong>Vendor ID:</strong> {device.vendor_id}</p>
              <p className="text-gray-600"><strong>Product ID:</strong> {device.product_id}</p>
              {device.manufacturer && (
                <p className="text-gray-600"><strong>Manufacturer:</strong> {device.manufacturer}</p>
              )}
              {device.product && (
                <p className="text-gray-600"><strong>Product:</strong> {device.product}</p>
              )}

              {device.is_storage ? (
                <>
                  <p className="text-green-600 font-semibold mt-2">🖴 Storage Device (Pendrive)</p>
                  {device.mount_path ? (
                    <>
                      <p className="text-gray-700"><strong>Mounted At:</strong> {device.mount_path}</p>
                      <h4 className="font-bold mt-3">📂 Files:</h4>
                      {device.files && device.files.length > 0 ? (
                        <ul className="list-disc list-inside bg-gray-50 p-2 rounded-md max-h-40 overflow-y-auto">
                          {device.files.map((file, idx) => (
                            <li key={idx} className="text-sm text-gray-700">{file}</li>
                          ))}
                        </ul>
                      ) : (
                        <p className="text-sm text-gray-500">No files found.</p>
                      )}
                    </>
                  ) : (
                    <p className="text-red-500 font-semibold">🚨 Pendrive detected but not mounted!</p>
                  )}
                </>
              ) : (
                <p className="text-blue-500 mt-2">🔌 Non-Storage USB Device</p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default UsbDevices;
