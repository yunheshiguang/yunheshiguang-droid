import { LayoutTemplate } from "lucide-react";

import { collectDeviceTags } from "@/entities/device";
import { pickSelectedDevices, useSelectedDeviceIds } from "@/features/device-selection";
import { FlowEditor } from "@/features/edit-automation-flow";
import { AutomationRunPanel, RunHistory } from "@/features/run-automation";
import { SchedulePanel } from "@/features/schedule-automation";
import { TemplatePackages } from "@/features/share-packages";
import { FlowTemplateList } from "@/features/use-flow-template";
import { useWorkspaceDevices, useWorkspaceGroups } from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";
import { SectionCard } from "@/shared/ui";

export function AutomationPage() {
  const { t } = useI18n();
  const devices = useWorkspaceDevices();
  const groups = useWorkspaceGroups();
  const selectedDeviceIds = pickSelectedDevices(devices, useSelectedDeviceIds()).map(
    (device) => device.id,
  );
  const tags = collectDeviceTags(devices);

  return (
    <div className="grid gap-6 p-6 xl:grid-cols-[1.1fr_0.9fr]">
      <div className="space-y-6">
        <FlowEditor />
        <SectionCard icon={LayoutTemplate} title={t("automation.templates.title")}>
          <FlowTemplateList />
        </SectionCard>
        <TemplatePackages />
      </div>

      <div className="space-y-6">
        <AutomationRunPanel selectedDeviceIds={selectedDeviceIds} groups={groups} tags={tags} />
        <SchedulePanel selectedDeviceIds={selectedDeviceIds} groups={groups} tags={tags} />
        <RunHistory />
      </div>
    </div>
  );
}
