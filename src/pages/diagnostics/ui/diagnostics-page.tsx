import { RevealLogsButton } from "@/features/reveal-logs";
import { useWorkspaceDiagnostics } from "@/features/workspace-data";
import { ConnectionPlanSummary } from "@/widgets/connection-plans";
import { DiagnosticsPanel } from "@/widgets/diagnostics-panel";

export function DiagnosticsPage() {
  const diagnostics = useWorkspaceDiagnostics();

  return (
    <div className="grid h-full gap-6 xl:grid-cols-[1.2fr_0.8fr]">
      <DiagnosticsPanel diagnostics={diagnostics} action={<RevealLogsButton />} />
      <ConnectionPlanSummary className="rounded-none border-x-0 border-t-0 xl:border-l" />
    </div>
  );
}
