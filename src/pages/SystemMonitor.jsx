import RamUsage from "../components/RamUsage";

function SystemMonitor() {
  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">System Monitor</h1>
      <RamUsage />
    </div>
  );
}

export default SystemMonitor;
