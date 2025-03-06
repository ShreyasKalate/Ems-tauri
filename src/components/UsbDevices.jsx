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
    <div className="p-4 bg-gray-100 rounded-lg shadow-md">
      <h2 className="text-xl font-bold mb-3">Connected USB Devices</h2>

      {error && <p className="text-red-500">{error}</p>}

      {devices.length === 0 ? (
        <p>No USB devices found.</p>
      ) : (
        <div className="space-y-4">
          {devices.map((device, index) => (
            <div key={index} className="p-3 border border-gray-300 rounded-lg bg-white">
              <h3 className="text-lg font-semibold">Device {index + 1}</h3>
              <p><strong>Vendor ID:</strong> {device.vendor_id}</p>
              <p><strong>Product ID:</strong> {device.product_id}</p>
              {device.manufacturer && <p><strong>Manufacturer:</strong> {device.manufacturer}</p>}
              {device.product && <p><strong>Product:</strong> {device.product}</p>}

              {device.is_storage ? (
                <>
                  <p className="text-green-600 font-semibold">🖴 Storage Device (Pendrive)</p>
                  {device.mount_path ? (
                    <>
                      <p><strong>Mounted At:</strong> {device.mount_path}</p>
                      <h4 className="font-bold mt-2">📂 Files:</h4>
                      {device.files && device.files.length > 0 ? (
                        <ul className="list-disc list-inside">
                          {device.files.map((file, idx) => (
                            <li key={idx} className="text-sm text-gray-700">{file}</li>
                          ))}
                        </ul>
                      ) : (
                        <p className="text-sm text-gray-500">No files found.</p>
                      )}
                    </>
                  ) : (
                    <p className="text-red-500">🚨 Pendrive detected but not mounted!</p>
                  )}
                </>
              ) : (
                <p className="text-blue-500">🔌 This is a non-storage USB device</p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default UsbDevices;
