  import { Link } from "react-router-dom";

  const Sidebar = () => {
    return (
      <nav className="w-64 h-screen bg-gray-800 text-white p-4">
        <h2 className="text-xl font-bold mb-4">Dashboard</h2>
        <ul className="space-y-2">
          <li>
            <Link
              to="/"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Home
            </Link>
          </li>
          <li>
            <Link
              to="/system-monitor"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              System Monitor
            </Link>
          </li>
          <li>
            <Link
              to="/running-apps"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Running Apps
            </Link>
          </li>
          <li>
            <Link
              to="/visible-apps"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Visible Apps
            </Link>
          </li>
          <li>
            <Link
              to="/installed-apps"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Installed Apps
            </Link>
          </li>
          <li>
            <Link
              to="/browser-history"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Browser History
            </Link>
          </li>
          <li>
            <Link
              to="/usb-devices"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Usb Devices
            </Link>
          </li>
          <li>
            <Link
              to="/afk-tracker"
              className="block p-2 hover:bg-gray-700 rounded transition"
            >
              Afk Tracker
            </Link>
          </li>
        </ul>
      </nav>
    );
  };

  export default Sidebar;
