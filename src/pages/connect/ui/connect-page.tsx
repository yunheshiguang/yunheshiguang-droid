import {
  CloudPhonePanel,
  ConnectDevicePanel,
  DiscoveredDeviceList,
  NetworkDeviceList,
} from "@/features/connect-device";
import { VendorGuidanceList } from "@/features/vendor-guidance";
import { useWorkspaceConnectionPlans, useWorkspaceDevices } from "@/features/workspace-data";
import { ConnectionPlanGrid } from "@/widgets/connection-plans";

export function ConnectPage() {
  const devices = useWorkspaceDevices();
  const transports = useWorkspaceConnectionPlans();

  return (
    <div className="space-y-6 p-6">
      <div className="grid gap-6 xl:grid-cols-[1.4fr_0.6fr]">
        <ConnectDevicePanel />
        <NetworkDeviceList devices={devices} />
      </div>
      <div className="grid gap-6 xl:grid-cols-2">
        <DiscoveredDeviceList transports={transports} />
        <VendorGuidanceList devices={devices} />
      </div>
      <CloudPhonePanel />
      <ConnectionPlanGrid />
    </div>
  );
}
