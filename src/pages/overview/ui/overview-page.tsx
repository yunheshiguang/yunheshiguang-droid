import { ConnectionPlanGrid } from "@/widgets/connection-plans";
import { OverviewShell } from "@/widgets/overview";

export function OverviewPage() {
  return <OverviewShell connectionPlans={<ConnectionPlanGrid />} />;
}
