import { Languages, MoonStar } from "lucide-react";

import { AndroidPlatformToolsSettings } from "@/features/android-platform-tools";
import { AssistantSettingsPanel } from "@/features/assistant-settings";
import { ApiGatewaySettings } from "@/features/manage-api-gateway";
import { LanguageSwitcher, ThemeSwitcher } from "@/features/workspace";
import { useWorkspaceSnapshot } from "@/features/workspace-data";
import { useI18n } from "@/shared/i18n";
import type { WorkspaceSnapshot } from "@/shared/proto";
import { SectionCard } from "@/shared/ui";

import { UpdateSettings } from "./update-settings";

const selectPlatformTools = (snapshot: WorkspaceSnapshot) => snapshot.androidPlatformTools;

export function SettingsPage() {
  const { t } = useI18n();
  const platformTools = useWorkspaceSnapshot(selectPlatformTools);

  return (
    <div className="space-y-4 p-6">
      <SectionCard title={t("settings.title")} />

      <div className="grid gap-4 md:grid-cols-2">
        <SectionCard icon={MoonStar} title={t("settings.theme.title")}>
          <ThemeSwitcher />
        </SectionCard>

        <SectionCard icon={Languages} title={t("settings.locale.title")}>
          <LanguageSwitcher />
        </SectionCard>
      </div>

      <UpdateSettings />
      <AndroidPlatformToolsSettings state={platformTools} />
      <AssistantSettingsPanel />
      <ApiGatewaySettings />
    </div>
  );
}
