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

  function renderFileStructure(files) {
    return (
      <ul className="list-disc list-inside bg-gray-50 p-2 rounded-md max-h-100 overflow-y-auto">
        {files.map((file, index) => (
          <li key={index} className="text-sm text-gray-700 ml-4">
            {file.is_dir ? (
              <span className="font-semibold text-blue-600">📁 {file.name}</span>
            ) : (
              <span className="text-gray-800">📄 {file.name}</span>
            )}
            {file.is_dir && file.files && file.files.length > 0 && (
              <div className="ml-4">{renderFileStructure(file.files)}</div>
            )}
          </li>
        ))}
      </ul>
    );
  }

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
              {device.is_storage && device.mount_path && device.files ? renderFileStructure(device.files) : null}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default UsbDevices;
