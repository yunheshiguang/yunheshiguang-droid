import { pickSelectedDevices, useSelectedDeviceIds } from "@/features/device-selection";
import { ScriptConsole, ScriptEngineList } from "@/features/script-console";
import { useWorkspaceDevices } from "@/features/workspace-data";

export function ScriptsPage() {
  const devices = useWorkspaceDevices();
  const selectedDevices = pickSelectedDevices(devices, useSelectedDeviceIds());

  return (
    <div className="space-y-6 p-6">
      <ScriptConsole selectedDevices={selectedDevices} devices={devices} />
      <ScriptEngineList />
    </div>
  );
}
