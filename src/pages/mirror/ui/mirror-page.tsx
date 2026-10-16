import { MirrorDeviceList, MirrorSettings } from "@/features/mirror-device";
import { useScrcpyState, useWorkspaceDevices } from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";
import { ErrorState } from "@/shared/ui";

export function MirrorPage() {
  const { t } = useI18n();
  const { state, error } = useScrcpyState();
  const devices = useWorkspaceDevices();

  return (
    <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
      <div className="space-y-4">
        {error ? <ErrorState title={t("mirror.failed")} detail={error} /> : null}
        <MirrorDeviceList devices={devices} state={state} />
      </div>
      <MirrorSettings state={state} />
    </div>
  );
}
